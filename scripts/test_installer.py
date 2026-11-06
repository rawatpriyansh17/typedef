"""Exercise installer success and failure boundaries without network or user files."""
import hashlib
import io
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest

INSTALLER = Path(__file__).with_name("install.sh").resolve()


class InstallerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="typedef-installer-test-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.bin = self.root / "install with spaces"
        mock = self.root / "commands"
        mock.mkdir()
        (mock / "uname").write_text('#!/bin/sh\ncase "$1" in -s) echo "${TEST_OS:-Linux}";; -m) echo "${TEST_ARCH:-x86_64}";; esac\n')
        (mock / "curl").write_text('''#!/usr/bin/env python3
import os, pathlib, shutil, sys
if os.environ.get('TEST_DOWNLOAD_FAIL'): sys.exit(22)
url = next(arg for arg in sys.argv if arg.startswith('https://'))
assert url.startswith('https://github.com/rawatpriyansh17/typedef/releases/download/v0.5.2/')
source = pathlib.Path(os.environ['TEST_ARTIFACTS']) / url.rsplit('/', 1)[-1]
shutil.copyfile(source, sys.argv[sys.argv.index('-o') + 1])
''')
        for tool in mock.iterdir():
            tool.chmod(0o755)
        self.env = dict(os.environ, PATH=f"{mock}:{os.environ['PATH']}", TYPEDEF_INSTALL_DIR=str(self.bin), TEST_ARTIFACTS=str(self.root))
        self.env.pop("TYPEDEF_VERSION", None)
        self.archive = self.root / "typedef-v0.5.2-x86_64-unknown-linux-musl.tar.gz"
        self.make_archive(self.archive)

    def make_archive(self, path):
        with tarfile.open(path, "w:gz") as bundle:
            for name in ["typedef", "tpdf"]:
                content = b'#!/bin/sh\nprintf "typedef 0.5.2\\n"\n'
                member = tarfile.TarInfo(name)
                member.mode = 0o755
                member.size = len(content)
                bundle.addfile(member, io.BytesIO(content))
        path.with_name(path.name + ".sha256").write_text(hashlib.sha256(path.read_bytes()).hexdigest() + "  " + path.name + "\n")

    def run_installer(self, *args):
        return subprocess.run(["sh", str(INSTALLER), *args], env=self.env, capture_output=True, text=True)

    def test_install_spaces_repeat_and_symlink(self):
        self.bin.mkdir()
        unrelated = self.root / "untouched"
        unrelated.write_text("keep me")
        (self.bin / "tpdf").symlink_to(unrelated)
        for _ in range(2):
            result = self.run_installer()
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("0.5.2", subprocess.check_output([self.bin / "tpdf", "--version"], text=True))
        self.assertEqual(unrelated.read_text(), "keep me")
        self.assertFalse((self.bin / "tpdf").is_symlink())

    def test_tampered_archive_preserves_install(self):
        self.assertEqual(self.run_installer().returncode, 0)
        before = (self.bin / "tpdf").read_bytes()
        self.archive.write_bytes(b"corrupted download")
        result = self.run_installer()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Checksum mismatch", result.stderr)
        self.assertEqual((self.bin / "tpdf").read_bytes(), before)

    def test_network_failure_does_not_install(self):
        self.env["TEST_DOWNLOAD_FAIL"] = "1"
        self.assertNotEqual(self.run_installer().returncode, 0)
        self.assertFalse(self.bin.exists())

    def test_unsupported_cpu_and_invalid_arguments(self):
        self.env["TEST_ARCH"] = "riscv64"
        self.assertNotEqual(self.run_installer().returncode, 0)
        self.assertNotEqual(self.run_installer("--version", "../bad").returncode, 0)
        self.assertNotEqual(self.run_installer("--bin-dir").returncode, 0)
        self.assertNotEqual(self.run_installer("--bin-dir", "relative").returncode, 0)
        self.assertFalse(self.bin.exists())

    def test_platform_detection(self):
        for system, machine, target in [
            ("Darwin", "arm64", "aarch64-apple-darwin"),
            ("Darwin", "x86_64", "x86_64-apple-darwin"),
            ("Linux", "aarch64", "aarch64-unknown-linux-musl"),
        ]:
            self.env.update(TEST_OS=system, TEST_ARCH=machine)
            self.make_archive(self.root / f"typedef-v0.5.2-{target}.tar.gz")
            result = self.run_installer()
            self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == "__main__":
    unittest.main()
