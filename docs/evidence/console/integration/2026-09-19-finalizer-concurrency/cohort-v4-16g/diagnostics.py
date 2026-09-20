"""Read-only diagnostics for one strictly identified disposable harness container."""
import datetime
import json
import os
from pathlib import Path
import re
import subprocess
import threading
import time
from urllib.parse import unquote, urlsplit

DOCKER = '/opt/homebrew/bin/docker'
IMAGE = 'postgres:18.4@sha256:65f70a152846cf504dff86e807007e9aeac98c3aeb7b62541b2c55ab9d264e56'
FIELDS = {'name': '.Name', 'image': '.Config.Image', 'ports': '.NetworkSettings.Ports',
          'state': '.State.Status', 'running': '.State.Running', 'oom_killed': '.State.OOMKilled',
          'exit_code': '.State.ExitCode', 'pid': '.State.Pid',
          'started_at': '.State.StartedAt', 'finished_at': '.State.FinishedAt',
          'memory_limit': '.HostConfig.Memory', 'memory_swap': '.HostConfig.MemorySwap',
          'nano_cpus': '.HostConfig.NanoCpus', 'shm_size': '.HostConfig.ShmSize',
          'pids_limit': '.HostConfig.PidsLimit', 'restart_count': '.RestartCount'}
FORMAT = '{' + ','.join(json.dumps(k) + ':{{json ' + v + '}}' for k,v in FIELDS.items()) + '}'

SAMPLE_COMMAND = r'''for f in /sys/fs/cgroup/memory.current /sys/fs/cgroup/memory.peak /sys/fs/cgroup/memory.events /proc/meminfo; do
  if test -r "$f"; then printf '%s\n' "$f"; cat "$f"; else printf 'unavailable:%s\n' "$f"; fi
done
for p in /proc/[0-9]*; do
  test -r "$p/comm" || continue
  IFS= read -r comm < "$p/comm" || continue
  case "$comm" in postgres*)
    printf 'postgres_pid=%s comm=%s\n' "${p##*/}" "$comm"
    if test -r "$p/status"; then awk '/^VmRSS:/ {print}' "$p/status"; fi ;;
  esac
done'''

class Diagnostics:
    def __init__(self, output):
        self.output = Path(output)
        uri = urlsplit(os.environ['DATABASE_URL'])
        match = re.fullmatch(r'console_cargo_test_([0-9]+)_contract', unquote(uri.path[1:]))
        user = os.environ.get('USER', 'user')
        if (uri.hostname != '127.0.0.1' or not uri.port or not match
            or not re.fullmatch(r'[A-Za-z0-9_-]+', user)):
            raise RuntimeError('diagnostic disposable container identity unavailable')
        self.name = 'console-cargo-postgres-' + user + '-' + match[1]
        self.port = str(uri.port)
        secrets = []
        for key,value in os.environ.items():
            if 'PASSWORD' in key or 'SECRET' in key or 'TOKEN' in key:
                if len(value) >= 4: secrets.append(value)
            if 'URL' in key:
                try:
                    pw = urlsplit(value).password
                    if pw: secrets.extend([pw,unquote(pw)])
                except ValueError: pass
        self.secrets = sorted(set(secrets), key=len, reverse=True)
        self.events=[];self.process=None;self.thread=None
        self.sampler_stop=threading.Event();self.sampler=None
        info = self.inspect()
        mappings = info.get('ports',{}).get('5432/tcp',[]) or []
        if (info.get('name') != '/' + self.name or info.get('image') != IMAGE
            or info.get('running') is not True
            or mappings != [{'HostIp':'127.0.0.1','HostPort':self.port}]):
            raise RuntimeError('diagnostic container image/identity/port mismatch')
        self.process = subprocess.Popen([DOCKER,'logs','--timestamps','--follow',self.name],
                                       stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
        self.thread = threading.Thread(target=self.stream,daemon=True);self.thread.start()
        self.capture('before-cold')
        self.sampler=threading.Thread(target=self.sample_memory,daemon=True);self.sampler.start()

    def redact(self, value):
        text=value.decode('utf-8','replace') if isinstance(value,bytes) else value
        for secret in self.secrets:text=text.replace(secret,'[REDACTED]')
        text=re.sub(r'(?i)postgres(?:ql)?://[^\s\x22\x27]+','[REDACTED_DATABASE_URI]',text)
        text=re.sub(r"(?i)(PASSWORD\s+)(?:E)?'(?:''|[^'])*'",r"\1'[REDACTED]'",text)
        return re.sub(r'(?i)(?<![0-9a-f])[0-9a-f]{64}(?![0-9a-f])','[REDACTED_HEX64]',text)

    def run(self,args):
        r=subprocess.run([DOCKER,*args],stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=10)
        return {'status':r.returncode,'stdout':self.redact(r.stdout),'stderr':self.redact(r.stderr)}

    def inspect(self):
        # The explicit projection excludes Env, labels, arguments, and State.Error.
        # Keep safe image digests intact for exact binding; never output raw inspect.
        r=subprocess.run([DOCKER,'inspect','--format',FORMAT,self.name],
                         stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,timeout=10)
        if r.returncode:raise RuntimeError('diagnostic selected-field inspect unavailable')
        return json.loads(r.stdout)

    def stream(self):
        try:
            with (self.output/'postgres-server.log').open('xb') as f:
                for line in self.process.stdout:
                    f.write(self.redact(line).encode());f.flush()
        except Exception as e:
            self.events.append({'kind':'log-stream-error','error_type':type(e).__name__})

    def sample_memory(self):
        count=0;failures=0;slow=0
        try:
            with (self.output/'postgres-memory-samples.jsonl').open('x') as f:
                while not self.sampler_stop.is_set():
                    started=time.monotonic()
                    sample={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
                            'sample_index':count,'target_interval_seconds':1}
                    try:
                        sample['observation']=self.run(['exec',self.name,'sh','-c',SAMPLE_COMMAND])
                        if sample['observation']['status'] != 0:failures+=1
                    except Exception as e:
                        sample['error_type']=type(e).__name__;failures+=1
                    elapsed=time.monotonic()-started
                    sample['elapsed_seconds']=round(elapsed,3)
                    if elapsed>1:slow+=1
                    f.write(json.dumps(sample)+'\n');f.flush();count+=1
                    self.sampler_stop.wait(max(0,1-elapsed))
        except Exception as e:
            self.events.append({'kind':'memory-sampler-error','error_type':type(e).__name__})
        finally:
            self.events.append({'kind':'memory-sampler-finish','samples':count,
                                'failed_samples':failures,'samples_over_one_second':slow})

    def capture(self,label):
        record={'label':label,'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
        try:record['inspect']=self.inspect()
        except Exception as e:record['inspect_error_type']=type(e).__name__
        for name,args in [
            ('stats',['stats','--no-stream','--format','{{json .}}',self.name]),
            ('memory_cgroup',['exec',self.name,'sh','-c','for f in /sys/fs/cgroup/memory.events /sys/fs/cgroup/memory.current /sys/fs/cgroup/memory.max /sys/fs/cgroup/memory.stat; do if test -r "$f"; then echo "$f"; cat "$f"; fi; done']),
            ('shared_memory',['exec',self.name,'df','-k','/dev/shm'])]:
            try:record[name]=self.run(args)
            except Exception as e:record[name]={'error_type':type(e).__name__}
        self.events.append(record);self.save()

    def save(self):
        (self.output/'postgres-diagnostics.json').write_text(json.dumps(self.events,indent=2)+'\n')

    def close(self):
        self.sampler_stop.set()
        if self.sampler is not None:self.sampler.join(timeout=11)
        self.events.append({'kind':'memory-sampler-joined','thread_finished':self.sampler is None or not self.sampler.is_alive()})
        self.capture('observer-finish')
        if self.process is not None and self.process.poll() is None:
            self.process.terminate()
            try:self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:self.process.kill();self.process.wait()
        if self.thread is not None:self.thread.join(timeout=5)
        self.events.append({'kind':'log-stream-finish','returncode':None if self.process is None else self.process.returncode,
                            'thread_finished':self.thread is None or not self.thread.is_alive()})
        self.save()
