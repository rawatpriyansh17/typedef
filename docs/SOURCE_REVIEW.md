> Historical v0.2 research notes. The v0.3 implementation removes audio entirely and adds explicit theme application, richer PR/progression views, daily quests, and further display controls. See [ROADMAP.md](ROADMAP.md) for current shipped behavior.

# Source review and UI plan — 0.2

Reviewed 2026-11-05. The catalog now lists **13 tools**, including clavirio. This pass inspected source code in shallow checkouts, not just README claims. No competitor implementation or assets were copied. This is a targeted audit of UI, wrapping, settings and feedback, not an exhaustive audit of every repository.

## Source evidence

### boomtypr
[Inspected source at a51b3bed](https://github.com/yagnikpt/boomtypr/blob/a51b3bed86c3b478cf23480326bfbe72b3654e11/internal/ui/model.go)
- Observed: Small centered composition, time/words/zen, resize-driven layout.
- Decision: Retain fast entry; replace the crowded home cards with a restrained launcher.

### clavirio
[Inspected source at d557f391](https://github.com/alexylon/clavirio/blob/d557f391a435807391ccad4b82ee81d0f3dd14d4/src/ui.rs)
- Observed: Keyboard rectangles, next-key highlights, finger hints, and whole-group wrapping.
- Decision: Add visible keycaps, QWERTY/Dvorak/Colemak maps, next-key guidance; layout diagrams do not remap OS input.

### gittype
[Inspected source at 09f3615c](https://github.com/unhappychoice/gittype/blob/09f3615c42b774c1fddea93ba3cd19af40cd7d4e/src/presentation/tui/views/session_detail_dialog/stage_results_view.rs)
- Observed: Stage-specific metrics and repository context.
- Decision: Keep code practice extensible; postpone repository extraction until source selection and licensing are designed.

### kbt
[Inspected source at 955802f3](https://github.com/bloznelis/kbt/blob/955802f3121d87062740a2ba80ac6b32bfb7ab94/src/view.rs)
- Observed: Key geometry with pressed/released/untouched styles; layout sizes.
- Decision: Draw real keycap geometry; do not claim terminal events equal hardware key diagnostics.

### octotype
[Inspected source at 77c394b5](https://github.com/mahlquistj/octotype/blob/77c394b5d8ca9116af2633a719812e0db3b59cbd/src/config.rs)
- Observed: Separate modes/sources/themes and ghost-line controls; online-source filtering.
- Decision: Offer width, spacing, and caret controls; no implicit network probe on launch.

### smassh
[Inspected source at 3a097972](https://github.com/kraanzu/smassh/blob/3a09797245c763852375cd336acc330ed36a1d36/smassh/ui/widgets/settings/settings_options.py)
- Observed: Described settings groups, confidence, blind, force-correct, threshold and caret options.
- Decision: Add grouped settings with explanations, no-backspace, blind, stop-on-error and target pacer.

### toofan
[Inspected source at 86e52f3e](https://github.com/vyrx-dev/toofan/blob/86e52f3e9850680730749270b4449d9bd5577258/internal/tui/profile.go)
- Observed: Monday-aligned activity grid, historical profile panels.
- Decision: Build intensity-based calendar with a legend and current/longest streak; retain actual dates.

### ttyper
[Inspected source at e218bad9](https://github.com/max-niederman/ttyper/blob/e218bad9d5c546ff31c1b2a141a42c2eebf50d23/src/ui.rs)
- Observed: Worst-key report and Braille line chart with labeled axes.
- Decision: Replace decorative sparklines with WPM/raw line charts and explicit units.

### tukai
[Inspected source at 6d948c36](https://github.com/hlsxx/tukai/blob/6d948c369533ea648e52f78ce1946e23029c8d22/src/screens/typing.rs)
- Observed: Separates current mistakes from all mistakes; styled inline typing.
- Decision: Preserve correction-aware accuracy and add independent visual feedback controls.

### Typeinc
[Inspected source at 553e1440](https://github.com/AnirudhG07/Typeinc/blob/553e14406b4758359a256c15ee39956c747c6a27/src/typeinc/configurations/keyboard.py)
- Observed: Keycap labels, modifier pairs, physical row coordinates.
- Decision: Use adaptive key widths rather than fixed terminal coordinates; avoid blocking effects.

### typespeed
[Inspected source at e0172115](https://github.com/mikeizbicki/typespeed/blob/e01721156e2ef15e600ae09426dee0da88e1e1d2/game/src/typespeed.c)
- Observed: Moving-word arcade and separate network/score routines.
- Decision: Arcade and multiplayer are separate product modes, not cosmetic settings. This inspected source is a teaching fork, not the upstream release.

### typioca
[Inspected source at baf41701](https://github.com/bloznelis/typioca/blob/baf41701e43deaba652e250abc048c043b046fc4/cmd/view.go)
- Observed: Word-aware wrapping explicitly accounts for whitespace and cursor display.
- Decision: Keep every source character indexed when reflowing, including trailing spaces and newlines.

### typtea
[Inspected source at f09f4ba7](https://github.com/ashish0kumar/typtea/blob/f09f4ba7eb0cd9a205024f1b0f13e8bae4176b89/internal/tui/view.go)
- Observed: Minimal centered test, scrolling lines, restrained metrics.
- Decision: Use a centered, limited-width typing surface and optional focus mode.

### monkeytype
[Inspected source at 2df8cbd4](https://github.com/monkeytypegame/monkeytype/blob/2df8cbd468386d6b0cc3dcbf35316b608bf6f8b3/frontend/src/ts/constants/default-config.ts)
- Observed: Font size, caret, confidence, blind, keymap, sounds/volume, pace caret, graph and account options.
- Decision: Translate portable settings; separate browser-specific font/audio/account features from terminal capabilities.

## Problems in typedef 0.1

- Wrapping happens per character, splitting words. Reflow must preserve character indices and never change the scored text. Only a token longer than the viewport may split.
- Marketing headings and excessive panels compete with the actual test. Remove the tagline and OFFLINE FIRST, use a small brand/navigation bar, increase whitespace around text.
- Three themes and six flat settings offer little control. Add a live theme gallery, organized settings, descriptive help and scrolling.
- Sparklines lack labeled time axes and raw/error context. Capture timestamped speed samples and show labeled charts.
- Streaks already exist, but there is no activity calendar, longest streak or key heatmap. Reuse real history; never seed fake progress.
- Current keyboard is just letters. Render keycaps and display next-key / recent-key feedback, with heatmaps separately labeled.

## Implementation scope

Ship: word-aware reflow with regression tests; twelve themes preserving original theme IDs; settings categories and previews; caret styles, line spacing, content width, focus/live metrics/keyboard toggles; QWERTY, Dvorak, Colemak diagrams; optional sound and visual error pulse; blind/no-backspace/stop-on-error modifiers; pacer; configurable daily goal; WPM/raw charts; dated activity calendar, longest streak, per-key error heatmap; missed-word retry; shareable deterministic seed via CLI.

## Capability boundaries

- **Font size** belongs to the terminal emulator. Provide terminal zoom guidance, width and line-spacing settings. Kitty has an [OSC 66 text-sizing protocol](https://sw.kovidgoyal.net/kitty/text-sizing-protocol/) but it is not a portable Ratatui font-size switch. Do not silently change all terminal tabs or fake a working font-size setting.
- **Haptics**: a normal terminal stream has no portable vibration API. Use an optional short visual error pulse and sound; actual device haptics remain unsupported.
- **Audio**: off by default; synthesized, original short sounds through an available system player, bounded background delivery, with terminal bell fallback. No audio library/service required for normal typing.
- **Online**: keep explicit HTTPS import/cache. Add a reproducible seed for sharing challenges. Accounts, public leaderboards and races require infrastructure, consent, validation and conflict handling; do not display fake online users or rankings.
- **Fairness**: daily challenge fixes content-affecting modifiers. Different training modifiers get separate personal-best categories. Blind mode must suppress accidental visual/audio error hints.

## Remaining breadth

Guided lessons/bigrams, language packs, code-source selection, ghost replays, richer exports/backups, free-writing zen, graphical font scaling, public content catalog, multiplayer and account sync remain separately scoped work. A feature matrix is useful, but claiming full Monkeytype parity would be inaccurate.
