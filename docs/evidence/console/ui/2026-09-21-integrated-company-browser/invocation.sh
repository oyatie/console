#!/usr/bin/env bash
# Root executes only while holding the app-build/PostgreSQL lease.
set -euo pipefail
umask 077
export PATH="/var/folders/66/4qlvtbgn6r1gl9bvfttp6sww0000gn/T/console-dotslash/bin:$PATH"
task_repo=/private/tmp/console-mvp-dev-integrate-20260921
retained_runtime=/private/tmp/console-native-browser-local-20260920-l1dn9ar1/runtime
cd "$task_repo"
[[ "$(git rev-parse HEAD)" == 9f2fb85307b28911d4bdc17813496df5b195f26f ]]
browser_stage="$(mktemp -d /private/tmp/console-integrated-company-browser-20260921.XXXXXX)"
printf '%s\n' "$browser_stage"
git show 019b32db4b3076122672e9f3eb938ff872a45bfd:tools/browser/company.cjs > "$browser_stage/company.cjs"
ln -s "$retained_runtime" "$browser_stage/runtime"
python3 - "$browser_stage" <<'CHECK'
import hashlib,json,sys
from pathlib import Path
stage=Path(sys.argv[1]);rt=stage/'runtime'
pins={'company.cjs':'9edab0630df042767ccd4da092b79d6b6f34b566f1bc1c2d9dd5df493187f7cd','runtime/package.json':'8d57d95d41a1c2833b846f382610db55b8d193c20e3b2473c1e59f43e798a9b9','runtime/package-lock.json':'a9c22966fb530b30d45f4f17faca408679a0405f3978fdaa9abd6b1857578044','runtime/node_modules/playwright-core/browsers.json':'545d52f8382c391e605562c330e9c1c534a16045898203037a49bb8bd769a946','runtime/browser/chrome-headless-shell-mac-arm64/chrome-headless-shell':'a0bfe7b4da4787b66058477d696cd1d09065d25f06a548947722b9af77ee8282'}
for name,expected in pins.items():
 assert hashlib.sha256((stage/name).read_bytes()).hexdigest()==expected,name
for name in ['playwright','playwright-core']:
 assert json.loads((rt/'node_modules'/name/'package.json').read_text())['version']=='1.63.0'
assert not (stage/'company.cjs').is_symlink()
assert not (stage/'company').exists()
(stage/'stage-receipt.json').write_text(json.dumps({'candidate':'9f2fb85307b28911d4bdc17813496df5b195f26f','runtime':str(rt.resolve()),'pins':pins,'status':'STAGED_NOT_EXECUTED','candidate_scope':'working source snapshot captured separately; base SHA alone is not candidate proof','driver_source_commit':'019b32db4b3076122672e9f3eb938ff872a45bfd'},indent=2)+'\n')
CHECK
# The harness omits flags from its build pass; constrain both passes through
# its existing public Buck-binary selector; the test pass remains serial.
cat > "$browser_stage/buck2-bounded" <<'BUCK'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == build ]]; then
  shift
  exec /private/tmp/console-mvp-dev-integrate-20260921/tools/buck2 build -j2 -c root//project.ignore=.git,backend/target "$@"
fi
if [[ "$1" == test ]]; then
  shift
  exec /private/tmp/console-mvp-dev-integrate-20260921/tools/buck2 test -c root//project.ignore=.git,backend/target "$@" --env "PATH=$PATH"
fi
exec /private/tmp/console-mvp-dev-integrate-20260921/tools/buck2 "$@"
BUCK
chmod 700 "$browser_stage/buck2-bounded"
export CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="$browser_stage/buck2-bounded"
export CONSOLE_COMPANY_BROWSER_DRIVER="$browser_stage/company.cjs"
export CONSOLE_COMPANY_BROWSER_SHA256=9edab0630df042767ccd4da092b79d6b6f34b566f1bc1c2d9dd5df493187f7cd
export CONSOLE_COMPANY_BROWSER_OUTPUT="$browser_stage/company"
export CONSOLE_BUCK_NEEDS_POSTGRES_TEST_EXACT=account_browser::deployment_operator_designation::company_setup::native_company_real_browser_create_reopen_and_workspace
unset DEBUG PWDEBUG NODE_DEBUG
bash tools/buck/test_needs_postgres.sh --num-threads=1 //tools/buck:app-auth-rest-browser-pg > "$browser_stage/run.log" 2>&1
