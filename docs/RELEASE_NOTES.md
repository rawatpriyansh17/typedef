Native terminal typing practice with nine modes, 28 themes, personal records and continuous progression.

This release adds prebuilt binaries for macOS (Apple Silicon and Intel), Linux (ARM64 and x86-64, static musl), and Windows x86-64. Every archive contains both `typedef` and `tpdf`, plus its license. macOS binaries target macOS 11 or later.

The macOS/Linux shell installer verifies the release SHA-256 before extraction and installs to `~/.local/bin` without sudo or shell-profile changes. Install with:

```sh
curl -fsSL https://typedef-tui.vercel.app/install.sh | sh
# Or use the project's Homebrew tap:
brew install rawatpriyansh17/tap/typedef
```

Then run `tpdf`. Windows users can extract the ZIP and run `tpdf.exe` in Windows Terminal.

Source, checksums, the installer and the generated Homebrew formula are included below.
