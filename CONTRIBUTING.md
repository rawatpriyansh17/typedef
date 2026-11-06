# Contributing to typedef

Small, thoughtful improvements are welcome: lessons, accessibility, bug reports, documentation, and code. Keep everyday practice offline, respect user data, and avoid adding dependencies unless they solve a concrete problem.

## New to Rust?

Rust compiles the source into one native executable. Cargo is its build, dependency, and test tool. `cargo run` compiles and runs; `cargo test` checks behavior; `cargo fmt` formats; `cargo clippy` catches common mistakes. Start with [The Rust Book](https://doc.rust-lang.org/book/) and [Ratatui's tutorials](https://ratatui.rs/tutorials/).

1. Install stable Rust with rustup.
2. Clone your fork, create a branch, and run `cargo run`.
3. Make a focused change. Add a regression test for behavior changes.
4. Run formatting, Clippy with warnings denied, and tests as listed in the README.
5. Explain the user-visible behavior and validation in your pull request. Include terminal size and OS for UI fixes.

## Code map

- `src/lib.rs`: shared command-line options, imports, terminal lifecycle and input loop.
- `src/main.rs` and `src/bin/tpdf.rs`: entry points into the same application.
- `src/import.rs`: in-app directory browsing and local path resolution.
- `src/engine.rs`: typing state and deterministic scoring. Inject elapsed time in tests.
- `src/content.rs`: seeded content generation and normalization.
- `src/store.rs`: settings, history, progression, locking, persistence.
- `src/app.rs`: navigation and actions; connects engine and persistence.
- `src/ui.rs`: Ratatui rendering, header hit areas, and responsive layout tests.
- `src/text_layout.rs`: cached word reflow with original character indices.
- `src/theme.rs` and `src/settings.rs`: palettes and settings metadata/actions.
- `src/progression.rs`: rank cycles, repeatable badge tracks, badges and daily quests; reward accounting lives in `store.rs`.
- `assets/english.txt`: bundled vocabulary.
- `website/`: independent static landing page; its README covers development and Vercel.
- `scripts/pty_smoke.py`: real Unix terminal integration checks.
- `scripts/render_media.py`: turn deterministic Ratatui captures into website images and video.

To add a mode, define its engine semantics before wiring the UI. Avoid keybindings that steal printable characters during practice. Never start a timer before the first typed character. Keep corrected mistakes in accuracy. Explicitly test deadline boundaries, UTF-8, empty content, and resize behavior.

To contribute text, write it yourself or provide its source and redistribution license. Don't copy Monkeytype's GPL datasets into this MIT repository without resolving licensing. Code snippets are text only and must never execute. For new languages, include native-speaker review and discuss grapheme/IME behavior.

Do not commit profiles, private text, binaries, or downloaded practice files. Preserve existing user data on parse and write errors. Networking must require an explicit action and work outside the input/rendering loop.

## Community

Website contributors: use Node.js 20+ and Python 3. Run `cd website && npm ci && npm run build`, then `npm run dev` to preview. Check phone and desktop layouts, keyboard navigation, visible focus and reduced-motion behavior. Use genuine app captures; keep copy concise. The app and website build independently.

Be welcoming, patient, and specific. Critique ideas and code, not people. Harassment and disclosure of another person's private information are not acceptable. Maintainers may remove abusive contributions. Security reports should omit private profile data and executable exploit payloads from public issues.
