#!/usr/bin/env python3
"""Exercise output selection against the actual compiled OpenAPI generator."""
import pathlib
import subprocess
import sys
import tempfile


def main():
    binary = pathlib.Path(sys.argv[1]).resolve(strict=True)
    executed = 0
    with tempfile.TemporaryDirectory(prefix="console-openapi-output-") as directory:
        root = pathlib.Path(directory)
        unrelated = root / "unrelated-cwd"
        unrelated.mkdir()
        published = root / "backend/openapi/openapi.yaml"
        codecs = root / "backend/crates/ontology/rest/src/typed_action_generated.rs"
        published.parent.mkdir(parents=True)
        codecs.parent.mkdir(parents=True)
        for args in [
            ["--unsupported", str(root)],
            ["--output-root"],
            ["--output-root", str(root), "extra"],
            ["--output-root", "relative-root"],
            ["--output-root", str(root / "missing")],
        ]:
            published.write_text("preserve-openapi")
            codecs.write_text("preserve-codecs")
            result = subprocess.run([str(binary), *args], capture_output=True, text=True, cwd=unrelated)
            executed += 1
            assert result.returncode != 0, f"invalid output arguments accepted: {args}"
            assert "wrote " not in result.stdout
            assert result.stderr.startswith("console-openapi-gen: output"), result.stderr
            assert published.read_text() == "preserve-openapi"
            assert codecs.read_text() == "preserve-codecs"
        # Both destination parents must be validated before either output write.
        other = root / "partial"
        partial = other / "backend/openapi/openapi.yaml"
        partial.parent.mkdir(parents=True)
        partial.write_text("preserve-partial")
        result = subprocess.run([str(binary), "--output-root", str(other)], capture_output=True, text=True, cwd=unrelated)
        executed += 1
        assert result.returncode != 0
        assert "wrote " not in result.stdout
        assert result.stderr.startswith("console-openapi-gen: output"), result.stderr
        assert partial.read_text() == "preserve-partial"
        result = subprocess.run([str(binary), "--output-root", str(root)], capture_output=True, text=True, cwd=unrelated)
        executed += 1
        assert result.returncode == 0, result.stderr
        spec, code = published.read_text(), codecs.read_text()
        assert "/api/v1/companies/{org_id}/payroll/runs:" in spec
        assert "operationId: listNativeCompanyPayrollRuns" in spec
        assert "nativeAccountSession:" in spec
        assert "fn bind_canonical_action_params" in code
        assert "fn reject_caller_action_key" in code
        assert "fn decode_dispatch_target" in code
        first = (published.read_bytes(), codecs.read_bytes())
        result = subprocess.run([str(binary), "--output-root", str(root)], capture_output=True, text=True, cwd=unrelated)
        executed += 1
        assert result.returncode == 0, result.stderr
        assert first == (published.read_bytes(), codecs.read_bytes()), "same executable inputs must reproduce both outputs"
    print(f"OpenAPI output selection: {executed} executable cases passed")


if __name__ == "__main__":
    main()
