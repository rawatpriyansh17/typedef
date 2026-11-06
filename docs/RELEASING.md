# Publishing a release

1. Update the version in `Cargo.toml`, `scripts/install.sh`, `scripts/test_installer.py`, `website/package.json`, `website/package-lock.json`, the website release link, and README examples. Run `cargo check --offline` to refresh the local package version in `Cargo.lock`.
2. Update `docs/RELEASE_NOTES.md`. Run the Rust checks, installer tests and website build described in CI.
3. Commit the changes locally, then create and push the matching `vX.Y.Z` tag. Wait to push `main` until the release is public, so the website never points its installer at an unpublished version. Never move a published tag.
4. The Release workflow tests and builds five native targets. Unix release binaries also run through real pseudoterminal tests. Only after every target passes does it publish the archives, SHA-256 checksums, installer, and generated Homebrew formula together.
5. In [homebrew-tap Actions](https://github.com/rawatpriyansh17/homebrew-tap/actions/workflows/update.yml), run **Update formula** to make the release immediately available to Homebrew users. The daily schedule also picks up new published releases automatically.
6. Push `main` after the release and formula are public. Vercel deploys the `website` directory, including the installer at `/install.sh`. Verify the production installer and `brew install rawatpriyansh17/tap/typedef` against the new release.

Keep the formula hashes generated from the actual release archives. Release artifacts are immutable after publication; publish a new version for a correction. A failed tag build does not publish a release.
