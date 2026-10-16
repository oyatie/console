import copy,gzip,hashlib,json,pathlib,re,subprocess,sys
ROOT=pathlib.Path(__file__).parent
sys.path.insert(0,str(ROOT));from read import read
REPO=pathlib.Path('/private/tmp/console-client-release-20260919')
base=(REPO/'docs/program/console-capability-registry.json').read_bytes();reg=json.loads(base);inv=reg['release_inventory']
source_dir='docs/evidence/console/research/2026-09-19-foundry-datasets/'
limits=['Retrieval occurred on 2026-09-20 UTC, still 2026-09-19 America/New_York. The target freeze date has no declared timezone; these mutable pages do not independently prove availability by 2026-09-19 UTC or historical release status.','Public reference only; no Console implementation, source-freeze completeness, selected dataset format, legal conclusion, or acceptance established.']
sources=[];raw_text={}
for p in sorted(ROOT.glob('*.json')):
 if p.stem in ['proposal','manifest','candidate-patch','verification']:continue
 s=json.loads(p.read_text());s['id']=p.stem;s['path']=source_dir+p.stem+'.html.gz';s['limitations']+=' '+' '.join(limits);sources.append(s)
 raw_text[s['id']]=' '.join(read(ROOT/(p.stem+'.html.gz')).split())
byid={s['id']:s for s in sources}
def binding(sid,quote):
 s=byid[sid];assert ' '.join(quote.split()) in raw_text[sid],(sid,quote)
 return {'source_id':sid,'artifact_path':s['path'],'sha256':s['sha256'],'quote':quote,'matching':'HTML page body text with whitespace normalization','scope':'Public reference only; Console implementation and acceptance not established; retrieval after target UTC date requires freeze-date reconciliation.'}
specs=[
('transaction-lifecycle','Open a dataset transaction, upload files, commit atomically or abort, and reopen its durable outcome after an interrupted response','data-integration-datasets','A transaction can be aborted, which puts it into an ABORTED state. Any files that were written during the transaction are ignored.',['F03.datasets']),
('file-mutation-modes','Apply snapshot, append, update, and logical file deletion with explicit incremental-consumer consequences and preserved prior versions','data-integration-datasets','There are four possible transaction types: SNAPSHOT, APPEND, UPDATE, and DELETE.',['F03.transaction-lifecycle']),
('schema-history','Inspect and evolve versioned schemas, nested and exact decimal types, and expose file/schema discrepancies before downstream use','data-integration-datasets','Because schemas are stored on a dataset view, schemas can change over time.',['F03.datasets']),
('manual-import-inference','Upload tabular or schema-less files, review inferred parsing and column types, and choose append versus replacement with a durable receipt','dataset-preview-overview','In Dataset Preview, you can upload files of the following types directly into a dataset: .csv, .tsv, .xls, .xlsm, and .xlsx.',['F03.schema-history','F03.file-mutation-modes']),
('authorized-preview','Inspect a labeled sample, filter and sort the authorized dataset, inspect column statistics, and report a column issue without disclosing restricted values','dataset-preview-overview','By default, the preview table will show a limited sample of the data; the exact number of rows is displayed in the preview table header.',['F03.datasets','F16.policy']),
('resource-inspection','Reopen dataset metadata, schema, logical files and authorized downloads, build history, health and resource usage with stable resource/version identity','dataset-preview-overview','The Dataset Preview application provides you with a variety of details of a given dataset, including metadata, build history, health, and more.',['F03.datasets','F16.policy']),
('historical-diff','Compare retained dataset versions or branches with schema changes and key-aware added, removed and modified rows, retaining current disclosure checks','dataset-preview-time-travel','A Key column allows Time Travel to match the same row across versions. With a stable, unique key, edits appear as modifications. Without one, changes to an existing row appear as a removal and an addition.',['F03.schema-history','F03.branches']),
('history-bisect','Locate the first retained version where a row or cell changed and preserve the query and loaded-history limits in the result','dataset-preview-time-travel','Bisect only searches across the versions currently loaded on the page.',['F03.historical-diff']),
('rollback-coherence','Restore a selected successful retained dataset version, reconcile every shared-job output and downstream object refresh, and show unchanged transformation logic','data-lineage-dataset-rollback','If multiple outputs are built together by the same job, each output would need to be rolled back to a transaction that was built by the same previous job.',['F03.datasets','F03.lineage']),
('retention-execution','Inspect applicable retention policies and consequences, remove eligible retained versions through the policy owner, and distinguish logical deletion from physical removal','data-integration-datasets','Since a DELETE transaction does not actually remove older data from the backing filesystem, you can use Retention policies to remove data in transactions which are no longer needed.',['F03.file-mutation-modes','F16.policy']),
('stream-archive-view','Inspect a stream as recent rows and archived dataset versions with explicit freshness, checkpoint and delivery semantics, preserving authorized continuous history','data-integration-streams','All data from within a Foundry stream is transferred from the hot buffer to the cold storage every few minutes.',['F03.datasets','F01.stream']),
('stream-reset-recovery','Reset an ingest stream through an authorized consequential command, retain resource references, update source endpoint identity and replay downstream consumers','data-integration-reset-stream','Note that resets are only available for ingest streams. Downstream consuming pipelines of the ingest stream must be replayed after a reset.',['F03.stream-archive-view','F03.lineage']),
('virtual-resource','Register and reopen a governed external-table resource with source locator, schema, supported read/write paths and explicit absence of local version/branch guarantees','data-integration-virtual-tables','A virtual table acts as a pointer to a table in a source system outside of Foundry.',['F03.datasets','F01.federation']),
('virtual-update-detection','Configure external-table change polling, distinguish versioned change detection from snapshot triggers, and expose stale-source and unsupported incremental outcomes','data-integration-virtual-tables','Update detection only controls when a downstream build is triggered. It does not determine whether the resulting build is a snapshot or an incremental build.',['F03.virtual-resource']),
('row-change-log','Apply governed row delete, update and merge changes with an inspectable changelog that downstream incremental readers can resume','iceberg-iceberg-vs-datasets','Row edits: Support for DELETE, UPDATE, and MERGE INTO statements, which allow you to conditionally modify rows without the need to re-snapshot.',['F03.datasets','F03.transaction-lifecycle']),
('table-maintenance','Compact table files and expire snapshots under retention and recovery rules, preserving valid incremental checkpoints and exposing irrecoverable history loss','iceberg-retention','After a snapshot is expired, it no longer appears in the table\'s snapshot history, and you cannot roll back or time travel to that snapshot.',['F03.retention-execution','F03.row-change-log']),
('multi-output-publication','Publish all outputs of one dataset job atomically with stable input versions and recover a failed partial-write attempt without duplicated effects','iceberg-transactions','All-or-nothing commits: All table updates in a job are committed together. If the job fails at any point, no partial writes are visible to downstream consumers or other jobs.',['F03.transaction-lifecycle']),
('branch-metadata-isolation','Evolve schema, partition and sort metadata on an isolated branch and publish through reviewed rebuilding without leaking changes to other branches','iceberg-branching','Foundry\'s Iceberg catalog extends standard Iceberg so that each branch can independently track its own current schema, default partition spec, and default sort order.',['F03.branches','F03.schema-history']),
('storage-locations','Inspect and govern managed or externally owned storage locations and encryption settings by resource scope while preserving dataset identity, recovery and current access','iceberg-storage','Foundry-managed Iceberg tables can either use Foundry-managed storage or BYOB storage, or a combination, where different projects or namespaces target different storage locations.',['F03.datasets','F16.policy'])]
leaves=[];bindings={}
for suffix,journey,sid,quote,deps in specs:
 id='F03.'+suffix
 leaves.append({'id':id,'milestone':6,'required':True,'owner':'data resources','dependencies':deps,'journey':journey,'acceptance':{'outcome':journey,'command':None,'fixtures':None,'boundary':None,'status':'not_bound'},'states':{'planned':True,'implemented':False,'integration_accepted':False,'production_qualified':False,'released':False},'evidence':{},'blockers':['Exact owning paths, reviewed design/test candidate and executable acceptance are not yet bound.','Reference capability availability at the target UTC date is not independently established; source retrieval date/timezone must be reconciled.'],'dependency_status':'initial; exact owner/contract dependencies not bound for lane dispatch'})
 bindings[id]=[binding(sid,quote)]
bindings['F03.datasets']=[binding('data-integration-datasets','The benefit of using Foundry datasets is that they provide integrated support for permission management, schema management, version control, and updates over time.')]
bindings['F03.branches']=[binding('data-integration-branching','Unlike Git, there is no support for merging dataset branches.')]
qs=[
('data-integration-datasets','An APPEND transaction cannot modify existing files in the current dataset view.'),
('data-integration-datasets','Note that committing a DELETE transaction does not delete the underlying file from the backing file system—it simply removes the file reference from the dataset view.'),
('data-integration-datasets','Viewing dataset retention policies is in the beta phase of development and may not be available on your enrollment.'),
('data-integration-branching','One open transaction per branch. Every branch can have at most one open (as in, opened and neither committed or aborted) transaction, and this transaction is always the latest transaction on the branch.'),
('dataset-preview-time-travel','Time Travel is in the beta phase of development.'),
('dataset-preview-time-travel','You can only inspect versions where data is still available under the applicable retention policy.'),
('data-lineage-dataset-rollback','After a rollback is carried out, the logic backing the dataset will be left unchanged and will need to be updated in order to apply to the next build.'),
('data-lineage-dataset-rollback','If the branch on which a rollback is being performed does not exist on the dataset, the rollback will be applied to a fallback branch.'),
('data-integration-streams','Streaming sources in Foundry currently only support AT_LEAST_ONCE semantics for extracts and exports.'),
('data-integration-streams','When EXACTLY_ONCE is enabled, records are only visible downstream after each checkpoint has completed (default is two seconds).'),
('data-integration-reset-stream','Resetting streams can have irreversible effects on data.'),
('data-integration-virtual-tables','Virtual tables do not benefit from Foundry dataset capabilities such as dataset versioning or branching.'),
('data-integration-virtual-tables','Virtual tables require a Foundry worker source and direct egress policies.'),
('data-integration-virtual-tables','Incremental support for virtual tables is currently limited to append-only changes and has additional source-specific prerequisites, such as enabling Delta\'s Change Data Feed ↗ or Iceberg\'s Incremental Reads ↗.'),
('iceberg-iceberg-vs-datasets','At the table level: Iceberg tables cannot directly back restricted views.'),
('iceberg-iceberg-vs-datasets','A marking inherited by any retained Iceberg snapshot can prevent users who do not satisfy that marking from reading the entire table, including otherwise unmarked branches.'),
('iceberg-transactions','The transaction guarantees described on this page apply only when running jobs through Foundry\'s build system.'),
('iceberg-transactions','Jobs writing to the same output are queued and run sequentially rather than concurrently.'),
('iceberg-branching','Branch isolation is available from transforms-tables library version 0.1211.0 and greater.'),
('iceberg-branching','Iceberg\'s native branch merge procedures, such as cherry-pick and fast-forward, operate on snapshot references only and are not aware of Foundry\'s branch-scoped schema properties.'),
('iceberg-retention','Remove orphan files in Foundry only deletes orphan files that are older than both the user-supplied older_than timestamp and the configured disaster recovery lookback window on your environment.'),
('iceberg-storage','Server-side encryption (SSE) is mandatory for all tables.'),
('iceberg-storage','Client-side Iceberg table encryption is a new and evolving capability that is not yet supported by all Foundry features, external compute engines, or tools that connect to Iceberg tables.')]
qualifications=[binding(s,q)for s,q in qs]
existing={x['id']for x in inv['leaves']};assert not existing.intersection(x['id']for x in leaves)
allids=existing|{x['id']for x in leaves};assert all(d in allids for x in leaves for d in x['dependencies'])
assert not set(byid).intersection(s['id']for s in inv['reference_freeze']['sources'])
patch=[{'op':'test','path':'/release_inventory/version','value':1}]
patch += [{'op':'add','path':'/release_inventory/reference_freeze/sources/-','value':x}for x in sources]
patch += [{'op':'add','path':'/release_inventory/leaves/-','value':x}for x in leaves]
for k,v in bindings.items():
 if k in inv['reference_freeze']['source_bindings']:
  patch += [{'op':'add','path':f'/release_inventory/reference_freeze/source_bindings/{k}/-','value':b}for b in v]
 else:patch.append({'op':'add','path':f'/release_inventory/reference_freeze/source_bindings/{k}','value':v})
for b in qualifications:patch.append({'op':'add','path':'/release_inventory/reference_freeze/runtime_qualifications/-','value':b})
for s in limits:patch.append({'op':'add','path':'/release_inventory/reference_freeze/completion_limitations/-','value':s})
proposal={'kind':'bounded-F03-operational-dataset-reference-proposal','observed_head_sha':subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPO,text=True).strip(),'registry_sha256':hashlib.sha256(base).hexdigest(),'reference_date':'2026-09-19','retrieval_utc_date':'2026-09-20','target_date_timezone_unresolved':True,'exhaustive':False,'sources':sources,'leaves_to_append':leaves,'source_bindings':bindings,'compatibility_and_lifecycle_qualifications':qualifications,'limitations':limits+['No new storage format chosen: PostgreSQL remains authoritative; compare formats against measured correctness/workloads before adopting Iceberg or any other format.','Existing F03.branches user-required reconciliation remains required; vendor dataset branch non-merge behavior does not waive it.','Public HTML quote membership is checked after page-body whitespace normalization; this is not independent entailment or historical authenticity proof.','Virtual-table source/table/engine matrices contain nuanced and potentially inconsistent cells; source-specific compatibility is not exhaustively resolved.','Source snapshots alone do not satisfy secure current-policy previews, downloads, corrections, retention/legal holds, browser completion, performance or recovery.']}
(ROOT/'proposal.json').write_text(json.dumps(proposal,ensure_ascii=False,indent=2)+'\n')
(ROOT/'candidate-patch.json').write_text(json.dumps(patch,ensure_ascii=False,indent=2)+'\n')
verification={'source_count':len(sources),'new_leaf_count':len(leaves),'bound_leaf_keys':len(bindings),'qualification_count':len(qualifications),'quote_membership_checks':sum(map(len,bindings.values()))+len(qualifications),'all_new_dependencies_resolve':True,'duplicate_ids':False,'implementation_accepted':False,'runtime_tests_executed':0}
(ROOT/'verification.json').write_text(json.dumps(verification,indent=2)+'\n')
manifest={p.name:hashlib.sha256(p.read_bytes()).hexdigest()for p in sorted(ROOT.iterdir())if p.is_file()and p.name!='manifest.json'}
(ROOT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(verification));print('proposal_sha256',manifest['proposal.json']);print('patch_sha256',manifest['candidate-patch.json'])
