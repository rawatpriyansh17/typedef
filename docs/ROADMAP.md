# Product roadmap

The goal is a terminal-native daily practice tool with the breadth of Monkeytype, not a visual clone. Status below distinguishes shipped behavior from proposals. No backend service is required.

## 0.1 — usable offline foundation (implemented)

Eight practice modes, bundled content, file and HTTPS imports, strict difficulties, three themes, keyboard lab, correction-aware scoring, results, JSON profile/export, weak-character word selection, deterministic daily seed, XP, levels, badges, streaks, contributor documentation, CI.

## 0.2 — interface and practice controls (implemented)

Removed promotional headings. Word-aware cached reflow; twelve themes and a live gallery; four settings categories; keyboard diagrams in three layouts; current/longest streaks, activity calendar and key heatmap; labeled WPM/raw curves and matching-settings history; optional background sound/visual pulse; caret/spacing/width/focus controls; blind/no-backspace/stop-on-error training; target pacer; missed-word retry; configurable daily goals; shared CLI seeds. See [SOURCE_REVIEW.md](SOURCE_REVIEW.md) for the source audit that informed the work.

## 0.3 — records, progression and customization (implemented)

PR dashboard and category records; block numerals, rank emblems, XP meters, recent speed sparkline and seven-day habit strip; previous-PR result comparison; ten ranks with unlimited levels; sixteen achievements; three daily quests with persistent once-per-day bonuses; distinct preview/applied theme states; 28 settings including accents, borders, key/graph styles, bold text, visible rows and static celebrations; live customization preview; compact layout fixes. Existing XP/history/badges are retained. Sound support, settings and playback code were removed.

## 0.4 — width, navigation and endless progression (implemented)

Removed the 116-column canvas cap; dashboard cards and collection grids reflow across the available width. Added a two-line TD monogram, direct cross-screen shortcuts, clickable header navigation, and protected typing/key-lab input. Expanded the gallery to 28 palettes while preserving every previous ID; added paging and first/last navigation. Six repeatable badge tracks award XP at each new tier, recognize qualifying historical progress without retroactive payouts, and have no final tier. Legend cycles continue every 50 levels after level 101. Text width supports automatic window sizing and wider presets.

## Next — typing depth

Shipped in 0.5: wider PR digits and consistent spacing; mode icons and an in-app guide; Endless with 50-word laps and unlimited quiet Zen; a local file picker with pasted paths and bounded UTF-8 reads; Tab indentation in Code and Custom; both `tpdf` and `typedef` commands; a responsive landing page with real app media and independent Vercel configuration in `website/`.

- Further optimize very long sessions and large-file scoring; grapheme-aware caret positions.
- Larger frequency-based dictionaries, language/source selector, curated lessons and content-pack metadata with licenses.
- Grapheme-cluster indexing, normalization options, IME/RTL feasibility, layout-specific beginner lessons.
- Minimum accuracy/speed thresholds, capitalization modifiers, repeated-word drill controls.
- Code source picker, repository snippet extraction without execution, indentation policies and language-aware lessons.
- Per-key and bigram latency, statistical confidence for weak-key rankings, finger guidance.
- Saved personal ghost runs; distinguish replay records from fresh attempts.
- Richer session filters, content-specific bests, CSV export, migrations, backup/restore.
- Home-row progression. Balance XP against trivial/very short replays.
- Free-writing zen, more keyboard maps, color-blind palettes and terminal-specific text scaling.

## Then — fun and connection

- A Typespeed-inspired word-defense arcade with lives, escalating difficulty, and local scores.
- Versioned, shareable challenge files with seed, content digest, and exact settings; race asynchronously offline.
- Opt-in community content catalog: checksum-verified, licensed plain-text packs, cached permanently. This is the preferred first richer online feature.
- Private online races with a small, separately deployable server; self-hostable rooms and clear disconnect behavior.
- Optional public daily leaderboard only after score validation, rate limiting, moderation, and explicit opt-in exist. Local scores must never masquerade as verified results.
- Optional encrypted profile sync, conflict resolution, and deletion controls. No account needed for local use.

## Release engineering

Test terminals and OSes beyond local macOS; publish signed/checksummed macOS, Linux, and Windows release artifacts; verify package name availability before crates.io/Homebrew distribution. Configure repository URLs, release automation, issue templates, and maintainership before public launch. Benchmark startup, memory, long-session rendering, and large imports on modest hardware. Do not promise platform compatibility or latency targets without measurements.
