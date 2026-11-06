# Typing-tool research and product decisions

This initial 0.1 documentation review is superseded by the [0.2 source-code audit](SOURCE_REVIEW.md), which includes the newly listed clavirio and implementation evidence.

Research date: 2026-11-05. Scope: all 12 tools in [Terminal Trove's typing catalog](https://terminaltrove.com/categories/typing/) plus Monkeytype. This is a documentation review of catalog descriptions and upstream READMEs, not a hands-on benchmark or complete source-code audit. Upstream features can change. The boomtypr catalog detail page failed to load; its upstream README was available.

## All twelve alternatives

| Tool / primary source | Distinctive documented ideas | Decision for typedef |
| --- | --- | --- |
| [boomtypr](https://github.com/yagnikpt/boomtypr) | Go/Bubble Tea; time, words, zen; uncluttered interface and responsive wrapping | Keep the short path from launch to typing. All three modes ship, with guided-text zen. |
| [gittype](https://github.com/unhappychoice/gittype) | Real project code, multiple languages, time attack, difficulty and developer-themed ranks | Ship small code lessons and local source-file input; defer repository extraction. Use progression titles without depending on a server. |
| [kbt](https://github.com/bloznelis/kbt) | Dedicated keyboard tester, multiple layouts, interactive menu; platform/input limitations | Ship a terminal-event keyboard lab. It is not a hardware diagnostic replacement; no global key hook or Accessibility permission. |
| [octotype](https://github.com/mahlquistj/octotype) | Custom modes, sources, themes, graphs; Rust/Ratatui | Separate content, scoring, rendering and storage. Three themes and graphs now; configurable packs and mode extensions later. |
| [smassh](https://github.com/kraanzu/smassh) | Time/words, speed carets, languages, themes; blind/confidence/force-correct modes and minimum thresholds | Ship strict difficulty levels and toggles. Add pacers and advanced modifiers after their scoring semantics are tested. Avoid required Nerd Fonts. |
| [toofan](https://github.com/vyrx-dev/toofan) | Curated code lessons, error review, ranks, per-duration bests, activity dashboard, local backups and ghosts; online work on a separate branch | Ship local history, error-key review and ranks. Queue detailed errors, ghost races, activity calendar and backup UX. |
| [ttyper](https://github.com/max-niederman/ttyper) | Rust/Ratatui, text-file input, language/keyword lists, detailed theme configuration | File input is first-class. Bundled English is an initial content pack, not a permanent English-only architecture. |
| [tukai](https://github.com/hlsxx/tukai) | Switchable templates, language selection, statistics, fast keyboard shortcuts | Ship navigable modes and rapid restart. Keep printable keys available during practice; richer template/language selection follows. |
| [typeinc](https://github.com/AnirudhG07/Typeinc) | Difficulty-driven scoring, grades and score lists; keyboard-oriented presentation | Reward careful practice with XP and badges. Keep speed and accuracy interpretable instead of hiding them behind a proprietary score. |
| [typespeed](https://typespeed.sourceforge.net/) | Arcade typing, CPS/WPM, high scores and network play | Plan a separate word-defense arcade and private races. Do not mix arcade scores with normal test bests. |
| [typioca](https://github.com/bloznelis/typioca) | Timed/count-based tests, prose lists, cursor-aware lines, Ctrl+W, custom/dynamic lists, SSH serving | Ship word deletion, prose and cursor-following text. SSH hosting is optional future infrastructure, not required for local practice. |
| [typtea](https://github.com/ashish0kumar/typtea) | Minimal UI, many programming-language vocabularies, scrolling typing text | Keep typing visually quiet. Expand code/vocabulary packs over time rather than promising large unreviewed collections. |

## Monkeytype as the breadth reference

[Monkeytype's upstream README](https://github.com/monkeytypegame/monkeytype) documents inline feedback, many lengths and languages, punctuation/numbers, quotes, themes, caret customization, accounts, challenges and modifiers. Its appeal is low friction plus deep customization. typedef's first release covers the basic practice loop, several modes, local progress and themes. Caret options, sounds, full language breadth, advanced modifiers, accounts and rankings are not implemented parity. The detailed staged backlog is in [ROADMAP.md](ROADMAP.md).

## Why Rust and Ratatui

Both Go/Bubble Tea and Rust/Ratatui are credible for this product; the catalog has successful examples of each. Rust is selected because it meets the requested stack, produces a native executable without a language runtime, and gives explicit ownership of timing and session state. [Ratatui](https://ratatui.rs/) provides the layouts, styled text, gauges and sparklines needed here. This is an architectural choice, not a claim that Rust automatically outperforms every Go application. The learning cost is addressed with a small module map, pure engine tests and a beginner contribution guide.

## Product principles

- Start practicing within two keystrokes; don't require onboarding or accounts.
- Accuracy should remain honest after corrections. Separate failed runs and different settings.
- Reward practice and consistency; make failures useful by surfacing weak characters.
- Never block the typing loop on network or disk work. Save only at settings changes, completion and exit.
- Keep content redistributable. Competitor ideas inform the design; their code and corpora have not been copied.
- Make online features additive: explicitly imported, cached text first; licensed community packs and private races later.
- A terminal has real constraints: cell-based cursor movement, inconsistent paste/key events, and variable Unicode rendering. State them instead of claiming browser-identical behavior.
