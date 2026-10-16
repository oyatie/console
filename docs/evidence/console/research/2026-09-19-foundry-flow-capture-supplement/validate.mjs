import fs from 'node:fs';
import crypto from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {validateConsoleTruthLedger} from '/private/tmp/console-client-release-20260919/scripts/console/validate-console-truth-ledger.mjs';
const root='/private/tmp/console-foundry-flow-capture-supplement-20260919';
const repo='/private/tmp/console-client-release-20260919';
const raw=execFileSync('git',['show','c659210e37f3f0fa5b8dc7d3b9ed7b8e07b7d139:docs/program/console-capability-registry.json'],{cwd:repo,maxBuffer:8*1024*1024});
const registry=JSON.parse(raw), proposal=JSON.parse(fs.readFileSync(root+'/proposal.json'));
if(crypto.createHash('sha256').update(raw).digest('hex')!==proposal.registry_sha256)throw Error('registry changed; rebase proposal before acceptance');
const patch=[...proposal.prerequisite_packets.flatMap(p=>JSON.parse(fs.readFileSync(p.packet+'/candidate-patch.json'))),...JSON.parse(fs.readFileSync(root+'/candidate-patch.json'))];
for(const p of patch){
 const parts=p.path.slice(1).split('/').map(p=>p.replaceAll('~1','/').replaceAll('~0','~'));
 const key=parts.pop();let at=registry;for(const part of parts)at=at[part];
 if(p.op==='test'){if(JSON.stringify(at[key])!==JSON.stringify(p.value))throw Error('patch test failed');}
 else if(p.op==='add'){if(key==='-')at.push(p.value);else at[key]=p.value;}
 else throw Error('unsupported op');
}
const jurisdiction=JSON.parse(fs.readFileSync(repo+'/docs/program/console-jurisdiction-register.json'));
const result=validateConsoleTruthLedger(registry,jurisdiction,{expectedCandidateSha:proposal.observed_head_sha});
const verification=JSON.parse(fs.readFileSync(root+'/verification.json'));
verification.patch_applied=true;verification.structural_validator_control=result;verification.structural_validator_caveat='Default permissive resolvers exercise schema and reference consistency only; not tracked-source, signed candidate train, implementation, runtime or production acceptance.';
console.log(JSON.stringify(verification));
