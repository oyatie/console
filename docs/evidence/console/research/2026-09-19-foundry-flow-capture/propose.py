import pathlib,hashlib,json,gzip,sys,subprocess,datetime
from zoneinfo import ZoneInfo
P=pathlib.Path(__file__).parent;sys.path.insert(0,str(P));from read import read
from specs import specs,existing,qs,limits
R=pathlib.Path('/private/tmp/console-client-release-20260919');HEAD='c659210e37f3f0fa5b8dc7d3b9ed7b8e07b7d139';rawbase=subprocess.check_output(['git','show',HEAD+':docs/program/console-capability-registry.json'],cwd=R);reg=json.loads(rawbase);inv=reg['release_inventory'];srcdir='docs/evidence/console/research/2026-09-19-foundry-flow-capture/'
def sha(b):return hashlib.sha256(b).hexdigest()
def dump(n,x):(P/n).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
sources=[];bodies={};checks=0
for p in sorted(P.glob('*.json')):
 s=json.loads(p.read_text())
 if not isinstance(s,dict)or 'requested_url'not in s:continue
 z=(P/(p.stem+'.html.gz')).read_bytes();raw=gzip.decompress(z);assert len(raw)==s['bytes'] and sha(raw)==s['sha256'] and sha(z)==s['compressed_sha256'];checks+=3
 assert datetime.datetime.fromisoformat(s['retrieved_utc']).astimezone(ZoneInfo('America/New_York')).date().isoformat()=='2026-09-19'
 s['id']=p.stem;s['capture_path_at_retrieval']=s['path'];s['path']=srcdir+p.stem+'.html.gz';s['limitations']+=' Target date2026-09-19 America/New_York; actual2026-09-20UTC retrieval preserved, no authenticated historical snapshot claimed.';sources.append(s);bodies[s['id']]=' '.join(read(P/(p.stem+'.html.gz')).split())
byid={s['id']:s for s in sources};quotechecks=0
assert not set(byid)&{s['id']for s in inv['reference_freeze']['sources']}
def binding(sid,q):
 global quotechecks
 assert ' '.join(q.split())in bodies[sid],(sid,q);quotechecks+=1
 return {'source_id':sid,'artifact_path':byid[sid]['path'],'sha256':byid[sid]['sha256'],'quote':q,'matching':'HTML article whitespace normalized','scope':'Public reference only; Console implementation/acceptance not established.'}
leaves=[];bindings={}
for id,journey,sid,q,deps in specs:
 leaves.append({'id':id,'milestone':7 if id.startswith('F16.') else 8,'required':True,'owner':'workflow capture application use cases and canonical media document and policy owners','dependencies':deps,'journey':journey,'acceptance':{'outcome':journey,'command':None,'fixtures':None,'boundary':None,'status':'not_bound'},'states':{'planned':True,'implemented':False,'integration_accepted':False,'production_qualified':False,'released':False},'evidence':{},'blockers':['Exact owning paths, reviewed design/test candidate and executable acceptance are not yet bound.','Capture source restrictions retention browser/media support and export conversion need explicit contracts and independent security/recovery oracles.'],'dependency_status':'initial; exact owner/contract dependencies not bound for lane dispatch'});bindings[id]=[binding(sid,q)]
for id,s,q in existing:bindings.setdefault(id,[]).append(binding(s,q))
qual=[binding(s,q)for s,q in qs]
def matrix(name,status,source,quotes,leaf,**extra):return {'runtime':name,'reference_status':status,'source':source,'source_bindings':[binding(source,q)for q in quotes],'required_leaf':leaf,'console_acceptance':'not established','command':None,**extra}
matrices={'runtimes':[
matrix('Workflow recorder capture modes','Beta same-enrollment browser recording; exact browser/runtime support unspecified','flow-capture-record-a-workflow',[qs[7][1],qs[8][1],qs[9][1],qs[10][1],qs[11][1],qs[12][1]],'F09.workflow-recorder-controls',capture_triggers=['click-auto','manual-control','Cmd+Shift+S','Ctrl+Shift+S'],frame_modes=['full','scaled','custom'],origin_scope='same enrollment/domain; Console current authority remains mandatory'),
matrix('Workflow capture export formats','Manual content and captured assets require native successor equivalence and explicit conversion loss','flow-capture-generate-documentation',[qs[20][1],qs[21][1],qs[22][1],'Export as .zip: Bundles Markdown content and images into a .zip file that can be downloaded.'],'F09.capture-document-export',formats=['native document (Notepad equivalent)','ordered walkthrough','PDF','ZIP Markdown and images']),
matrix('Workflow capture optional audio and transcripts','Optional microphone assets and editable/regenerable transcripts; provider model and browser/version support not established','flow-capture-record-a-workflow',['Enable Record audio if you want spoken narration and transcription.',qs[19][1]],'F09.capture-audio-transcripts',intelligence_scope='LLM document generation remains separate; classical transcription contract required')
]}
known={x['id']for x in inv['leaves']};assert not known&{x['id']for x in leaves};graph={x['id']:x['dependencies']for x in inv['leaves']+leaves};seen=set();active=set()
def visit(k):
 assert k not in active
 if k in seen:return
 active.add(k)
 for d in graph[k]:assert d in graph,(k,d);visit(d)
 active.remove(k);seen.add(k)
for k in graph:visit(k)
patch=[{'op':'test','path':'/release_inventory/version','value':1}]
for s in sources:patch.append({'op':'add','path':'/release_inventory/reference_freeze/sources/-','value':s})
for l in leaves:patch.append({'op':'add','path':'/release_inventory/leaves/-','value':l})
for k,bs in bindings.items():
 path='/release_inventory/reference_freeze/source_bindings/'+k
 patch.extend([{'op':'add','path':path+'/-','value':b}for b in bs]if k in inv['reference_freeze']['source_bindings']else[{'op':'add','path':path,'value':bs}])
for q in qual:patch.append({'op':'add','path':'/release_inventory/reference_freeze/runtime_qualifications/-','value':q})
for x in matrices['runtimes']:
 assert not any(r['runtime']==x['runtime']for r in inv['support_matrices']['runtimes']['entries']);patch.append({'op':'add','path':'/release_inventory/support_matrices/runtimes/entries/-','value':x})
for l in limits:patch.append({'op':'add','path':'/release_inventory/reference_freeze/completion_limitations/-','value':l})
proposal={'kind':'bounded-flow-capture-reference-proposal','observed_head_sha':HEAD,'registry_sha256':sha(rawbase),'reference_date':'2026-09-19','reference_timezone':'America/New_York','retrieval_utc_date':'2026-09-20','exhaustive':False,'sources':sources,'leaves_to_append':leaves,'source_bindings':bindings,'support_matrix_additions':matrices,'compatibility_and_security_qualifications':qual,'limitations':limits}
dump('proposal.json',proposal);dump('candidate-patch.json',patch);dump('verification.json',{'source_count':len(sources),'new_leaf_count':len(leaves),'binding_keys':len(bindings),'qualification_count':len(qual),'matrix_rows':sum(map(len,matrices.values())),'quote_membership_checks':quotechecks,'size_and_hash_checks':checks,'combined_leaf_count':len(graph),'all_dependencies_resolve':True,'acyclic_dependencies':True,'runtime_tests_executed':0,'implementation_accepted':False})
dump('manifest.json',{p.name:sha(p.read_bytes())for p in sorted(P.iterdir())if p.is_file()and p.name!='manifest.json'});print((P/'verification.json').read_text());print('proposal_sha256',sha((P/'proposal.json').read_bytes()));print('patch_sha256',sha((P/'candidate-patch.json').read_bytes()))
