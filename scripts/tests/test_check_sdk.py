import importlib.util
from pathlib import Path
import unittest


spec = importlib.util.spec_from_file_location("check_sdk", Path(__file__).resolve().parents[1] / "check-sdk.py")
check_sdk = importlib.util.module_from_spec(spec)
spec.loader.exec_module(check_sdk)


class RegistryDependencyTests(unittest.TestCase):
    def test_local_consumer_with_registry_sdk_is_valid(self):
        packages = [
            {"id": "consumer", "name": "xrat-sdk-consumer", "source": None},
            {"id": "sdk", "name": "xrat-sdk", "source": "registry+https://github.com/rust-lang/crates.io-index"},
        ]
        check_sdk.check_registry_dependencies(packages, ["consumer"])

    def test_internal_path_and_git_dependencies_are_rejected(self):
        for source in [None, "git+https://github.com/mhyrzt/xrat"]:
            with self.subTest(source=source), self.assertRaises(AssertionError):
                check_sdk.check_registry_dependencies(
                    [{"id": "config", "name": "xrat-config", "source": source}], ["consumer"])
