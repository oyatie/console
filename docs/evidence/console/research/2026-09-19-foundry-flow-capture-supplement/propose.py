import copy,datetime,gzip,hashlib,json,pathlib,re,shutil,subprocess
from zoneinfo import ZoneInfo
P=pathlib.Path(__file__).parent;R=pathlib.Path('/private/tmp/console-client-release-20260919');HEAD='c659210e37f3f0fa5b8dc7d3b9ed7b8e07b7d139'
def sha(b):return hashlib.sha256(b).hexdigest()
def dump(n,v):(P/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
def apply(reg,ops):
 for op in ops:
  path=op['path'].split('/')[1:];k=path.pop();at=reg
  for key in path:at=at[int(key)]if isinstance(at,list)else at[key]
  if op['op']=='test':assert at[k]==op['value']
  elif op['op']=='add':
   if k=='-':assert isinstance(at,list);at.append(op['value'])
   else:assert k not in at,(op['path'],'replacement forbidden');at[k]=op['value']
  else:raise ValueError(op['op'])
rawbase=subprocess.check_output(['git','show',HEAD+':docs/program/console-capability-registry.json'],cwd=R);base=json.loads(rawbase);reg=copy.deepcopy(base);prereqs=[]
for stem,expected in [('flow-capture','964d2ca200558c12bafafc0ef113bdc222711c6f124482879c1514af2cfc5e28'),('vulcan','4b04c5d6038a0d4c15ee029683ebf83a4e71c0ff8cd3d98f2ef98a096e480a10')]:
 p=pathlib.Path('/private/tmp/console-foundry-'+stem+'-20260919/candidate-patch.json');raw=p.read_bytes();assert sha(raw)==expected;apply(reg,json.loads(raw));prereqs.append({'packet':str(p.parent),'patch_sha256':expected,'order':len(prereqs)+1})
inv=reg['release_inventory'];assert len(inv['leaves'])==400
source=next(s for s in inv['reference_freeze']['sources']if s['id']=='announcements-release-notes');z=pathlib.Path('/private/tmp/console-foundry-vulcan-20260919/announcements-release-notes.html.gz').read_bytes();raw=gzip.decompress(z);assert sha(raw)==source['sha256']and sha(z)==source['compressed_sha256']and len(raw)==source['bytes'];(P/'announcements-release-notes.html.gz').write_bytes(z);dump('reused-source.json',source)
assert datetime.datetime.fromisoformat(source['retrieved_utc']).astimezone(ZoneInfo('America/New_York')).date().isoformat()=='2026-09-20'
data=json.loads(re.search(r'<script id="__NEXT_DATA__"[^>]*>(.*?)</script>',raw.decode(),re.S).group(1));records={}
ids=['a21993316797017cbc9a5fef35917a7b2d3c90ff4efe3c367b6ac9ddf0dc07d6','886238144efc782699c2a8738ce997fddd82413138b3212104fc482bdf730c30','3dda01de40b1da0b2e4d9e2f75a96dfcfe12f3c2fbb17fdd2dc24db8a13e8325','fdb44475cd28e8d43e82dd85c1fc327539565997fd83e3b0587c90b38f3c3692']
for i,release in enumerate(data['props']['pageProps']['releases']):
 for k,n in enumerate(release['releaseNotes']):
  if n['id']in ids:records[n['id']]={'json_path':f'/props/pageProps/releases/{i}/releaseNotes/{k}','record':n}
assert set(records)==set(ids);dump('dated-announcements.json',records)
quotechecks=0
def bind(id,q):
 global quotechecks
 record=records[id];assert ' '.join(q.split())in' '.join(record['record']['description'].split());quotechecks+=1
 return {'source_id':source['id'],'artifact_path':source['path'],'sha256':source['sha256'],'quote':q,'matching':'HTML __NEXT_DATA__ JSON '+record['json_path']+'/description whitespace normalized','announcement_id':id,'vendor_announced_date':record['record']['date'],'reference_temporal_status':'Post-cutoff capture of pre-cutoff vendor-dated assertion; immutable historical page equivalence unqualified','scope':'Public reference only; Console implementation and acceptance not established.'}
VIDEO,UNSAVED,RESIZE,LINK=ids
quotes={VIDEO:'After uploading a prerecorded demo, you can review the extracted screenshots in a thumbnail grid, adjust timestamps, delete unwanted images, and confirm your selections before finalizing the documentation.',UNSAVED:'Flow Capture now displays a confirmation dialog when you attempt to navigate away from the edit page with unsaved changes.',RESIZE:'Flow Capture now allows you to resize images and have those changes persist in documentation. Resized images also carry over to Notepad and PDF exports.',LINK:'When clicking a link that would normally open in a new tab (via target="_blank" or cmd/ctrl+click), users are presented with a dialog offering two options: open in a new tab (exiting the recording context) or open in the current tab (continuing the recording).'}
journey='Upload an authorized prerecorded workflow video and review extracted screenshots with stable timestamps thumbnail selection deletion and explicit confirmation before document publication through isolated bounded media processing'
leaf={'id':'F09.capture-video-review','milestone':8,'required':True,'owner':'workflow capture application use cases and canonical media document and policy owners','dependencies':['F09.workflow-capture-resources','F09.capture-asset-editing','F04.video-transform-library','F16.upload-execution-isolation','F16.capture-source-policy'],'journey':journey,'acceptance':{'outcome':journey,'command':None,'fixtures':None,'boundary':None,'status':'not_bound'},'states':{'planned':True,'implemented':False,'integration_accepted':False,'production_qualified':False,'released':False},'evidence':{},'blockers':['Exact owning paths, reviewed design/test candidate and executable acceptance are not yet bound.','Post-cutoff announcement capture retains historical-validity HOLD; required upload formats, extraction fidelity, timestamp editing, policy, bounded resources and recovery need independent qualification.'],'dependency_status':'initial; exact owner/contract dependencies not bound for lane dispatch'}
bindings={leaf['id']:[bind(VIDEO,quotes[VIDEO])],'F09.capture-asset-editing':[bind(RESIZE,quotes[RESIZE]),bind(UNSAVED,quotes[UNSAVED])],'F09.capture-document-export':[bind(RESIZE,quotes[RESIZE])],'F09.workflow-recorder-controls':[bind(LINK,quotes[LINK]),bind(UNSAVED,quotes[UNSAVED])],'F09.documents':[bind(UNSAVED,quotes[UNSAVED])]}
quals=[bind(k,quotes[k])for k in ids]
matrices={'runtimes':[
{'runtime':'Workflow capture prerecorded video review','reference_status':'Vendor dated February9,2026 assertion captured after cutoff; exact historical support unqualified','source':source['id'],'source_bindings':[bind(VIDEO,quotes[VIDEO])],'required_leaf':leaf['id'],'console_acceptance':'not established','command':None,'review_steps':['upload authorized prerecorded demo','review extracted screenshots in thumbnail grid','adjust timestamps','delete unwanted extracted images','confirm selection before finalization'],'supported_formats':'not specified by announcement; no codec/container/browser/size support inferred'},
{'runtime':'Workflow capture image resize persistence','reference_status':'Vendor dated February18,2026 assertion captured after cutoff; exact historical support unqualified','source':source['id'],'source_bindings':[bind(RESIZE,quotes[RESIZE])],'required_leaf':'F09.capture-asset-editing','console_acceptance':'not established','command':None,'document_persistence':True,'specified_export_successors':['native document (Notepad equivalent)','PDF'],'other_export_resize_support':'ZIP and walkthrough behavior not specified by this announcement'},
{'runtime':'Workflow capture navigation preservation','reference_status':'Vendor February14/18,2026 assertions captured after cutoff; exact historical support unqualified','source':source['id'],'source_bindings':[bind(UNSAVED,quotes[UNSAVED]),bind(LINK,quotes[LINK])],'required_leaf':'F09.workflow-recorder-controls','console_acceptance':'not established','command':None,'unsaved_edit_navigation':'warn; offer staying to save','recording_new_tab_triggers':['target=_blank','cmd+click','ctrl+click'],'recording_new_tab_choices':['new tab with explicit recording-context exit','current tab continuing recording']}
]}
limits=[
'This supplement binds four dated Flow Capture release-note behaviors omitted from the three-page current documentation packet. It depends on prior reviewed Flow Capture and Vulcan packets; it appends one planned video-review leaf and does not replace or re-state the single canonical inventory.',
'The exact reused announcements-release-notes source was captured September20 America/New_York after the September19 freeze. February9/14/18,2026 dates are public vendor assertions, not immutable historical page custody. The earlier overview establishes broad classical recording/edit/export scope but does not independently establish these four exact behaviors; historical-validity HOLD remains.',
'Prerecorded-video upload and screenshot extraction are classical media operations. No LLM generation prompts or model-context features are added to this release; independently required transcription retains its existing classical execution contract.',
'Video upload containers/codecs duration size browser support thumbnail frame selection precision variable-frame-rate timestamp meaning and asset limits are not specified in the dated announcement. All require explicit contracts, isolated untrusted processing, scoped credentials, cancellation and exact source-version provenance.',
'Uploaded video and derived screenshots/timestamp selections inherit source and uploader policy. Neither deletion from review nor crop resize blur or removed audio establishes disclosure release; previews exports regeneration retained originals and revocation must obey current authority and legal holds.',
'Unsaved-change and new-tab dialogs are reference recovery behavior, not durability proof. Console must preserve acknowledged draft progress through interrupted saves refresh tab closure lost responses and resume; UI confirmation cannot substitute for the durable owner or silently discard progress.',
'Reference behavior allows deliberately leaving recording context in a new tab. Console must clearly preserve or end the recording state, recheck same-origin/current authority on continued navigation, and not treat captured links or modified click events as command execution authority.',
'Image resize persistence is specifically asserted for documentation Notepad and PDF. ZIP and walkthrough resize behavior is unspecified and must not be inferred. Render dimensions are presentation metadata unless a separately versioned image transform is committed; test export parity without corrupting source pixels or redaction history.',
'All new and existing leaf acceptance states remain unchanged except the added planned-only video-review requirement. Real browser policy owner persistence recovery accessibility and measured workload acceptance remain release blockers; this supplement does not close full Flow Capture or Foundry completeness.'
]
patch=[{'op':'test','path':'/release_inventory/version','value':1},{'op':'test','path':'/release_inventory/reference_freeze/source_bindings/F09.workflow-recorder-controls/0/source_id','value':'flow-capture-record-a-workflow'},{'op':'add','path':'/release_inventory/leaves/-','value':leaf}]
for k,bs in bindings.items():
 path='/release_inventory/reference_freeze/source_bindings/'+k;patch.extend([{'op':'add','path':path+'/-','value':b}for b in bs]if k in inv['reference_freeze']['source_bindings']else[{'op':'add','path':path,'value':bs}])
for q in quals:patch.append({'op':'add','path':'/release_inventory/reference_freeze/runtime_qualifications/-','value':q})
for row in matrices['runtimes']:
 assert not any(e['runtime']==row['runtime']for e in inv['support_matrices']['runtimes']['entries']);patch.append({'op':'add','path':'/release_inventory/support_matrices/runtimes/entries/-','value':row})
for v in limits:patch.append({'op':'add','path':'/release_inventory/reference_freeze/completion_limitations/-','value':v})
pre=copy.deepcopy(reg);apply(reg,patch);graph={x['id']:x['dependencies']for x in reg['release_inventory']['leaves']};assert len(graph)==401;seen=set();active=set()
def visit(k):
 assert k not in active
 if k in seen:return
 active.add(k)
 for d in graph[k]:assert d in graph,(k,d);visit(d)
 active.remove(k);seen.add(k)
for k in graph:visit(k)
assert reg['release_inventory']['leaves'][:400]==pre['release_inventory']['leaves']
proposal={'kind':'bounded-flow-capture-dated-supplement-reference-proposal','observed_head_sha':HEAD,'registry_sha256':sha(rawbase),'prerequisite_packets':prereqs,'prerequisite_leaf_count':400,'reference_date':'2026-09-19','reference_timezone':'America/New_York','retrieval_utc_date':'2026-09-20','exhaustive':False,'sources':[],'sources_reused':[source],'leaves_to_append':[leaf],'source_bindings':bindings,'support_matrix_additions':matrices,'compatibility_and_security_qualifications':quals,'limitations':limits}
dump('proposal.json',proposal);dump('candidate-patch.json',patch);dump('verification.json',{'prerequisite_packets':2,'base_leaf_count':386,'prerequisite_leaf_count':400,'combined_leaf_count':401,'reused_source_count':1,'new_source_count':0,'new_leaf_count':1,'binding_keys':len(bindings),'qualification_count':len(quals),'matrix_rows':3,'quote_membership_checks':quotechecks,'source_size_and_hash_checks':3,'strict_additive_patch_operations':len(patch),'existing_leaf_bytes_preserved_as_values':True,'all_dependencies_resolve':True,'acyclic_dependencies':True,'runtime_tests_executed':0,'implementation_accepted':False})
dump('manifest.json',{p.name:sha(p.read_bytes())for p in sorted(P.iterdir())if p.is_file()and p.name!='manifest.json'})
print((P/'verification.json').read_text());print('proposal_sha256',sha((P/'proposal.json').read_bytes()));print('patch_sha256',sha((P/'candidate-patch.json').read_bytes()));print('manifest_sha256',sha((P/'manifest.json').read_bytes()))
