#!/usr/bin/env python3
"""Focused tests for production audit filtering and failure handling."""

import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location(
    "audit_production", Path(__file__).with_name("audit-production.py")
)
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)

wrapper_spec = importlib.util.spec_from_file_location(
    "audit_native_rustc", Path(__file__).with_name("audit-native-rustc.py")
)
wrapper = importlib.util.module_from_spec(wrapper_spec)
wrapper_spec.loader.exec_module(wrapper)


class NativeWrapperTest(unittest.TestCase):
    def test_wasm_bypasses_instrumentation_but_preserves_compiler_arguments(self):
        for target in [
            ["--target", "wasm32v1-none"],
            ["--target=wasm32-unknown-unknown"],
            ["--target", "wasm64-unknown-unknown"],
        ]:
            with self.subTest(target=target):
                args = ["/tools/cargo-auditable", "/tools/rustc", "input.rs", *target]
                env = {"RUSTC_WORKSPACE_WRAPPER": args[0]}
                self.assertEqual(wrapper.compiler_command(args, env), args[1:])

    def test_native_targets_and_host_queries_retain_instrumentation(self):
        for target in [[], ["-vV"], ["--target", "x86_64-unknown-linux-gnu"],
                       ["--target=aarch64-apple-darwin"]]:
            with self.subTest(target=target):
                args = ["/tools/cargo-auditable", "/tools/rustc", *target]
                env = {"RUSTC_WORKSPACE_WRAPPER": args[0]}
                self.assertEqual(wrapper.compiler_command(args, env), args)

    def test_dependency_compiler_and_unrelated_workspace_wrapper_are_preserved(self):
        for prefix in [["/tools/rustc"], ["/tools/other-wrapper", "/tools/rustc"]]:
            args = [*prefix, "--target", "wasm32v1-none"]
            env = {"RUSTC_WORKSPACE_WRAPPER": "/tools/other-wrapper"}
            self.assertEqual(wrapper.compiler_command(args, env), args)

    def test_existing_outer_wrapper_is_forwarded_for_native_and_wasm(self):
        for target in ["wasm32v1-none", "x86_64-unknown-linux-gnu"]:
            args = ["/tools/cargo-auditable", "/tools/rustc", "--target", target]
            env = {
                "RUSTC_WORKSPACE_WRAPPER": args[0],
                "DUNITER_AUDIT_ORIGINAL_RUSTC_WRAPPER": "/tools/sccache",
            }
            expected = args[1:] if target.startswith("wasm") else args
            self.assertEqual(wrapper.compiler_command(args, env), ["/tools/sccache", *expected])


class ProductionAuditTest(unittest.TestCase):
    def run_audit(self, *, missing_metadata=False, audit_error=False, changed_lock=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "resources").mkdir()
            (root / "resources/g1_metadata.scale").write_bytes(b"fixture metadata")
            (root / ".cargo").mkdir()
            policy = b'[advisories]\nignore = []\n'
            (root / ".cargo/audit.toml").write_bytes(policy)
            packages = [
                {"id": name, "name": name, "version": "1.0.0", "source": None}
                for name in (*audit.BINARIES, "compiled", "inactive")
            ]
            lock = "version = 4\n" + "".join(
                f'\n[[package]]\nname = "{p["name"]}"\nversion = "1.0.0"\n'
                for p in packages
            )
            (root / "Cargo.lock").write_text(lock)

            def check_output(command, **kwargs):
                if command[0] == "rustc":
                    return "host: aarch64-apple-darwin\n"
                if command[1] == "metadata":
                    return json.dumps({"packages": packages})
                binary = Path(command[1]).name
                entries = [] if missing_metadata else [
                    {"name": binary, "version": "1.0.0", "root": True},
                    {"name": "compiled", "version": "1.0.0"},
                    {"name": "inactive", "version": "1.0.0"},
                ]
                return json.dumps({"packages": entries})

            def build(command, stdout, **kwargs):
                self.assertEqual(
                    kwargs["env"]["RUSTC_WRAPPER"], str(root / "scripts/audit-native-rustc.py")
                )
                self.assertIn("--release", command)
                self.assertIn("--locked", command)
                self.assertIn("--no-default-features", command)
                binary = command[command.index("--bin") + 1]
                features = command[command.index("--features") + 1]
                self.assertEqual(
                    features,
                    "g1,embed" if binary == "duniter" else "g1,standalone,std",
                )
                if changed_lock:
                    (root / "Cargo.lock").write_text(lock + "# concurrent edit\n")
                executable = root / binary
                executable.write_bytes(b"fixture")
                for name in (binary, "compiled"):
                    stdout.write(json.dumps({
                        "reason": "compiler-artifact", "package_id": name,
                        "target": {"name": name}, "fresh": True,
                        "executable": str(executable) if name == binary else None,
                    }) + "\n")

            def scan(command, report, **kwargs):
                if "--json" in command:
                    result = {} if audit_error else {"vulnerabilities": {"list": []}}
                    report.write_text(json.dumps(result))
                else:
                    report.write_text("fixture report\n")
                return 1

            previous = Path.cwd()
            try:
                with (
                    patch.object(audit, "ROOT", root),
                    patch.object(audit.shutil, "which", return_value="tool"),
                    patch.object(audit.subprocess, "check_output", side_effect=check_output),
                    patch.object(audit.subprocess, "run", side_effect=build),
                    patch.object(audit, "audit", side_effect=scan),
                    patch.dict(os.environ, {"SKIP_WASM_BUILD": ""}),
                    patch("sys.argv", ["audit-production.py"]),
                    contextlib.redirect_stdout(io.StringIO()),
                ):
                    status = audit.main()
                report_dir = next((root / "target/audit/production").iterdir())
                self.assertEqual((report_dir / "audit.toml").read_bytes(), policy)
                active = (report_dir / "compiled.Cargo.lock").read_text()
                self.assertIn('name = "compiled"', active)
                self.assertNotIn('name = "inactive"', active)
                self.assertEqual((root / "Cargo.lock").read_text(), lock)
                compiled = json.loads((report_dir / "compiled-packages.json").read_text())
                shared = next(p for p in compiled if p["name"] == "compiled")
                self.assertEqual(shared["binaries"], sorted(audit.BINARIES))
                surplus = json.loads((report_dir / "duniter.metadata-surplus.json").read_text())
                self.assertEqual([p["name"] for p in surplus], ["inactive"])
                return status
            finally:
                os.chdir(previous)

    def test_cached_artifacts_and_inactive_metadata(self):
        self.assertEqual(self.run_audit(), 1)

    def test_missing_embedded_metadata_is_an_error(self):
        with self.assertRaisesRegex(RuntimeError, "Missing or incorrect"):
            self.run_audit(missing_metadata=True)

    def test_audit_error_is_not_a_vulnerability_result(self):
        with self.assertRaisesRegex(RuntimeError, "complete vulnerability report"):
            self.run_audit(audit_error=True)

    def test_concurrent_lockfile_change_is_rejected(self):
        with self.assertRaisesRegex(RuntimeError, "Cargo.lock changed"):
            self.run_audit(changed_lock=True)


if __name__ == "__main__":
    unittest.main()
