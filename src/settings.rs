use crate::{
    engine::{Difficulty, Settings},
    theme::THEMES,
};
pub const GROUPS: [&str; 4] = ["practice", "appearance", "interface", "training"];
pub fn count(group: usize) -> usize {
    [6, 12, 5, 5][group % 4]
}
pub fn rows(s: &Settings, group: usize) -> Vec<(&'static str, String, &'static str)> {
    let on = |b| if b { "on".into() } else { "off".into() };
    match group % 4 {
        0 => vec![
            (
                "Duration",
                format!("{} seconds", s.seconds),
                "Timed tests. Daily challenges always use 60 seconds.",
            ),
            (
                "Word count",
                s.words.to_string(),
                "Applies to words and weak-key practice.",
            ),
            (
                "Punctuation",
                on(s.punctuation),
                "Add punctuation to generated words.",
            ),
            (
                "Numbers",
                on(s.numbers),
                "Mix numbers into generated words.",
            ),
            (
                "Difficulty",
                s.difficulty.label().into(),
                "Normal: continue. Expert: fail at a space with errors. Master: fail on any mistake.",
            ),
            (
                "Daily goal",
                format!("{} sessions", s.daily_goal),
                "Successful sessions per local calendar day.",
            ),
        ],
        1 => vec![
            (
                "Theme",
                THEMES[s.theme % THEMES.len()].name.into(),
                "Applied immediately here. Press t on Home to preview a gallery before applying.",
            ),
            (
                "Caret",
                ["block", "underline", "highlight"][s.caret % 3].into(),
                "Choose how the current character is highlighted.",
            ),
            (
                "Line spacing",
                ["compact", "comfortable", "airy"][s.line_gap.min(2)].into(),
                "Space between typing rows. Font size uses your terminal's zoom controls (often Cmd+ or Ctrl+Shift+).",
            ),
            (
                "Text width",
                if s.text_width == 0 {
                    "auto (window)".into()
                } else {
                    format!("{} columns", s.text_width)
                },
                "Auto follows the window. Fixed widths keep a reading measure. Whole words reflow on resize.",
            ),
            (
                "Keyboard",
                on(s.show_keyboard),
                "Show next-key guidance below the text when there is room.",
            ),
            (
                "Keyboard layout",
                ["QWERTY", "Dvorak", "Colemak"][s.keyboard_layout % 3].into(),
                "Changes the diagram, not your operating system's input layout.",
            ),
            (
                "Live metrics",
                on(s.live_stats),
                "Show speed and accuracy while typing. Results always include both.",
            ),
            (
                "Focus mode",
                on(s.focus),
                "Hide navigation, keyboard and extra metrics during a test.",
            ),
            (
                "Accent color",
                ["theme default", "mint", "cyan", "gold", "coral", "lilac"][s.accent].into(),
                "Override the active theme's accent. Reset to theme default to use its original palette.",
            ),
            (
                "Panel borders",
                ["rounded", "sharp", "double", "borderless"][s.panel_style].into(),
                "Customize cards and charts throughout the interface.",
            ),
            (
                "Key style",
                ["keycaps", "tiles", "outline"][s.key_style].into(),
                "Choose filled keycaps, flat tiles or an outlined keyboard.",
            ),
            (
                "Graph marks",
                ["braille", "dots", "blocks"][s.graph_style].into(),
                "Use dots or blocks if your terminal font does not render Braille well.",
            ),
        ],
        2 => vec![
            (
                "Visible lines",
                s.visible_lines.to_string(),
                "Maximum text rows shown at once. The viewport still follows your cursor.",
            ),
            (
                "Bold text",
                on(s.bold_text),
                "Heavier typing text without changing the terminal font size.",
            ),
            (
                "Large results",
                on(s.large_results),
                "Use pixel-style WPM numerals on roomy result screens.",
            ),
            (
                "Celebrations",
                on(s.celebrations),
                "Decorative PR and level-up banners. No flashing or motion.",
            ),
            (
                "Error pulse",
                on(s.error_flash),
                "Brief progress-bar feedback. Suppressed in blind mode.",
            ),
        ],
        _ => vec![
            (
                "Blind mode",
                on(s.blind),
                "Hide error colors, error feedback and live accuracy until results.",
            ),
            (
                "No backspace",
                on(s.no_backspace),
                "Disable character and word deletion. Cannot be combined with stop-on-error.",
            ),
            (
                "Stop on error",
                on(s.stop_on_error),
                "Wrong keys count as mistakes but do not advance the caret. Incompatible with blind/no-backspace.",
            ),
            (
                "Pace target",
                if s.pace == 0 {
                    "off".into()
                } else {
                    format!("{} wpm", s.pace)
                },
                "A second underline tracks a constant target speed.",
            ),
            (
                "Tab restart",
                on(s.tab_restart),
                "Opt in to restarting with Tab. Ctrl+R always restarts.",
            ),
        ],
    }
}
fn cycle<T: Copy + PartialEq>(value: &mut T, options: &[T], back: bool) {
    let i = options.iter().position(|x| x == value).unwrap_or(0);
    *value = options[(i + if back { options.len() - 1 } else { 1 }) % options.len()];
}
pub fn change(s: &mut Settings, group: usize, row: usize, back: bool) {
    match (group % 4, row) {
        (0, 0) => cycle(&mut s.seconds, &[15, 30, 60, 120], back),
        (0, 1) => cycle(&mut s.words, &[10, 25, 50, 100], back),
        (0, 2) => s.punctuation = !s.punctuation,
        (0, 3) => s.numbers = !s.numbers,
        (0, 4) => cycle(
            &mut s.difficulty,
            &[Difficulty::Normal, Difficulty::Expert, Difficulty::Master],
            back,
        ),
        (0, 5) => cycle(&mut s.daily_goal, &[1, 3, 5, 10, 20], back),
        (1, 0) => s.theme = (s.theme + if back { THEMES.len() - 1 } else { 1 }) % THEMES.len(),
        (1, 1) => cycle(&mut s.caret, &[0, 1, 2], back),
        (1, 2) => cycle(&mut s.line_gap, &[0, 1, 2], back),
        (1, 3) => cycle(
            &mut s.text_width,
            &[0, 50, 65, 80, 100, 120, 160, 200],
            back,
        ),
        (1, 4) => s.show_keyboard = !s.show_keyboard,
        (1, 5) => cycle(&mut s.keyboard_layout, &[0, 1, 2], back),
        (1, 6) => s.live_stats = !s.live_stats,
        (1, 7) => s.focus = !s.focus,
        (1, 8) => cycle(&mut s.accent, &[0, 1, 2, 3, 4, 5], back),
        (1, 9) => cycle(&mut s.panel_style, &[0, 1, 2, 3], back),
        (1, 10) => cycle(&mut s.key_style, &[0, 1, 2], back),
        (1, 11) => cycle(&mut s.graph_style, &[0, 1, 2], back),
        (2, 0) => cycle(&mut s.visible_lines, &[1, 3, 5, 7, 9], back),
        (2, 1) => s.bold_text = !s.bold_text,
        (2, 2) => s.large_results = !s.large_results,
        (2, 3) => s.celebrations = !s.celebrations,
        (2, 4) => s.error_flash = !s.error_flash,
        (3, 0) => {
            s.blind = !s.blind;
            if s.blind {
                s.stop_on_error = false;
            }
        }
        (3, 1) => {
            s.no_backspace = !s.no_backspace;
            if s.no_backspace {
                s.stop_on_error = false;
            }
        }
        (3, 2) => {
            s.stop_on_error = !s.stop_on_error;
            if s.stop_on_error {
                s.blind = false;
                s.no_backspace = false;
            }
        }
        (3, 3) => cycle(&mut s.pace, &[0, 30, 45, 60, 80, 100, 120, 150, 200], back),
        (3, 4) => s.tab_restart = !s.tab_restart,
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_settings_receive_safe_defaults() {
        let s: Settings = serde_json::from_str(r#"{"theme":2,"seconds":60}"#).unwrap();
        assert!(s.large_results);
        assert!(s.show_keyboard);
        assert_eq!(s.theme, 2);
        assert_eq!(s.daily_goal, 3);
    }
    #[test]
    fn incompatible_modifiers_are_resolved() {
        let mut s = Settings::default();
        change(&mut s, 3, 2, false);
        assert!(s.stop_on_error);
        change(&mut s, 3, 1, false);
        assert!(s.no_backspace);
        assert!(!s.stop_on_error);
        change(&mut s, 3, 2, false);
        assert!(!s.no_backspace);
        assert!(s.stop_on_error);
        change(&mut s, 3, 0, false);
        assert!(s.blind);
        assert!(!s.stop_on_error);
    }
    #[test]
    fn every_setting_cycles_safely() {
        let mut s = Settings::default();
        for group in 0..4 {
            for row in 0..count(group) {
                for _ in 0..20 {
                    change(&mut s, group, row, false);
                    s.sanitize();
                    assert_eq!(rows(&s, group).len(), count(group));
                }
            }
        }
    }
}
