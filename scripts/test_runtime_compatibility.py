#!/usr/bin/env python3
"""Exercise the metadata comparison without a node or a Rust build."""

from copy import deepcopy
import unittest

from check_runtime_compatibility import check


def metadata(network):
    def string(value):
        encoded = value.encode()
        return [len(encoded) << 2, *encoded]
    common = {
        "name": "System", "index": 0,
        "constants": [{"name": "Version", "ty": 0,
                       "value": string(network) + string("duniter") + [1, 2, 3]}],
        "calls": {"ty": 1},
    }
    variants = [{"name": "System", "index": 0, "fields": [{"type": 0}]}]
    pallets = [common]
    if network == "gtest":
        pallets.append({"name": "Sudo", "index": 20})
        variants.append({"name": "Sudo", "index": 20, "fields": []})
    return {
        "types": {"types": [
            {"id": 0, "type": {"def": {"primitive": "u32"}}},
            {"id": 1, "type": {"path": [f"{network}_runtime", "RuntimeCall"],
                                "def": {"variant": {"variants": variants}}}},
        ]},
        "pallets": pallets,
        "extrinsic": {"call_ty": 1},
    }


class CompatibilityTests(unittest.TestCase):
    def setUp(self):
        self.gtest = metadata("gtest")
        self.g1 = metadata("g1")

    def test_allowed_differences(self):
        check(self.gtest, self.g1)

    def test_type_ids_can_differ(self):
        moved = deepcopy(self.g1["types"]["types"][0])
        moved["id"] = 42
        self.g1["types"]["types"][0] = moved
        self.g1["pallets"][0]["constants"][0]["ty"] = 42
        self.g1["types"]["types"][1]["type"]["def"]["variant"]["variants"][0]["fields"][0]["type"] = 42
        check(self.gtest, self.g1)

    def test_root_type_id_can_differ(self):
        self.gtest["ty"] = 0
        self.g1["ty"] = 42
        self.g1["types"]["types"].append({"id": 42, "type": {"def": {"primitive": "u32"}}})
        check(self.gtest, self.g1)

    def test_reject_different_root_type(self):
        self.gtest["ty"] = 0
        self.g1["ty"] = 42
        self.g1["types"]["types"].append({"id": 42, "type": {"def": {"primitive": "u64"}}})
        with self.assertRaises(AssertionError):
            check(self.gtest, self.g1)

    def test_reject_sudo_in_g1(self):
        self.g1["pallets"].append({"name": "Sudo", "index": 20})
        with self.assertRaises(AssertionError):
            check(self.gtest, self.g1)

    def test_reject_sudo_call_without_pallet(self):
        self.g1["types"]["types"][1]["type"]["def"]["variant"]["variants"].append({"name": "Sudo", "index": 20, "fields": []})
        with self.assertRaises(AssertionError):
            check(self.gtest, self.g1)

    def test_reject_different_call_encoding(self):
        self.g1["types"]["types"][1]["type"]["def"]["variant"]["variants"][0]["index"] = 1
        with self.assertRaises(AssertionError):
            check(self.gtest, self.g1)

    def test_reject_extra_g1_section(self):
        self.g1["custom"] = {}
        with self.assertRaises(AssertionError):
            check(self.gtest, self.g1)

    def test_reject_runtime_names_outside_type_paths(self):
        self.gtest["extrinsic"]["extension_name"] = "gtest_runtime"
        self.g1["extrinsic"]["extension_name"] = "g1_runtime"
        with self.assertRaises(AssertionError):
            check(self.gtest, self.g1)

    def test_reject_different_constant(self):
        for state, value in ((self.gtest, 5), (self.g1, 2)):
            state["pallets"][0]["constants"].append({"name": "MinCerts", "ty": 0, "value": [value]})
        with self.assertRaises(AssertionError):
            check(self.gtest, self.g1)


class G1RefactorTests(unittest.TestCase):
    def setUp(self):
        self.before = metadata("gtest")
        self.after = metadata("g1")
        version = self.before["pallets"][0]["constants"][0]
        def string(value):
            raw = value.encode()
            return [len(raw) << 2, *raw]
        version["value"] = string("g1") + string("duniter-g1") + [1, 2, 3]
        self.before["types"]["types"][1]["type"]["path"][0] = "g1_runtime"

    def test_only_expected_g1_changes(self):
        check(self.before, self.after, before_g1=True)

    def test_reject_changed_g1_type_path(self):
        self.after["types"]["types"][1]["type"]["path"][0] = "gtest_runtime"
        with self.assertRaises(AssertionError):
            check(self.before, self.after, before_g1=True)

    def test_reject_changed_spec_version(self):
        self.after["pallets"][0]["constants"][0]["value"][-1] += 1
        with self.assertRaises(AssertionError):
            check(self.before, self.after, before_g1=True)

    def test_reject_unexpected_implementation(self):
        self.before["pallets"][0]["constants"][0]["value"][5] = ord("x")
        with self.assertRaises(AssertionError):
            check(self.before, self.after, before_g1=True)

    def test_reject_changed_pallet_index(self):
        self.after["pallets"][0]["index"] = 4
        with self.assertRaises(AssertionError):
            check(self.before, self.after, before_g1=True)

    def test_reject_changed_runtime_api(self):
        self.before["apis"] = [{"name": "Core", "output": 0}]
        self.after["apis"] = [{"name": "Other", "output": 0}]
        with self.assertRaises(AssertionError):
            check(self.before, self.after, before_g1=True)

    def test_reject_changed_storage_type(self):
        self.before["pallets"][0]["storage"] = {"entries": [{"type": {"Plain": 0}}]}
        self.after["pallets"][0]["storage"] = {"entries": [{"type": {"Plain": 1}}]}
        with self.assertRaises(AssertionError):
            check(self.before, self.after, before_g1=True)


if __name__ == "__main__":
    unittest.main()
