#!/usr/bin/env bash
# Test-tool bootstrap only. Product and test binaries are built by Buck2.
set -euo pipefail
umask 077

fail() { echo "native browser: $1" >&2; exit 1; }
for tool in npm node curl unzip openssl uname; do
  command -v "${tool}" >/dev/null || fail "required bootstrap tool missing"
done
[[ "$(uname -s)" == Linux && "$(uname -m)" == x86_64 ]] || fail "unsupported reviewed platform"
[[ -n "${RUNNER_TEMP:-}" && -d "${RUNNER_TEMP}" && "${RUNNER_TEMP}" == /* ]] || fail "absolute runner temporary directory required"
[[ -n "${GITHUB_ENV:-}" && -f "${GITHUB_ENV}" && -w "${GITHUB_ENV}" ]] || fail "writable GITHUB_ENV required"
case "${RUNNER_TEMP}${GITHUB_ENV}" in
  *$'\n'*|*$'\r'*) fail "newline in output path" ;;
esac
node -e 'if (Number(process.versions.node.split(".")[0]) < 18) process.exit(1)' || fail "Node 18 or newer required"

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source_dir="${repo_root}/docs/evidence/console/integration/2026-09-19-root-entry-browser"
stage="$(mktemp -d "${RUNNER_TEMP}/console-native-browser.XXXXXX")"
chmod 700 "${stage}"
mkdir "${stage}/runtime" "${stage}/outputs" "${stage}/runtime/browser"

verify_sha() {
  local actual
  actual="$(openssl dgst -sha256 "$1")" || fail "required pinned file missing"
  [[ "${actual##* }" == "$2" ]] || fail "pinned file digest mismatch"
}
cp "${source_dir}/playwright-package.json" "${stage}/runtime/package.json"
cp "${source_dir}/playwright-package-lock.json" "${stage}/runtime/package-lock.json"
verify_sha "${stage}/runtime/package.json" 8d57d95d41a1c2833b846f382610db55b8d193c20e3b2473c1e59f43e798a9b9
verify_sha "${stage}/runtime/package-lock.json" a9c22966fb530b30d45f4f17faca408679a0405f3978fdaa9abd6b1857578044
for entry in \
  account:3e59f4f63cce565fee6cd7da94c4f6dc81bba05d7a4e6b309ecc2e13f88c161d \
  company:10edae6d0f4d6f66ccf0cbe2d17eaba8cbbc977ede47b4b5fc94b3a201f7fcc2 \
  company-preview:a1c4c3ad5b1cf5a5c01c86795b4e0db10d3b6c233a7a7d2f84065735ea919d90 \
  hydration:fcb0d0b95981833017d47f2723459879478640e1faed8f4065cac1a0d6a5ac1c \
  native_header:915f37ffb3b7580202158ca23425e047e4587278aaaa5725687f9a6eb71a34c4 \
  people_journey:051ccf6be53899a9b077df697e4fea36d7edacc38e8b8e2065ca37b5c52e098b \
  react_people_controls:04990da9c37c41e8b478d118effeb44a1c6f805a408fddb455039404bd32c97c \
  group_process_journey:a4764161086e35b974c94bf96a7ed9d4411b940a00d1a3d62589c1b229c8ba7f \
  group_invalid_form_journey:f301776ce4bcb9c7b5101baa4f4cab8c27c5dc633e277d17db2a89939b46a6dd \
  group_navigation_held:0c8b4d46a5ca248df5a5b08c0a1bf0da6c71b59049516e683389375d51794ca2 \
  policy_journey:766da87a7ad3c7c6c41dd98725ddbda951477d1e7b43e5961e213d92c4d5f4ea \
  recovery_controls:fe3afc43196cc034d6a5f9bd0b12ad787eaeddf2cb1b1b837b595cb9f49036c6 \
  account-company-handoff:ee8e799bd55024e2274bde98195d6a03ab564cc5b063cb70d8f445b593a3c92d \
  account_company_handoff:68d5e45686d19088cb3690b1703dc8ad83f8ea1a340935ca0ff1f473c72f0b78 \
  account-controller:c91cbff5a96176fb67f34753483a52c4960c9552fd6677d476906ef98d9aa67a \
  account_controller_controls:540382452084a0e40205c78ba502bf9360a2ec647a0aeb30425e20bbc4d1b8dc \
  account_controller_evidence:50e37ea2b53fdf8aa6b4f649e43ed0500946cbbcffaff139914d516ff0485341; do
  cp "${repo_root}/tools/browser/${entry%%:*}.cjs" "${stage}/${entry%%:*}.cjs"
  verify_sha "${stage}/${entry%%:*}.cjs" "${entry#*:}"
done

npm --prefix "${stage}/runtime" ci --ignore-scripts --no-audit --no-fund
verify_sha "${stage}/runtime/node_modules/playwright-core/browsers.json" 545d52f8382c391e605562c330e9c1c534a16045898203037a49bb8bd769a946
node "${stage}/runtime/node_modules/playwright/cli.js" install-deps chromium-headless-shell
curl --fail --location --silent --show-error \
  https://cdn.playwright.dev/builds/cft/153.0.8010.12/linux64/chrome-headless-shell-linux64.zip \
  --output "${stage}/browser.zip"
verify_sha "${stage}/browser.zip" a9da028861a0cf789ff25c2fed45f5f1aaf969ed9247835b6a7821a4f7af9d1d
unzip -q "${stage}/browser.zip" -d "${stage}/runtime/browser"
browser="${stage}/runtime/browser/chrome-headless-shell-linux64/chrome-headless-shell"
verify_sha "${browser}" ded93a9c9a53a1ae040f08124badcca95c938e9d5015ff340c3b5538c41bf39e
chmod 700 "${browser}"

# Publish only after every prerequisite check; no environment or credentials enter
# the receipt. These version/digest facts are not browser execution evidence.
node - "${stage}" "${GITHUB_ENV}" "$(npm --version)" "$(openssl version)" <<'NODE'
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const [stage, envFile, npmVersion, opensslVersion] = process.argv.slice(2);
const hash = name => crypto.createHash('sha256').update(fs.readFileSync(path.join(stage, name))).digest('hex');
const pkg = JSON.parse(fs.readFileSync(path.join(stage, 'runtime/node_modules/playwright/package.json')));
const core = JSON.parse(fs.readFileSync(path.join(stage, 'runtime/node_modules/playwright-core/package.json')));
if (pkg.version !== '1.63.0' || core.version !== '1.63.0') throw Error('pinned package version mismatch');
const drivers = {
  CONSOLE_BROWSER_JOURNEY: 'account',
  CONSOLE_COMPANY_BROWSER: 'company',
  CONSOLE_COMPANY_PREVIEW_BROWSER: 'company-preview',
  CONSOLE_HYDRATION_BROWSER: 'hydration',
};
const companions = ['native_header', 'people_journey', 'react_people_controls', 'group_process_journey', 'group_invalid_form_journey', 'group_navigation_held', 'policy_journey', 'recovery_controls', 'account-company-handoff', 'account_company_handoff', 'account-controller', 'account_controller_controls', 'account_controller_evidence'];
const digests = {
  package: hash('runtime/package.json'), lock: hash('runtime/package-lock.json'),
  browsers: hash('runtime/node_modules/playwright-core/browsers.json'),
  archive: hash('browser.zip'),
  executable: hash('runtime/browser/chrome-headless-shell-linux64/chrome-headless-shell'),
  drivers: Object.fromEntries(Object.values(drivers).map(name => [name + '.cjs', hash(name + '.cjs')])),
  companions: Object.fromEntries(companions.map(name => [name + '.cjs', hash(name + '.cjs')])),
};
fs.writeFileSync(path.join(stage, 'prerequisites.json'), JSON.stringify({
  tools: {node: process.version, npm: npmVersion, openssl: opensslVersion},
  versions: {playwright: pkg.version, chromium: '153.0.8010.12', revision: '1243'},
  digests, status: 'PREREQUISITES_STAGED_BROWSER_EXECUTION_PENDING',
}, null, 2) + '\n', {flag: 'wx', mode: 0o600});
const settings = Object.entries(drivers).flatMap(([prefix, name]) => [
  prefix + '_DRIVER=' + path.join(stage, name + '.cjs'),
  prefix + '_SHA256=' + digests.drivers[name + '.cjs'],
  prefix + '_OUTPUT=' + path.join(stage, 'outputs', name),
]);
fs.appendFileSync(envFile, settings.join('\n') + '\n');
NODE
