from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib
import unittest


class VersionTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        (self.root / "scripts").mkdir()
        self.script = self.root / "scripts/set-version.py"
        shutil.copy(Path(__file__).resolve().parents[1] / "set-version.py", self.script)
        self.manifest = self.root / "Cargo.toml"
        self.manifest.write_text(
            '[workspace.package]\nversion = "0.20.0"\nedition = "2024"\n'
            '[workspace.dependencies]\n'
            'xrat-model = { path = "crates/xrat-model", version = "0.20.0" }\n'
            'serde = "1.0"\n'
        )

    def run_script(self, *args):
        return subprocess.run([sys.executable, str(self.script), *args], capture_output=True)

    def test_minor_bump_updates_internal_requirements(self):
        self.assertEqual(self.run_script("0.21.0").returncode, 0)
        data = tomllib.loads(self.manifest.read_text())["workspace"]
        self.assertEqual(data["package"]["version"], "0.21.0")
        self.assertEqual(data["dependencies"]["xrat-model"]["version"], "0.21.0")
        self.assertEqual(data["dependencies"]["serde"], "1.0")
        self.assertEqual(self.run_script("--check").returncode, 0)

    def test_check_rejects_unsynchronized_versions(self):
        self.manifest.write_text(self.manifest.read_text().replace('version = "0.20.0"', 'version = "0.21.0"', 1))
        self.assertNotEqual(self.run_script("--check").returncode, 0)

    def test_invalid_version_does_not_modify_manifest(self):
        original = self.manifest.read_text()
        self.assertNotEqual(self.run_script("invalid").returncode, 0)
        self.assertEqual(self.manifest.read_text(), original)
