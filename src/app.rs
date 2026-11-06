use crate::{
    content,
    engine::{Difficulty, Mode, Session},
    store::{Profile, Store},
};
use chrono::{Datelike, Local, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{io, time::Instant};
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Home,
    Typing,
    Results,
    Profile,
    Settings,
    Help,
    Keyboard,
    Themes,
    Import,
    ModeInfo,
}
pub struct App {
    pub profile: Profile,
    pub store: Store,
    pub screen: Screen,
    pub session: Session,
    pub selected: usize,
    pub setting: usize,
    pub custom: String,
    pub picker: crate::import::Picker,
    pub start: Option<Instant>,
    pub seed: u64,
    pub reward: u64,
    pub pb: bool,
    pub quit: bool,
    pub last_key: String,
    pub pressed: Vec<char>,
    pub settings_group: usize,
    pub profile_tab: usize,
    pub history_offset: usize,
    pub fixed_seed: Option<u64>,
    pub reflow: crate::text_layout::Reflow,
    pub last_input: Option<(char, bool, Instant)>,
    pub theme_cursor: usize,
    pub theme_columns: usize,
    pub previous_best: f64,
    pub previous_level: u64,
    pub unlocks: Vec<String>,
    pub quest_bonus: u64,
    pub milestone_bonus: u64,
    pub nav_hits: Vec<(ratatui::layout::Rect, Screen)>,
}
impl App {
    pub fn new(profile: Profile, store: Store, custom: String) -> Self {
        let settings = profile.settings.clone();
        let selected = Mode::ALL
            .iter()
            .position(|m| *m == settings.mode)
            .unwrap_or(0);
        Self {
            profile,
            store,
            screen: Screen::Home,
            session: Session::new(String::new(), settings.clone()),
            selected,
            setting: 0,
            custom,
            picker: Default::default(),
            start: None,
            seed: 0,
            reward: 0,
            pb: false,
            quit: false,
            last_key: "Press any key to inspect it".into(),
            pressed: vec![],
            settings_group: 0,
            profile_tab: 0,
            history_offset: 0,
            fixed_seed: None,
            reflow: Default::default(),
            last_input: None,
            theme_cursor: settings.theme,
            theme_columns: 3,
            previous_best: 0.,
            previous_level: 1,
            unlocks: vec![],
            quest_bonus: 0,
            milestone_bonus: 0,
            nav_hits: vec![],
        }
    }
    fn navigate(&mut self, screen: Screen) {
        if screen == Screen::Themes && self.screen != Screen::Themes {
            self.theme_cursor = self.profile.settings.theme;
        }
        self.screen = screen;
        self.nav_hits.clear();
        self.start = None;
    }
    pub fn mouse(&mut self, event: crossterm::event::MouseEvent) {
        use crossterm::event::{MouseButton, MouseEventKind};
        if self.screen == Screen::Typing || event.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }
        let point = ratatui::layout::Position::new(event.column, event.row);
        if let Some((_, screen)) = self.nav_hits.iter().find(|(rect, _)| rect.contains(point)) {
            self.navigate(*screen);
        }
    }
    pub fn open_import(&mut self) {
        let dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        if let Err(e) = self.picker.open(dir) {
            self.picker.error = e.to_string();
        }
        self.navigate(Screen::Import);
    }
    pub fn paste_path(&mut self, text: &str) {
        if self.screen == Screen::Import {
            self.picker.input.extend(
                text.chars()
                    .filter(|c| !c.is_control())
                    .take(4096usize.saturating_sub(self.picker.input.chars().count())),
            );
        }
    }
    fn import_key(&mut self, k: KeyEvent) {
        match k.code {
            KeyCode::Up => {
                self.picker.selected = self.picker.selected.saturating_sub(1);
                self.picker.input.clear();
            }
            KeyCode::Down => {
                self.picker.selected =
                    (self.picker.selected + 1).min(self.picker.entries.len().saturating_sub(1));
                self.picker.input.clear();
            }
            KeyCode::Backspace => {
                self.picker.input.pop();
            }
            KeyCode::Char('u') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                self.picker.input.clear()
            }
            KeyCode::Char(c) if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.picker.input.chars().count() < 4096 {
                    self.picker.input.push(c);
                }
            }
            KeyCode::Enter => {
                let path = self.picker.path();
                if path.is_dir() {
                    if let Err(e) = self.picker.open(path) {
                        self.picker.error = e.to_string();
                    }
                } else {
                    let loaded = content::read_text(&path).and_then(|text| {
                        let tmp = self.store.dir.join("import.tmp");
                        std::fs::write(&tmp, &text)?;
                        std::fs::rename(tmp, self.store.dir.join("import.txt"))?;
                        Ok(text)
                    });
                    match loaded {
                        Ok(text) => {
                            self.custom = text;
                            self.selected = 7;
                            self.launch(false);
                        }
                        Err(e) => self.picker.error = e.to_string(),
                    }
                }
            }
            _ => {}
        }
    }
    pub fn launch(&mut self, same: bool) {
        if !same && Mode::ALL[self.selected] == Mode::Custom && self.custom.is_empty() {
            self.open_import();
            return;
        }
        self.profile.settings.mode = Mode::ALL[self.selected];
        if !same {
            self.seed = self.fixed_seed.unwrap_or_else(rand::random);
        }
        let mut settings = if same {
            self.session.settings.clone()
        } else {
            self.profile.settings.clone()
        };
        if settings.mode == Mode::Daily {
            self.seed = Utc::now().date_naive().num_days_from_ce() as u64;
            settings.seconds = 60;
            settings.punctuation = false;
            settings.numbers = false;
            settings.difficulty = Difficulty::Normal;
            settings.blind = false;
            settings.no_backspace = false;
            settings.stop_on_error = false;
        }
        if settings.mode == Mode::Zen {
            settings.focus = true;
        }
        let text = if same {
            self.session.target.iter().collect()
        } else {
            content::generate(&settings, self.seed, &self.profile.weak(), &self.custom)
        };
        self.session = Session::new(text, settings);
        self.start = None;
        self.reflow.clear();
        self.last_input = None;
        self.screen = Screen::Typing;
    }
    pub fn tick(&mut self) -> io::Result<()> {
        if self.screen == Screen::Typing {
            if let Some(start) = self.start {
                self.session.tick(start.elapsed().as_secs_f64());
            }
            self.finish()?;
        }
        Ok(())
    }
    fn finish(&mut self) -> io::Result<()> {
        if self.session.finished && self.screen == Screen::Typing {
            if self.session.elapsed >= 0.1 {
                self.session
                    .pace_points
                    .push((self.session.elapsed, self.session.wpm()));
                self.session
                    .raw_points
                    .push((self.session.elapsed, self.session.raw()));
            }
            self.previous_best = self.profile.best(&self.session.settings);
            self.previous_level = self.profile.level();
            let today = Local::now().date_naive();
            let tracks = self.profile.milestones();
            let badges = self.profile.achievements.clone();
            let claims = self
                .profile
                .quest_claims
                .get(&today)
                .cloned()
                .unwrap_or_default();
            (self.reward, self.pb) = self.profile.record_on(&self.session, today);
            self.unlocks = self
                .profile
                .achievements
                .iter()
                .filter(|v| !badges.contains(v))
                .cloned()
                .collect();
            self.quest_bonus = self
                .profile
                .quests(today)
                .iter()
                .filter(|q| q.claimed && !claims.contains(&q.id.to_string()))
                .map(|q| q.reward)
                .sum();
            self.milestone_bonus = 0;
            for (before, after) in tracks.iter().zip(self.profile.milestones()) {
                let earned = after.tier() - before.tier();
                self.milestone_bonus += earned * after.reward;
                if earned > 0 {
                    self.unlocks
                        .push(format!("{} tier {}", after.name, after.tier()));
                }
            }
            self.store.save(&self.profile)?;
            self.screen = Screen::Results;
        }
        Ok(())
    }
    fn type_char(&mut self, c: char) {
        let good = self.session.target.get(self.session.typed.len()) == Some(&c);
        self.session.input(c);
        self.last_input = Some((c, good, Instant::now()));
    }
    fn retry_missed(&mut self) {
        let words = self.session.missed_words();
        if words.is_empty() {
            return;
        }
        let mut settings = self.profile.settings.clone();
        settings.mode = Mode::Custom;
        self.session = Session::new(words.join(" "), settings);
        self.start = None;
        self.reflow.clear();
        self.last_input = None;
        self.screen = Screen::Typing;
    }
    pub fn key(&mut self, k: KeyEvent) -> io::Result<()> {
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl && k.code == KeyCode::Char('c') {
            self.quit = true;
            return Ok(());
        }
        if k.code == KeyCode::Esc {
            self.navigate(Screen::Home);
            return Ok(());
        }
        if self.screen == Screen::Import && !matches!(k.code, KeyCode::F(_)) {
            self.import_key(k);
            return Ok(());
        }
        if self.screen != Screen::Typing {
            if k.code == KeyCode::Char('o') && self.screen != Screen::Keyboard && !ctrl {
                self.open_import();
                return Ok(());
            }
            let route = match k.code {
                KeyCode::Char('i') if self.screen != Screen::Keyboard && !ctrl => {
                    Some(Screen::ModeInfo)
                }
                KeyCode::F(1) => Some(Screen::Help),
                KeyCode::F(2) => Some(Screen::Home),
                KeyCode::F(3) => Some(Screen::Profile),
                KeyCode::F(4) => Some(Screen::Settings),
                KeyCode::F(5) => Some(Screen::Themes),
                KeyCode::F(6) => Some(Screen::Keyboard),
                KeyCode::Char('p') if self.screen != Screen::Keyboard && !ctrl => {
                    Some(Screen::Profile)
                }
                KeyCode::Char('s') if self.screen != Screen::Keyboard && !ctrl => {
                    Some(Screen::Settings)
                }
                KeyCode::Char('t') if self.screen != Screen::Keyboard && !ctrl => {
                    Some(Screen::Themes)
                }
                KeyCode::Char('b') if self.screen != Screen::Keyboard && !ctrl => {
                    Some(Screen::Keyboard)
                }
                KeyCode::Char('?') if self.screen != Screen::Keyboard && !ctrl => {
                    Some(Screen::Help)
                }
                _ => None,
            };
            if let Some(route) = route {
                self.navigate(route);
                return Ok(());
            }
            if self.screen != Screen::Keyboard && !ctrl && k.code == KeyCode::Char('g') {
                self.profile_tab = 5;
                self.history_offset = 0;
                self.navigate(Screen::Profile);
                return Ok(());
            }
            if self.screen != Screen::Keyboard
                && self.screen != Screen::Results
                && !ctrl
                && k.code == KeyCode::Char('r')
            {
                self.profile_tab = 1;
                self.history_offset = 0;
                self.navigate(Screen::Profile);
                return Ok(());
            }
        }
        if self.screen == Screen::Keyboard {
            self.last_key = format!("{:?}  {:?}", k.code, k.modifiers);
            if let KeyCode::Char(c) = k.code
                && !self.pressed.contains(&c)
            {
                self.pressed.push(c);
            }
            return Ok(());
        }
        match self.screen {
            Screen::Home | Screen::ModeInfo => match k.code {
                KeyCode::Char('q') => self.quit = true,
                KeyCode::Down | KeyCode::Char('j') => {
                    self.selected = (self.selected + 1) % Mode::ALL.len()
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.selected = (self.selected + Mode::ALL.len() - 1) % Mode::ALL.len()
                }
                KeyCode::Enter => self.launch(false),
                KeyCode::Char(c @ '1'..='9') => {
                    self.selected = c as usize - '1' as usize;
                    self.launch(false);
                }
                _ => {}
            },
            Screen::Typing => {
                if ctrl {
                    match k.code {
                        KeyCode::Char('r') => self.launch(false),
                        KeyCode::Char('w') => self.session.delete_word(),
                        KeyCode::Char('d')
                            if self.session.settings.mode.endless() && self.session.started =>
                        {
                            self.session.finished = true;
                        }
                        _ => {}
                    }
                } else {
                    match k.code {
                        KeyCode::Tab
                            if matches!(self.session.settings.mode, Mode::Code | Mode::Custom) =>
                        {
                            if self.session.indent() > 0 && self.start.is_none() {
                                self.start = Some(Instant::now());
                            }
                        }
                        KeyCode::Tab if self.session.settings.tab_restart => self.launch(false),
                        KeyCode::Backspace => self.session.backspace(),
                        KeyCode::Char(c) => {
                            if self.start.is_none() {
                                self.start = Some(Instant::now());
                            }
                            self.type_char(c);
                        }
                        KeyCode::Enter
                            if self.session.target.get(self.session.typed.len()) == Some(&'\n') =>
                        {
                            if self.start.is_none() {
                                self.start = Some(Instant::now());
                            }
                            self.type_char('\n');
                        }
                        _ => {}
                    }
                }
                if (self.session.timed() || self.session.settings.mode.endless())
                    && self
                        .session
                        .target
                        .len()
                        .saturating_sub(self.session.typed.len())
                        < 100
                {
                    let text = content::generate(
                        &self.session.settings,
                        self.seed.wrapping_add(self.session.target.len() as u64),
                        &self.profile.weak(),
                        "",
                    );
                    self.session.target.push(' ');
                    self.session.target.extend(text.chars());
                }
                self.finish()?;
            }
            Screen::Results => match k.code {
                KeyCode::Enter => self.launch(false),
                KeyCode::Char('r') => self.launch(true),
                KeyCode::Char('w') => {
                    self.selected = 6;
                    self.launch(false);
                }
                KeyCode::Char('m') => self.retry_missed(),
                _ => {}
            },
            Screen::Settings => match k.code {
                KeyCode::Tab | KeyCode::BackTab => {
                    self.settings_group =
                        (self.settings_group + if k.code == KeyCode::BackTab { 3 } else { 1 }) % 4;
                    self.setting = 0;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.setting = (self.setting + 1) % crate::settings::count(self.settings_group)
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.setting = (self.setting + crate::settings::count(self.settings_group) - 1)
                        % crate::settings::count(self.settings_group)
                }
                KeyCode::Right | KeyCode::Left | KeyCode::Enter | KeyCode::Char(' ') => {
                    crate::settings::change(
                        &mut self.profile.settings,
                        self.settings_group,
                        self.setting,
                        k.code == KeyCode::Left,
                    );
                    self.store.save(&self.profile)?;
                }
                _ => {}
            },
            Screen::Themes => match k.code {
                KeyCode::Home => self.theme_cursor = 0,
                KeyCode::End => self.theme_cursor = crate::theme::THEMES.len() - 1,
                KeyCode::PageUp => {
                    self.theme_cursor = self.theme_cursor.saturating_sub(self.theme_columns * 3)
                }
                KeyCode::PageDown => {
                    self.theme_cursor = (self.theme_cursor + self.theme_columns * 3)
                        .min(crate::theme::THEMES.len() - 1)
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.theme_cursor = self.theme_cursor.saturating_sub(1)
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.theme_cursor = (self.theme_cursor + 1).min(crate::theme::THEMES.len() - 1)
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.theme_cursor = self.theme_cursor.saturating_sub(self.theme_columns)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.theme_cursor =
                        (self.theme_cursor + self.theme_columns).min(crate::theme::THEMES.len() - 1)
                }
                KeyCode::Enter => {
                    self.profile.settings.theme = self.theme_cursor;
                    self.store.save(&self.profile)?;
                }
                _ => {}
            },
            Screen::Profile => match k.code {
                KeyCode::Tab => {
                    self.profile_tab = (self.profile_tab + 1) % 6;
                    self.history_offset = 0;
                }
                KeyCode::BackTab => {
                    self.profile_tab = (self.profile_tab + 5) % 6;
                    self.history_offset = 0;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.history_offset = (self.history_offset + 1).min(match self.profile_tab {
                        4 => crate::progression::BADGES.len().saturating_sub(1),
                        5 => crate::progression::TRACK_COUNT - 1,
                        1 => self.profile.records().len().saturating_sub(1),
                        _ => self.profile.history.len().saturating_sub(1),
                    })
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.history_offset = self.history_offset.saturating_sub(1)
                }
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_app(name: &str) -> App {
        let dir = std::env::temp_dir().join(format!("typedef-{name}-{}", std::process::id()));
        let store = Store::open(Some(dir)).unwrap();
        App::new(Profile::default(), store, "hello world".into())
    }
    fn cleanup(a: App) {
        let dir = a.store.dir.clone();
        drop(a);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn endless_and_zen_continue_past_initial_content() {
        for mode in [Mode::Endless, Mode::Zen] {
            let mut a = test_app(if mode == Mode::Zen {
                "zen-long"
            } else {
                "endless-long"
            });
            a.selected = Mode::ALL.iter().position(|m| *m == mode).unwrap();
            a.launch(false);
            let initial = a.session.target.len();
            for _ in 0..initial + 200 {
                let c = a.session.target[a.session.typed.len()];
                a.key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
                    .unwrap();
            }
            assert!(a.session.target.len() > initial);
            assert!(!a.session.finished);
            assert!(a.profile.history.is_empty());
            a.key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL))
                .unwrap();
            assert_eq!(a.screen, Screen::Results);
            assert_eq!(a.profile.history.len(), 1);
            cleanup(a);
        }
    }
    #[test]
    fn in_app_import_keeps_previous_text_on_error_and_loads_without_restart() {
        let mut a = test_app("import");
        let file = a.store.dir.join("example with spaces.rs");
        std::fs::write(&file, "fn main() {\n\tprintln!(\"hi\");\n}").unwrap();
        a.open_import();
        a.paste_path("missing-file.txt");
        a.key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
            .unwrap();
        assert_eq!(a.screen, Screen::Import);
        assert_eq!(a.custom, "hello world");
        assert!(!a.picker.error.is_empty());
        a.picker.input.clear();
        a.paste_path(&file.to_string_lossy());
        a.key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
            .unwrap();
        assert_eq!(a.screen, Screen::Typing);
        assert!(a.custom.contains("    println!"));
        assert_eq!(
            std::fs::read_to_string(a.store.dir.join("import.txt")).unwrap(),
            a.custom
        );
        let original = a.custom.clone();
        a.paste_path("unexpected paste");
        assert_eq!(a.custom, original);
        cleanup(a);
    }
    #[test]
    fn abandon_never_records_and_restart_clears_clock() {
        let mut a = test_app("abandon");
        a.launch(false);
        a.key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE))
            .unwrap();
        a.key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
            .unwrap();
        assert!(a.profile.history.is_empty());
        a.launch(false);
        assert!(a.start.is_none());
        assert!(!a.session.started);
        cleanup(a);
    }
    #[test]
    fn finished_run_is_persisted_exactly_once() {
        let mut a = test_app("finish");
        a.selected = 7;
        a.launch(false);
        for c in "hello world".chars() {
            a.key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
                .unwrap();
        }
        a.tick().unwrap();
        a.tick().unwrap();
        assert!(a.screen == Screen::Results);
        assert_eq!(a.profile.history.len(), 1);
        assert_eq!(a.store.load().unwrap().history.len(), 1);
        cleanup(a);
    }
    #[test]
    fn daily_ignores_personal_modifiers_and_repeats() {
        let mut a = test_app("daily");
        a.selected = 5;
        a.profile.settings.seconds = 15;
        a.profile.settings.punctuation = true;
        a.launch(false);
        let first = a.session.target.clone();
        a.launch(false);
        assert_eq!(first, a.session.target);
        assert_eq!(a.session.settings.seconds, 60);
        assert!(!a.session.settings.punctuation);
        cleanup(a);
    }
    #[test]
    fn theme_preview_cancel_and_apply_are_distinct() {
        let mut a = test_app("theme-preview");
        let press = |a: &mut App, code| a.key(KeyEvent::new(code, KeyModifiers::NONE)).unwrap();
        press(&mut a, KeyCode::Char('t'));
        a.theme_columns = 2;
        press(&mut a, KeyCode::Down);
        assert_eq!(a.theme_cursor, 2);
        assert_eq!(a.profile.settings.theme, 0);
        press(&mut a, KeyCode::Esc);
        press(&mut a, KeyCode::Char('t'));
        assert_eq!(a.theme_cursor, 0);
        press(&mut a, KeyCode::Right);
        press(&mut a, KeyCode::Enter);
        assert!(a.screen == Screen::Themes);
        assert_eq!(a.store.load().unwrap().settings.theme, 1);
        press(&mut a, KeyCode::Right);
        press(&mut a, KeyCode::Esc);
        assert_eq!(a.profile.settings.theme, 1);
        press(&mut a, KeyCode::Char('t'));
        press(&mut a, KeyCode::End);
        press(&mut a, KeyCode::Enter);
        assert_eq!(
            a.store.load().unwrap().settings.theme,
            crate::theme::THEMES.len() - 1
        );
        press(&mut a, KeyCode::Home);
        assert_eq!(a.theme_cursor, 0);
        press(&mut a, KeyCode::Char('r'));
        assert!(a.screen == Screen::Profile);
        assert_eq!(a.profile_tab, 1);
        cleanup(a);
    }
    #[test]
    fn every_non_typing_screen_can_navigate_directly() {
        let mut a = test_app("navigation");
        let routes = [
            (KeyCode::F(1), Screen::Help),
            (KeyCode::F(2), Screen::Home),
            (KeyCode::F(3), Screen::Profile),
            (KeyCode::F(4), Screen::Settings),
            (KeyCode::F(5), Screen::Themes),
            (KeyCode::F(6), Screen::Keyboard),
        ];
        for source in [
            Screen::Home,
            Screen::Profile,
            Screen::Settings,
            Screen::Themes,
            Screen::Results,
            Screen::Help,
            Screen::Keyboard,
        ] {
            for (code, destination) in routes {
                a.screen = source;
                a.key(KeyEvent::new(code, KeyModifiers::NONE)).unwrap();
                assert_eq!(a.screen, destination);
            }
            if source != Screen::Keyboard {
                for (code, destination) in [
                    ('p', Screen::Profile),
                    ('s', Screen::Settings),
                    ('t', Screen::Themes),
                    ('b', Screen::Keyboard),
                    ('?', Screen::Help),
                ] {
                    a.screen = source;
                    a.key(KeyEvent::new(KeyCode::Char(code), KeyModifiers::NONE))
                        .unwrap();
                    assert_eq!(a.screen, destination);
                }
            }
        }
        a.navigate(Screen::Themes);
        a.theme_cursor = 27;
        a.key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE))
            .unwrap();
        a.key(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE))
            .unwrap();
        assert_eq!(a.theme_cursor, a.profile.settings.theme);
        a.key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE))
            .unwrap();
        assert_eq!(a.profile_tab, 5);
        a.key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE))
            .unwrap();
        assert_eq!(a.profile_tab, 0);
        a.key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT))
            .unwrap();
        assert_eq!(a.profile_tab, 5);
        cleanup(a);
    }
    #[test]
    fn navigation_does_not_steal_typing_or_key_lab_letters() {
        let mut a = test_app("literal-navigation");
        a.session = Session::new("pstbgr?x".into(), crate::engine::Settings::default());
        a.screen = Screen::Typing;
        for c in "pstbgr?".chars() {
            a.key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
                .unwrap();
        }
        assert_eq!(a.screen, Screen::Typing);
        assert_eq!(a.session.typed.iter().collect::<String>(), "pstbgr?");
        for key in 1..=6 {
            a.key(KeyEvent::new(KeyCode::F(key), KeyModifiers::NONE))
                .unwrap();
        }
        assert_eq!(a.screen, Screen::Typing);
        a.screen = Screen::Keyboard;
        for c in "pstbgr?".chars() {
            a.key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
                .unwrap();
        }
        assert_eq!(a.screen, Screen::Keyboard);
        assert_eq!(a.pressed.iter().collect::<String>(), "pstbgr?");
        a.key(KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE))
            .unwrap();
        assert_eq!(a.screen, Screen::Themes);
        cleanup(a);
    }
}

#[cfg(test)]
mod replay_tests {
    use super::*;
    #[test]
    fn replay_preserves_missed_word_text() {
        let dir = std::env::temp_dir().join(format!("typedef-missed-{}", std::process::id()));
        let store = Store::open(Some(dir.clone())).unwrap();
        let mut a = App::new(Profile::default(), store, "hello world".into());
        a.selected = 7;
        a.launch(false);
        a.type_char('x');
        a.retry_missed();
        assert_eq!(a.session.target.iter().collect::<String>(), "hello");
        a.launch(true);
        assert_eq!(a.session.target.iter().collect::<String>(), "hello");
        drop(a);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
