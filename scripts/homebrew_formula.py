"""Generate the tap formula from release checksums. Never guess a digest."""
from pathlib import Path
import sys
import tomllib

version = tomllib.loads(Path("Cargo.toml").read_text())["package"]["version"]
artifacts = Path(sys.argv[1])
base = f"https://github.com/rawatpriyansh17/typedef/releases/download/v{version}"
lines = [
    "class Typedef < Formula",
    '  desc "Fast, lightweight terminal typing practice"',
    '  homepage "https://typedef-cli.vercel.app"',
    f'  version "{version}"',
    '  license "MIT"',
    "",
]
for os_name, platform in [("macos", "apple-darwin"), ("linux", "unknown-linux-musl")]:
    lines.append(f"  on_{os_name} do")
    for arch, cpu in [("arm", "aarch64"), ("intel", "x86_64")]:
        filename = f"typedef-v{version}-{cpu}-{platform}.tar.gz"
        digest = (artifacts / (filename + ".sha256")).read_text().split()[0]
        assert len(digest) == 64 and all(c in "0123456789abcdef" for c in digest)
        lines += [f"    on_{arch} do", f'      url "{base}/{filename}"', f'      sha256 "{digest}"', "    end"]
    lines += ["  end", ""]
lines += [
    "  def install", '    bin.install "typedef", "tpdf"', "  end", "",
    "  test do", '    assert_match version.to_s, shell_output("#{bin}/tpdf --version")',
    '    assert_match "--mode", shell_output("#{bin}/typedef --help")', "  end", "end", "",
]
(artifacts / "typedef.rb").write_text("\n".join(lines))
