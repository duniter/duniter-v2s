#!/usr/bin/env python3
"""Compare runtime metadata across networks or across the G1 refactor."""

import argparse
import json
from pathlib import Path
import subprocess


def load(path):
    encoded = subprocess.check_output(
        ["subxt", "metadata", "--file", str(path), "--format", "json"], text=True
    )
    _, versions = json.loads(encoded)
    version, metadata = next(iter(versions.items()))
    if version not in ("V15", "V16"):
        raise ValueError(f"Unsupported metadata version: {version}")
    return version, metadata


def version_without_name(value, expected, implementation_name="duniter", keep_spec_name=False):
    data = bytes(value)
    # Both network names and the shared implementation name fit SCALE's one-byte mode.
    def string(data):
        if data[0] & 3:
            raise AssertionError("Unexpected runtime name encoding")
        length = data[0] >> 2
        return data[1:1 + length].decode(), data[1 + length:]
    name, rest = string(data)
    assert name == expected, f"Expected spec_name={expected}, got {name}"
    implementation, tail = string(rest)
    assert implementation == implementation_name, f"Unexpected impl_name={implementation}"
    return list(data[:len(data) - len(rest)] + tail) if keep_spec_name else list(rest)


def check(gtest, g1, *, before_g1=False):
    assert gtest.keys() == g1.keys(), "Metadata sections differ"
    registries = [{t["id"]: t["type"] for t in m["types"]["types"]} for m in (gtest, g1)]
    seen = set()
    references = {
        "type", "ty", "key", "value", "Plain", "output", "implicit",
        "additional_signed", "bitStoreType", "bitOrderType",
    }

    def compare_type(left, right, where):
        pair = (left, right)
        if pair in seen:
            return
        seen.add(pair)
        compare(registries[0][left], registries[1][right], f"{where}.type")

    def compare(left, right, where, key=""):
        if isinstance(left, dict) and isinstance(right, dict):
            a = {k: v for k, v in left.items() if k != "docs"}
            b = {k: v for k, v in right.items() if k != "docs"}
            assert a.keys() == b.keys(), f"Different fields at {where}: {a.keys()} / {b.keys()}"
            for field in a:
                compare(a[field], b[field], f"{where}.{field}", field)
        elif isinstance(left, list) and isinstance(right, list):
            assert len(left) == len(right), f"Different lengths at {where}"
            for index, (a, b) in enumerate(zip(left, right)):
                if key == "tuple":
                    compare_type(a, b, f"{where}[{index}]")
                else:
                    compare(a, b, f"{where}[{index}]", "path" if key == "path" else "")
        elif isinstance(left, int) and isinstance(right, int) and (key in references or key.endswith("_ty")):
            compare_type(left, right, where)
        else:
            def normalize(value):
                if not before_g1 and key == "path" and value in ("gtest_runtime", "g1_runtime"):
                    return "duniter_runtime"
                return value
            assert normalize(left) == normalize(right), f"Difference at {where}: {left!r} / {right!r}"

    # Remove only the explicitly allowed sudo variants from the aggregate runtime enums.
    for network_index, registry in enumerate(registries):
        for ty in registry.values():
            path = ty.get("path", [])
            if path and path[0] in ("gtest_runtime", "g1_runtime") and path[-1] in ("RuntimeCall", "RuntimeEvent", "RuntimeError"):
                variants = ty.get("def", {}).get("variant", {}).get("variants", [])
                if network_index == 1:
                    assert all(v["name"] != "Sudo" for v in variants), "G1 must not expose a sudo call or event"
                ty["def"]["variant"]["variants"] = [v for v in variants if v["name"] != "Sudo"]

    test_pallets = {p["name"]: p for p in gtest["pallets"]}
    main_pallets = {p["name"]: p for p in g1["pallets"]}
    assert "Sudo" not in main_pallets, "G1 must not contain sudo"
    assert test_pallets.pop("Sudo")["index"] == 20, "The reference sudo pallet must retain index 20"
    assert test_pallets.keys() == main_pallets.keys(), "Unexpected pallet differences"
    for name, pallet in test_pallets.items():
        other = main_pallets[name]
        if name == "System":
            for metadata, expected, implementation in (
                (pallet, "g1" if before_g1 else "gtest", "duniter-g1" if before_g1 else "duniter"),
                (other, "g1", "duniter"),
            ):
                constant = next(c for c in metadata["constants"] if c["name"] == "Version")
                constant["value"] = version_without_name(
                    constant["value"], expected, implementation, keep_spec_name=before_g1
                )
        compare(pallet, other, name)
    for section in gtest.keys() - {"types", "pallets"}:
        compare(gtest[section], g1[section], section, section)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--before-g1", type=Path, help="Compare pre-refactor G1 with --g1")
    parser.add_argument("--gtest", type=Path, default=Path("resources/gtest_metadata.scale"))
    parser.add_argument("--g1", type=Path, default=Path("resources/g1_metadata.scale"))
    args = parser.parse_args()
    test_version, gtest = load(args.before_g1 or args.gtest)
    main_version, g1 = load(args.g1)
    assert test_version == main_version, "Metadata versions differ"
    check(gtest, g1, before_g1=args.before_g1 is not None)
    if args.before_g1:
        print("G1 metadata match apart from sudo removal and impl_name.")
        return
    print("Runtime metadata match apart from spec_name and gtest sudo.")


if __name__ == "__main__":
    main()
