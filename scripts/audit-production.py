#!/usr/bin/env python3
"""Build and audit the production executables with embedded dependency data."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib


ROOT = Path(__file__).resolve().parent.parent
BINARIES = ("duniter", "distance-oracle")
CARGO = str(ROOT / "scripts" / "cargo_with_vendor.sh")


def audit(command, report, *, json_output=False):
    with report.open("w") as output:
        process = subprocess.Popen(
            command, stdout=subprocess.PIPE,
            stderr=None if json_output else subprocess.STDOUT, text=True
        )
        for line in process.stdout:
            if not json_output:
                print(line, end="", flush=True)
            output.write(line)
        return process.wait()


def main():
    parser = argparse.ArgumentParser(
        description=__doc__,
        epilog=(
            "First prepare the SDK and chain metadata; see scripts/README.md. "
            "Prerequisites: cargo install --locked cargo-auditable cargo-audit "
            "rust-audit-info. Cross-compilation requires the Rust target and linker. "
            "Uses the repository toolchain, Docker release features and Cargo.lock. "
            "Requires Python 3.11+. Audits embedded metadata and cross-checks it against "
            "Cargo build artifacts, including build dependencies. Returns 1 for "
            "vulnerabilities, unsound or yanked compiled dependencies, 2 for audit errors. "
            "Unmaintained dependencies remain warnings. Policy: .cargo/audit.toml."
        ),
    )
    parser.add_argument(
        "--target", help="Rust target triple; defaults to the compiler's host target"
    )
    parser.add_argument(
        "--chain", choices=("g1", "gtest", "gdev"), default="g1",
        help="release chain features; defaults to g1",
    )
    args = parser.parse_args()
    if os.environ.get("SKIP_WASM_BUILD"):
        parser.error("unset SKIP_WASM_BUILD to audit a complete production build")
    os.chdir(ROOT)
    metadata_path = ROOT / "resources" / f"{args.chain}_metadata.scale"
    if not metadata_path.is_file() or not metadata_path.stat().st_size:
        parser.error(
            f"missing resources/{args.chain}_metadata.scale; run "
            f"METADATA_CHAINS={args.chain} ./scripts/generate_all_metadata.sh"
        )
    for tool in ("cargo", "rustc", "cargo-auditable", "cargo-audit", "rust-audit-info"):
        if not shutil.which(tool):
            parser.error(f"missing {tool}; see --help for installation instructions")

    rustc = subprocess.check_output(["rustc", "-vV"], text=True)
    target = args.target or next(
        line.removeprefix("host: ") for line in rustc.splitlines()
        if line.startswith("host: ")
    )
    reports = ROOT / "target" / "audit" / "production"
    reports.mkdir(parents=True, exist_ok=True)
    report_dir = Path(tempfile.mkdtemp(prefix="run-", dir=reports))
    (report_dir / "rustc.txt").write_text(rustc)
    (report_dir / "target.txt").write_text(target + "\n")
    (report_dir / "chain.txt").write_text(args.chain + "\n")
    original_lock = (ROOT / "Cargo.lock").read_bytes()
    (report_dir / "original.Cargo.lock").write_bytes(original_lock)
    policy = ROOT / ".cargo" / "audit.toml"
    if policy.is_file():
        shutil.copyfile(policy, report_dir / "audit.toml")
    shutil.copyfile(metadata_path, report_dir / metadata_path.name)
    print(f"Target: {target}\nReports: {report_dir}", flush=True)

    # Metadata is used only to identify package IDs reported by the actual builds.
    # Its resolved feature graph can overestimate dependencies on stable Cargo.
    cargo_metadata = json.loads(subprocess.check_output(
        [CARGO, "metadata", "--locked", "--format-version=1"], text=True
    ))
    packages_by_id = {p["id"]: p for p in cargo_metadata["packages"]}
    compiled = {}
    binaries = []
    build_env = os.environ.copy()
    # cargo-auditable's workspace wrapper still instruments native executables
    # and invalidates their Cargo cache. The outer wrapper lets nested Wasm
    # compile normally, without auditing the SDK's generated temporary lockfile.
    build_env["DUNITER_AUDIT_ORIGINAL_RUSTC_WRAPPER"] = build_env.get("RUSTC_WRAPPER", "")
    build_env["RUSTC_WRAPPER"] = str(ROOT / "scripts" / "audit-native-rustc.py")
    for binary in BINARIES:
        package = binary
        features = (
            f"{args.chain},embed" if binary == "duniter"
            else f"{args.chain},standalone,std"
        )
        print(f"Building {binary} with cargo-auditable...", flush=True)
        command = [
            CARGO, "auditable", "build", "--release", "--locked",
            "--no-default-features", "--features", features,
            "--target", target, "--package", package, "--bin", binary,
            "--message-format=json-render-diagnostics",
        ]
        messages = report_dir / f"{binary}.build.jsonl"
        with messages.open("w") as output:
            subprocess.run(command, stdout=output, check=True, env=build_env)

        executable = None
        binary_packages = set()
        for line in messages.read_text().splitlines():
            artifact = json.loads(line)
            if artifact.get("reason") == "compiler-artifact":
                package_info = packages_by_id[artifact["package_id"]]
                key = (
                    package_info["name"], package_info["version"], package_info["source"]
                )
                compiled.setdefault(key, set()).add(binary)
                binary_packages.add(key[:2])
            if (
                artifact.get("reason") == "compiler-artifact"
                and artifact.get("target", {}).get("name") == binary
                and artifact.get("executable")
            ):
                executable = Path(artifact["executable"])
        if executable is None or not executable.is_file():
            raise RuntimeError(f"Cargo did not produce {binary}")

        # Refuse cargo-audit's fallback that guesses dependencies from panic strings.
        embedded = subprocess.check_output(
            ["rust-audit-info", str(executable), str(executable.stat().st_size)],
            text=True,
        )
        metadata = json.loads(embedded)
        if not metadata.get("packages") or not any(
            p.get("root") and p.get("name") == package for p in metadata["packages"]
        ):
            raise RuntimeError(f"Missing or incorrect auditable metadata in {binary}")
        (report_dir / f"{binary}.dependencies.json").write_text(embedded + "\n")
        surplus = [
            p for p in metadata["packages"]
            if (p["name"], p["version"]) not in binary_packages
        ]
        (report_dir / f"{binary}.metadata-surplus.json").write_text(
            json.dumps(surplus, indent=2) + "\n"
        )
        if surplus:
            print(
                f"{binary}: {len(surplus)} embedded entries have no build artifact; "
                "see metadata-surplus.json. The compiled-dependency audit excludes them.",
                flush=True,
            )
        binaries.append(str(executable))

    (report_dir / "binaries.json").write_text(json.dumps(binaries, indent=2) + "\n")
    print("Auditing production executables...", flush=True)
    # Embedded chain specs can exceed cargo-audit's 100 MB default.
    command = [
        "cargo", "audit", "--color", "never",
        "bin", "--max-binary-size", str(max(Path(p).stat().st_size for p in binaries)),
        *binaries,
    ]
    binary_status = audit(command, report_dir / "audit-binaries.txt")
    if binary_status not in (0, 1):
        return 2

    # Audit only packages for which Cargo emitted an artifact, including cached
    # artifacts, build scripts and procedural macros. Keep the real Cargo.lock intact.
    if (ROOT / "Cargo.lock").read_bytes() != original_lock:
        raise RuntimeError("Cargo.lock changed during the audit; rerun with a stable lockfile")
    lock = tomllib.loads(original_lock.decode())
    active_lock = report_dir / "compiled.Cargo.lock"
    found = set()
    with active_lock.open("w") as output:
        output.write("version = 4\n")
        for package in lock["package"]:
            key = (package["name"], package["version"], package.get("source"))
            if key not in compiled:
                continue
            found.add(key)
            output.write("\n[[package]]\n")
            # Dependency edges are deliberately omitted. Binary membership is
            # recorded separately; feature-inactive lockfile edges are not evidence.
            for field in ("name", "version", "source", "checksum"):
                if field in package:
                    value = json.dumps(package[field], ensure_ascii=False)
                    output.write(f"{field} = {value}\n")
    if found != set(compiled):
        raise RuntimeError("Some compiled packages could not be matched to Cargo.lock")
    (report_dir / "compiled-packages.json").write_text(json.dumps([
        {"name": k[0], "version": k[1], "source": k[2], "binaries": sorted(v)}
        for k, v in sorted(
            compiled.items(),
            key=lambda item: (item[0][0], item[0][1], item[0][2] or ""),
        )
    ], indent=2) + "\n")
    print("Auditing dependencies confirmed by Cargo build artifacts...", flush=True)
    status = audit([
        "cargo", "audit", "--color", "never", "--no-fetch",
        "--file", str(active_lock),
    ], report_dir / "audit-compiled.txt")
    json_status = audit([
        "cargo", "audit", "--color", "never", "--no-fetch", "--json",
        "--file", str(active_lock),
    ], report_dir / "audit-compiled.json", json_output=True)
    result = json.loads((report_dir / "audit-compiled.json").read_text())
    if not isinstance(result.get("vulnerabilities", {}).get("list"), list):
        raise RuntimeError("cargo-audit did not return a complete vulnerability report")
    if json_status != status:
        raise RuntimeError("Text and JSON audits disagree; inspect the reports")
    print(
        f"Embedded-metadata audit exit code: {binary_status}\n"
        f"Compiled-dependency audit exit code: {status}\nReports: {report_dir}", flush=True
    )
    return status if status in (0, 1) else 2


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (
        OSError, ValueError, KeyError, TypeError, RuntimeError,
        subprocess.CalledProcessError,
    ) as error:
        print(f"Production audit failed: {error}", file=sys.stderr)
        sys.exit(2)
