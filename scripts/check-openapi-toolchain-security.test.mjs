import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";

import { readFileFromUrl } from "@redocly/openapi-core";

const rootPackage = JSON.parse(
  readFileSync(new URL("../package.json", import.meta.url), "utf8"),
);
const lockfile = JSON.parse(
  readFileSync(new URL("../package-lock.json", import.meta.url), "utf8"),
);

test("pins the audited callable minimatch adapter and brace-expansion repair", () => {
  assert.deepEqual(rootPackage.overrides?.["@redocly/openapi-core@1.34.17"], {
    minimatch: "file:../../../tools/npm/minimatch-callable-compat",
  });
  assert.equal(rootPackage.overrides?.["brace-expansion"], "5.0.8");

  assert.deepEqual(
    lockfile.packages[
      "node_modules/@redocly/openapi-core/node_modules/minimatch"
    ],
    {
      resolved: "tools/npm/minimatch-callable-compat",
      link: true,
    },
  );
  assert.equal(
    lockfile.packages["tools/npm/minimatch-callable-compat"]?.version,
    "5.1.9-maintenance.1",
  );
  assert.equal(
    lockfile.packages["tools/npm/minimatch-callable-compat"]?.dependencies?.[
      "minimatch-modern"
    ],
    "npm:minimatch@10.2.5",
  );
  assert.equal(
    lockfile.packages["node_modules/minimatch-modern"]?.version,
    "10.2.5",
  );
  assert.equal(
    lockfile.packages["node_modules/brace-expansion"]?.version,
    "5.0.8",
  );
});

test("preserves Redocly's callable CommonJS minimatch contract", () => {
  const requireFromRoot = createRequire(import.meta.url);
  const requireFromRedocly = createRequire(
    requireFromRoot.resolve("@redocly/openapi-core/package.json"),
  );
  const minimatch = requireFromRedocly("minimatch");

  assert.equal(typeof minimatch, "function");
  assert.equal(
    minimatch(
      "example.test/openapi.yaml",
      "example.test/{openapi,schema}.yaml",
    ),
    true,
  );
  assert.equal(
    minimatch("example.test/other.yaml", "example.test/{openapi,schema}.yaml"),
    false,
  );
});

test("applies matching Redocly HTTP headers through the real URL-loading path", async () => {
  let observedRequest;
  const result = await readFileFromUrl("https://example.test/openapi.yaml", {
    headers: [
      {
        matches: "example.test/{openapi,schema}.yaml",
        name: "x-maintenance-contract",
        value: "verified",
      },
    ],
    customFetch: async (url, init) => {
      observedRequest = { url, headers: init.headers };
      return new Response("openapi: 3.1.0\n", {
        status: 200,
        headers: { "content-type": "application/yaml" },
      });
    },
  });

  assert.deepEqual(observedRequest, {
    url: "https://example.test/openapi.yaml",
    headers: { "x-maintenance-contract": "verified" },
  });
  assert.deepEqual(result, {
    body: "openapi: 3.1.0\n",
    mimeType: "application/yaml",
  });
});
