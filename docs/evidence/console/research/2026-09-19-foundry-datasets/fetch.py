import concurrent.futures,datetime,gzip,hashlib,json,pathlib,sys,urllib.request,urllib.parse
ROOT=pathlib.Path('/private/tmp/console-foundry-datasets-20260919')
def fetch(path):
 url='https://www.palantir.com'+path
 request=urllib.request.Request(url,headers={'User-Agent':'Mozilla/5.0 (documentation research)','Accept-Encoding':'identity'})
 with urllib.request.urlopen(request,timeout=40) as r:
  effective=r.geturl();assert urllib.parse.urlparse(effective).hostname in ['www.palantir.com','palantir.com']
  raw=r.read();assert r.status==200;assert 'text/html' in r.headers.get('content-type','')
  name='-'.join(path.strip('/').split('/')[2:]);out=ROOT/(name+'.html.gz');compressed=gzip.compress(raw,mtime=0);out.write_bytes(compressed)
  record={'requested_url':url,'effective_url':effective,'retrieved_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':r.status,'headers':dict(r.headers.items()),'path':str(out),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'compressed_sha256':hashlib.sha256(compressed).hexdigest(),'limitations':'Mutable public documentation captured at retrieval; not evidence of Console implementation or exhaustive vendor support.'}
  (ROOT/(name+'.json')).write_text(json.dumps(record,indent=2)+'\n');return {'source':name,'sha256':record['sha256'],'bytes':len(raw)}
with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
 for path,res in zip(sys.argv[1:],pool.map(fetch,sys.argv[1:])):print(json.dumps(res))
