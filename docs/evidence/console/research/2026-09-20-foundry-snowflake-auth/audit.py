#!/usr/bin/env python3
"""Read captured repository evidence; write only a caller-owned private packet."""
from pathlib import Path
from datetime import datetime
from zoneinfo import ZoneInfo
import gzip,hashlib,json,re,subprocess
ROOT=Path('/private/tmp/console-client-release-20260919')
OUT=Path(__file__).parent
REG=Path('docs/program/console-capability-registry.json')
SOURCE=Path('docs/evidence/console/research/2026-09-19-foundry-vulcan/announcements-release-notes.html.gz')
META=SOURCE.with_suffix('').with_suffix('.json')
def sha(b):return hashlib.sha256(b).hexdigest()
def save(name,value):(OUT/name).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n')
registry=json.loads((ROOT/REG).read_text());inventory=registry['release_inventory']
assert len(inventory['leaves'])==401
assert len(inventory['support_matrices']['connectors']['entries'])==245
indices=[i for i,e in enumerate(inventory['support_matrices']['connectors']['entries']) if e['reference_path']=='/docs/foundry/available-connectors/snowflake/']
assert indices==[180]
row=inventory['support_matrices']['connectors']['entries'][180]
assert set(row)=={'reference_path','status','console_acceptance','command'}
compressed=(ROOT/SOURCE).read_bytes();raw=gzip.decompress(compressed);metadata=json.loads((ROOT/META).read_text())
assert sha(compressed)==metadata['compressed_sha256']
assert sha(raw)==metadata['sha256']
assert len(raw)==metadata['bytes']
registered=[s for s in inventory['reference_freeze']['sources'] if s['id']=='announcements-release-notes']
assert len(registered)==1 and registered[0]['sha256']==sha(raw) and registered[0]['path']==str(SOURCE)
match=re.search(rb'<script[^>]*id="__NEXT_DATA__"[^>]*>(.*?)</script>',raw,re.S)
assert match is not None
page=json.loads(match.group(1));record=page['props']['pageProps']['releases'][60]['releaseNotes'][4]
assert record['id']=='4f3c1dfa766233ff6d3a89cb838c44ce4ccf52cbbce64eb64f950b6f51232b38'
assert record['date']=='June 10, 2026'
assert record['productId']=='Foundry' and record['categoryId']=='data-integration'
assert record['title']=='Snowflake connector now supports external IdP OAuth authentication'
quote='Data Connection now supports authenticating Snowflake connections using an external identity provider (such as Entra ID, Okta, or PingFederate) via the OAuth2 client credentials flow.'
assert record['description'].startswith(quote)
terms=['external IdP','OAuth2 client credentials','programmatic access token']
search={term:[] for term in terms}
def scan(v,path=''):
 if isinstance(v,dict):
  for k,w in v.items():scan(w,path+'/'+k)
 elif isinstance(v,list):
  for i,w in enumerate(v):scan(w,path+'/'+str(i))
 elif isinstance(v,str):
  for term in terms:
   if term.lower() in v.lower():search[term].append(path)
scan(inventory)
assert not search['external IdP'] and not search['OAuth2 client credentials']
local=datetime.fromisoformat(metadata['retrieved_utc']).astimezone(ZoneInfo('America/New_York'))
assert local.isoformat()=='2026-09-20T00:02:15.175438-04:00'
binding={'source_id':'announcements-release-notes','artifact_path':str(SOURCE),'sha256':sha(raw),'compressed_sha256':sha(compressed),'json_path':'/props/pageProps/releases/60/releaseNotes/4','record_id':record['id'],'vendor_asserted_date':record['date'],'quote':record['description'],'matching':'exact JSON description string inside captured __NEXT_DATA__','reference_temporal_status':'post-cutoff capture of vendor-dated June10 assertion; exact September19 support unqualified','scope':'Connector authentication reference only; no mode/version compatibility or Console implementation/acceptance established.'}
save('dated-source-record.json',{'source_binding':binding,'record':record})
save('inventory-gap.json',{'registry_path':str(REG),'registry_sha256':sha((ROOT/REG).read_bytes()),'leaf_count':401,'connector_entry_count':245,'target_json_path':'/release_inventory/support_matrices/connectors/entries/180','current_entry':row,'exact_term_census':search,'gap':'Snowflake external-IdP OAuth2 client-credentials authentication dimension is absent; current entry is only a discovered documentation link.','not_a_new_leaf':'Existing401 leaves remain unchanged; general source/authentication owners already require this support-matrix decomposition.','excluded_inferences':['Connector name alone proves no support','Snowflake mention as external Pipeline Builder engine does not establish connector authentication','Not every IdP that implements OAuth2 is shown compatible','Announcement does not establish data modes/server versions/driver versions or lifecycle phase']})
save('proposal.json',{'kind':'bounded-source-backed-missing-support-matrix-proposal','authority':'HOLD pending root serialized inventory decision and independent review; no lane or implementation authorization','reference_date':'2026-09-19','reference_timezone':'America/New_York','exhaustive':False,'leaves_to_append':[],'existing_leaf_count_unchanged':401,'target_reference_path':row['reference_path'],'proposed_reference_capability':{'reference_status':'vendor-dated June10,2026 authentication support assertion; exact cutoff qualification pending','source':'announcements-release-notes','authentication':{'flow':'OAuth2 client credentials through external identity provider','provider_examples':['Entra ID','Okta','PingFederate'],'documented_configuration_fields':['token endpoint','client ID','client secret','OAuth scopes','user claim'],'vendor_release_version_label':record['releaseVersion'],'lifecycle_phase':'announcement says supports; GA/beta phase not specified','per_mode_version_matrix':'not specified by announcement; no batch/CDC/stream/federation compatibility inferred'},'source_bindings':[binding],'console_acceptance':'not established','command':None},'link_to_existing_work':'Authentication dimension of existing F01 source-connection work; scoped source credentials/egress/current policy obligations already exist in F11.workspace-external-sources and F16.policy. No new capability leaf or completed stage is proposed.','historical_missing_input':{'exact_needed':'An independently retained vendor source or dated compatibility statement binding this authentication behavior and its provider/mode/version limits to 2026-09-19 America/New_York or earlier.','relevant_vendor_page':'https://www.palantir.com/docs/foundry/available-connectors/snowflake/#authentication','cutoff_utc':'2026-09-20T04:00:00+00:00','existing_capture_utc':metadata['retrieved_utc'],'existing_capture_new_york':local.isoformat(),'why_not_satisfied':'Captured vendor announcement asserts June10 support but was retrieved2minutes15seconds after cutoff; Last-Modified and dated prose do not establish immutable cutoff page bytes or detailed compatibility.'},'remaining_unqualified':['source scopes and per-mode support','Snowflake and driver versions','token endpoint TLS/egress and issuer/audience claim constraints','supported scope/user-claim mappings and provider-specific requirements','secret/token rotation and revocation','retry/cancellation/unknown-outcome behavior','Console owning paths, reviewed fixtures and executable acceptance']})
paths=[REG,Path('README.md'),Path('docs/current/PRODUCT.md'),Path('docs/current/ROADMAP.md'),Path('docs/current/DELIVERY.md'),SOURCE,META,Path('docs/evidence/console/research/2026-09-19-foundry-vulcan/manifest.json'),Path('docs/evidence/console/research/2026-09-19-foundry-vulcan/independent-security-review.json')]
save('custody.json',{'observed_head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'sources':{str(p):sha((ROOT/p).read_bytes()) for p in paths},'source_raw_sha256':sha(raw),'source_bytes':len(raw),'source_metadata_custody_verified':True,'registered_source_identity_verified':True,'repository_mutations':False,'new_network_retrievals':False,'build_runtime_database_execution':False})
print('Verified401 existing leaves,245 connector entries, Snowflake row180, raw/compressed custody and exact dated announcement JSON; proposed no new leaf.')
