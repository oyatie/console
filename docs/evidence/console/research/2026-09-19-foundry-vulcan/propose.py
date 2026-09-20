import pathlib,hashlib,json,gzip,sys,subprocess,datetime
from zoneinfo import ZoneInfo
P=pathlib.Path(__file__).parent;sys.path.insert(0,str(P));from read import read
from specs import specs,existing,qs,limits
R=pathlib.Path('/private/tmp/console-client-release-20260919');HEAD='c659210e37f3f0fa5b8dc7d3b9ed7b8e07b7d139';rawbase=subprocess.check_output(['git','show',HEAD+':docs/program/console-capability-registry.json'],cwd=R);reg=json.loads(rawbase);inv=reg['release_inventory'];srcdir='docs/evidence/console/research/2026-09-19-foundry-vulcan/'
def sha(b):return hashlib.sha256(b).hexdigest()
def dump(n,x):(P/n).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
sources=[];bodies={};checks=0
for p in sorted(P.glob('*.json')):
 s=json.loads(p.read_text())
 if not isinstance(s,dict)or 'requested_url'not in s:continue
 z=(P/(p.stem+'.html.gz')).read_bytes();raw=gzip.decompress(z);assert len(raw)==s['bytes'] and sha(raw)==s['sha256'] and sha(z)==s['compressed_sha256'];checks+=3
 local_date=datetime.datetime.fromisoformat(s['retrieved_utc']).astimezone(ZoneInfo('America/New_York')).date().isoformat();s['reference_temporal_status']='pre-cutoff capture' if local_date=='2026-09-19' else 'post-cutoff capture; exact historical support unqualified'
 s['id']=p.stem;s['capture_path_at_retrieval']=s['path'];s['path']=srcdir+p.stem+'.html.gz';s['limitations']+=' Target date2026-09-19 America/New_York; actual retrieval preserved. '+s['reference_temporal_status']+'; Last-Modified is not historical equivalence. No authenticated historical snapshot claimed.';sources.append(s);bodies[s['id']]=' '.join(read(P/(p.stem+'.html.gz')).split())
 if s['id']=='announcements-release-notes':
  import re
  data=json.loads(re.search(r'<script id="__NEXT_DATA__"[^>]*>(.*?)</script>',raw.decode(),re.S).group(1));record=data['props']['pageProps']['releases'][6]['releaseNotes'][2];assert record['id']=='0c3d7d64-ff10-45ef-9a6e-7684478055b4';bodies[s['id']]=' '.join((record['description']+' '+record['longDescription']).split())
byid={s['id']:s for s in sources};quotechecks=0
assert not set(byid)&{s['id']for s in inv['reference_freeze']['sources']}
def binding(sid,q):
 global quotechecks
 assert ' '.join(q.split())in bodies[sid],(sid,q);quotechecks+=1
 return {'source_id':sid,'artifact_path':byid[sid]['path'],'sha256':byid[sid]['sha256'],'quote':q,'matching':'HTML __NEXT_DATA__ JSON announcement0c3d7d64-ff10-45ef-9a6e-7684478055b4 description/longDescription whitespace normalized' if sid=='announcements-release-notes' else 'HTML article whitespace normalized','reference_temporal_status':byid[sid]['reference_temporal_status'],'scope':'Public reference only; Console implementation/acceptance not established.'}
leaves=[];bindings={}
for id,journey,sid,q,deps in specs:
 leaves.append({'id':id,'milestone':8 if id.startswith('F10.') else 7,'required':True,'owner':'CAD analysis application use cases and canonical media schema Action and function owners','dependencies':deps,'journey':journey,'acceptance':{'outcome':journey,'command':None,'fixtures':None,'boundary':None,'status':'not_bound'},'states':{'planned':True,'implemented':False,'integration_accepted':False,'production_qualified':False,'released':False},'evidence':{},'blockers':['Exact owning paths, reviewed design/test candidate and executable acceptance are not yet bound.','Exact post-cutoff reference details retain historical-validity HOLD; geometry schema units policy numerical accuracy and recovery require independent acceptance.'],'dependency_status':'initial; exact owner/contract dependencies not bound for lane dispatch'});bindings[id]=[binding(sid,q)]
for id,s,q in existing:bindings.setdefault(id,[]).append(binding(s,q))
qual=[binding(s,q)for s,q in qs]
def matrix(name,status,source,quotes,leaf,**extra):return {'runtime':name,'reference_status':status,'source':source,'source_bindings':[binding(source,q)for q in quotes],'required_leaf':leaf,'console_acceptance':'not established','command':None,**extra}
matrices={'runtimes':[]}
for fmt in ['GLB','GLTF','STL','OBJ','PLY','3MF','DXF','LAS','LAZ']:
 category='mesh' if fmt in ['GLB','GLTF','STL','OBJ','PLY','3MF'] else 'drawing' if fmt=='DXF' else 'point-cloud'
 quote={'mesh':'Mesh formats: GLB, GLTF, STL, OBJ, PLY, 3MF','drawing':'Engineering drawings: DXF','point-cloud':'Point clouds: LAS, LAZ'}[category]
 row=matrix('Vulcan geometry format '+fmt,'Overview-supported direct rendering; post-cutoff unit/axis detail not historically qualified','vulcan-overview',[quote],'F08.cad-model-scenes',format=fmt,geometry=category,up_axis_default='Y' if fmt in ['GLB','GLTF'] else 'Z',units='file-declared' if fmt in ['GLB','GLTF','3MF'] else 'model_unit m/mm or explicit scene units',detail_reference_temporal_status='post-cutoff historical-validity HOLD')
 row['source_bindings'] += [binding('vulcan-schema-reference',qs[6][1]),binding('vulcan-schema-reference',qs[8][1])];matrices['runtimes'].append(row)
matrices['runtimes'] += [
matrix('Vulcan transform result schema','Post-cutoff detail; historical qualification pending','vulcan-widget',[qs[16][1],qs[17][1]],'F08.cad-part-transforms',required_fields=['id:string','transform:16 finite numbers column-major']),
matrix('Vulcan color result schema','Post-cutoff detail; historical qualification pending','vulcan-widget',[qs[18][1],qs[19][1]],'F08.cad-part-styling',fields=['partInstanceIds:string[]','color:CSS string?','opacity:number0..1?','selectedOpacity:number0..1?']),
matrix('Vulcan embedded variable directions','Post-cutoff exact directions; vendor dated announcement supports broad binding capability','vulcan-widget',[qs[14][1],qs[20][1],qs[21][1],qs[26][1],qs[27][1]],'F10.cad-widget-bindings',bidirectional=['part ID','part instance ID','camera target x/y/z','camera position x/y/z','camera zoom'],outputs=['click x/y/z','measurement scene distance number','measurement unit distance formatted string'])
]

catalog=json.loads((P/'schema-support-catalog.json').read_text())['source_tables']
for sid,tables in catalog.items():
 matrices['runtimes'].append({'runtime':'Vulcan exact schema tables '+sid,'reference_status':'Post-cutoff documentation; historical-validity HOLD and executable support unqualified','source':sid,'source_bindings':[binding(sid,' | '.join(row))for table in tables for row in table[1:]],'required_leaf':'F10.cad-widget-bindings' if sid=='vulcan-widget' else 'F08.cad-model-scenes','console_acceptance':'not established','command':None,'catalog_tables':tables})

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
proposal={'kind':'bounded-vulcan-reference-proposal','observed_head_sha':HEAD,'registry_sha256':sha(rawbase),'reference_date':'2026-09-19','reference_timezone':'America/New_York','retrieval_utc_date':'2026-09-20','exhaustive':False,'sources':sources,'leaves_to_append':leaves,'source_bindings':bindings,'support_matrix_additions':matrices,'compatibility_and_security_qualifications':qual,'limitations':limits}
dump('proposal.json',proposal);dump('candidate-patch.json',patch);dump('verification.json',{'source_count':len(sources),'new_leaf_count':len(leaves),'binding_keys':len(bindings),'qualification_count':len(qual),'matrix_rows':sum(map(len,matrices.values())),'quote_membership_checks':quotechecks,'size_and_hash_checks':checks,'combined_leaf_count':len(graph),'all_dependencies_resolve':True,'acyclic_dependencies':True,'runtime_tests_executed':0,'implementation_accepted':False})
dump('manifest.json',{p.name:sha(p.read_bytes())for p in sorted(P.iterdir())if p.is_file()and p.name!='manifest.json'});print((P/'verification.json').read_text());print('proposal_sha256',sha((P/'proposal.json').read_bytes()));print('patch_sha256',sha((P/'candidate-patch.json').read_bytes()))
