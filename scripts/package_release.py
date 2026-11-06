"""Package tested native binaries and emit SHA-256 sidecars."""
import hashlib
import os
from pathlib import Path
import sys
import tarfile
import tomllib
import zipfile

target = sys.argv[1]
version = "v" + tomllib.loads(Path("Cargo.toml").read_text())["package"]["version"]
assert os.environ.get("GITHUB_REF_NAME", version) == version, "Tag and Cargo version differ"
windows = "windows" in target
suffix = ".exe" if windows else ""
out = Path("release")
out.mkdir(exist_ok=True)
archive = out / f"typedef-{version}-{target}.{'zip' if windows else 'tar.gz'}"
members = [(Path(f"target/{target}/release/{name}{suffix}"), f"{name}{suffix}") for name in ["typedef", "tpdf"]]
members += [(Path("LICENSE"), "LICENSE"), (Path("README.md"), "README.md")]
if windows:
    with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as bundle:
        for path, name in members:
            bundle.write(path, name)
else:
    with tarfile.open(archive, "w:gz") as bundle:
        for path, name in members:
            bundle.add(path, arcname=name)
digest = hashlib.sha256(archive.read_bytes()).hexdigest()
archive.with_name(archive.name + ".sha256").write_text(f"{digest}  {archive.name}\n")
print(f"Packaged {archive}")
