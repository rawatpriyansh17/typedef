use crate::{
    app::{App, Screen},
    engine::Mode,
    theme::{self, Theme},
};
use chrono::{Datelike, Duration, Local};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        Axis, Block, BorderType, Borders, Chart, Dataset, Gauge, GraphType, Paragraph, Sparkline,
        Wrap,
    },
};
fn inner(r: Rect) -> Rect {
    r.inner(Margin::new(if r.width >= 42 { 2 } else { 1 }, 1))
}
fn panel(title: &str, p: Theme) -> Block<'static> {
    let (borders, kind) = match p.panel_style {
        1 => (Borders::ALL, BorderType::Plain),
        2 => (Borders::ALL, BorderType::Double),
        3 => (Borders::NONE, BorderType::Plain),
        _ => (Borders::ALL, BorderType::Rounded),
    };
    Block::default()
        .borders(borders)
        .border_type(kind)
        .border_style(Style::default().fg(p.muted))
        .title(format!(" {title} "))
        .style(Style::default().bg(p.panel))
}

fn text(f: &mut Frame, r: Rect, lines: Vec<Line<'_>>, p: Theme) {
    f.render_widget(Paragraph::new(lines).fg(p.fg).wrap(Wrap { trim: false }), r);
}
fn center(r: Rect, width: u16) -> Rect {
    let w = width.min(r.width);
    Rect::new(r.x + (r.width - w) / 2, r.y, w, r.height)
}
fn label(f: &mut Frame, r: Rect, s: String, p: Theme) {
    f.render_widget(Paragraph::new(s).fg(p.muted), r);
}
pub fn draw(f: &mut Frame, a: &mut App) {
    let p = theme::customized(
        if a.screen == Screen::Themes {
            a.theme_cursor
        } else {
            a.profile.settings.theme
        },
        &a.profile.settings,
    );
    a.nav_hits.clear();
    let area = f.area();
    f.render_widget(Block::default().bg(p.bg).fg(p.fg), area);
    if area.width < 60 || area.height < 20 {
        text(
            f,
            inner(area),
            vec![
                Line::from("typedef").fg(p.accent),
                "".into(),
                "Resize to at least 60 × 20.".into(),
                "Ctrl+C exits.".into(),
            ],
            p,
        );
        return;
    }
    let canvas = area.inner(Margin::new(2, 1));
    let parts = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(1),
    ])
    .split(canvas);
    let focused = a.screen == Screen::Typing && a.session.settings.focus;
    if !focused {
        header(f, parts[0], a, p);
    }
    match a.screen {
        Screen::Home => home(f, parts[1], a, p),
        Screen::Typing => typing(f, parts[1], a, p),
        Screen::Results => results(f, parts[1], a, p),
        Screen::Profile => profile(f, parts[1], a, p),
        Screen::Settings => settings(f, parts[1], a, p),
        Screen::Themes => {
            a.theme_columns = (parts[1].width as usize / 32).clamp(2, 6);
            themes(f, parts[1], a, p);
        }
        Screen::Keyboard => keyboard_lab(f, parts[1], a, p),
        Screen::Help => help(f, parts[1], p),
        Screen::Import => import_picker(f, parts[1], a, p),
        Screen::ModeInfo => mode_info(f, parts[1], a, p),
    }
    let hint = match a.screen {
        Screen::Home if area.width < 80 => "↑↓ mode  ↵ start  i info  o file  s settings  t themes",
        Screen::Typing if a.session.settings.mode.endless() => {
            "ctrl+d finish & save   ctrl+r restart   esc abandon"
        }
        Screen::Import if area.width < 80 => {
            "↑↓ browse  ↵ open  paste path  ctrl+u clear  esc home"
        }
        Screen::Import => "↑↓ browse   enter open   type / paste path   ctrl+u clear   esc home",
        Screen::ModeInfo => "↑↓ mode   enter start   o load file   esc home",
        Screen::Typing if area.width < 80 => "ctrl+r restart   ctrl+w erase word   esc home",
        Screen::Results if area.width < 80 => {
            "↵ next   r retry   m missed   w weak keys   p profile"
        }
        Screen::Home => "↑↓ mode   enter start   i mode info   o load file   g journey   t themes",
        Screen::Typing => "ctrl+r restart   ctrl+w erase word   esc home   ctrl+c quit",
        Screen::Results => "enter next   r retry   m missed words   w weak keys   p profile",
        Screen::Settings => "tab category   ↑↓ setting   ←→ change   esc home",
        Screen::Themes => "arrows / pg↑↓ browse   enter apply   esc discard preview",
        Screen::Profile => "tab view   ↑↓ scroll   g journey   s settings   t themes",
        _ => "esc home   ctrl+c quit",
    };
    if !focused {
        f.render_widget(
            Paragraph::new(hint)
                .fg(p.muted)
                .alignment(Alignment::Center),
            Rect {
                y: parts[2].y,
                height: 1,
                ..parts[2]
            },
        );
    }
}
fn header(f: &mut Frame, r: Rect, a: &mut App, p: Theme) {
    // A two-line TD monogram: ordinary Unicode blocks, no patched font or image protocol.
    f.render_widget(
        Paragraph::new(vec![
            Line::from("▀█▀ █▀▄").fg(p.accent),
            Line::from(" █  █▄▀").fg(p.secondary),
        ]),
        Rect::new(r.x, r.y, 7, 2),
    );
    f.render_widget(
        Paragraph::new("typedef").fg(p.accent).bold(),
        Rect::new(r.x + 9, r.y, 7, 1),
    );
    let nav = Rect::new(r.x + 19, r.y, r.width.saturating_sub(19), 2);
    let destinations = [
        (Screen::Home, "F2 home"),
        (Screen::Profile, "F3 profile"),
        (Screen::Settings, "F4 settings"),
        (Screen::Themes, "F5 themes"),
        (Screen::Keyboard, "F6 keys"),
        (Screen::Help, "F1 help"),
    ];
    let count = if nav.width >= 85 { 6 } else { 4 };
    let two_rows = nav.width < 51;
    let active = if matches!(a.screen, Screen::Typing | Screen::Results) {
        Screen::Home
    } else {
        a.screen
    };
    let mut x = nav.x;
    for (i, (screen, label)) in destinations.iter().take(count).enumerate() {
        let rect = if two_rows {
            Rect::new(
                nav.x + (i % 2) as u16 * (nav.width / 2),
                nav.y + (i / 2) as u16,
                nav.width / 2,
                1,
            )
        } else {
            let width = label.len() as u16 + 3;
            let rect = Rect::new(x, nav.y, width, 1);
            x += width;
            rect
        };
        let style = if a.screen != Screen::Typing && *screen == active {
            Style::default().fg(p.on_accent()).bg(p.accent).bold()
        } else {
            Style::default().fg(p.muted)
        };
        f.render_widget(Paragraph::new(format!(" {label} ")).style(style), rect);
        if a.screen != Screen::Typing {
            a.nav_hits.push((rect, *screen));
        }
    }
}
const DESCRIPTIONS: [&str; 9] = [
    "beat the clock",
    "set your distance",
    "punctuation & prose",
    "∞ unlimited + live stats",
    "an open-ended session",
    "today's shared seed",
    "practice your weak keys",
    "your file or cached text",
    "syntax + tab indentation",
];
fn home(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    let columns = Layout::horizontal([
        Constraint::Length(if r.width >= 90 { 26 } else { 18 }),
        Constraint::Min(20),
    ])
    .spacing(2)
    .split(r);
    f.render_widget(panel("PLAY", p), columns[0]);
    let rail = inner(columns[0]);
    let roomy = rail.height >= 23;
    let mut lines = vec![Line::from("SELECT A MODE").fg(p.muted), Line::from("")];
    for (i, mode) in Mode::ALL.iter().enumerate() {
        let chosen = i == a.selected;
        lines.push(
            Line::from(format!(
                "{:<width$}",
                format!(
                    "{} {:02} {} {}",
                    if chosen { "▶" } else { " " },
                    i + 1,
                    mode.icon(),
                    mode.label()
                ),
                width = rail.width as usize
            ))
            .style(if chosen {
                Style::default().bg(p.accent).fg(p.on_accent()).bold()
            } else {
                Style::default().fg(p.fg)
            }),
        );
        if roomy {
            lines.push(Line::from(""));
        }
    }
    if rail.height >= 15 {
        lines.push(Line::from(""));
        lines.push(Line::from(DESCRIPTIONS[a.selected]).fg(p.muted));
    }
    text(f, rail, lines, p);
    if rail.height >= 28 {
        let unlocked = a.profile.achievements.len();
        let mut stamps = vec![];
        for i in 0..crate::progression::BADGES.len() {
            stamps.push(Span::styled(
                if i < unlocked { "◆ " } else { "◇ " },
                Style::default().fg(if i < unlocked { p.accent } else { p.muted }),
            ));
        }
        text(
            f,
            Rect::new(rail.x, rail.bottom() - 6, rail.width, 6),
            vec![
                Line::from(format!("BADGES  {unlocked:02} / 16")).fg(p.secondary),
                Line::from(""),
                Line::from(stamps),
                Line::from(format!(
                    "∞ {} tiers · g journey",
                    a.profile.milestone_tiers()
                ))
                .fg(p.secondary),
            ],
            p,
        );
    }
    if r.width >= 136 && r.height >= 22 {
        let rows = Layout::vertical([Constraint::Length(9), Constraint::Min(12)])
            .spacing(1)
            .split(columns[1]);
        let top = Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
            .spacing(2)
            .split(rows[0]);
        let bottom = Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
            .spacing(2)
            .split(rows[1]);
        record_card(f, top[0], a, p);
        career(f, top[1], a, p);
        quests(f, bottom[0], a, p);
        journey_panel(f, bottom[1], a, p);
    } else {
        let areas = Layout::vertical([
            Constraint::Length(if r.height >= 25 {
                9
            } else if r.height >= 18 {
                7
            } else {
                5
            }),
            Constraint::Length(if r.height >= 25 {
                8
            } else if r.height >= 21 {
                5
            } else {
                4
            }),
            Constraint::Min(5),
        ])
        .spacing(u16::from(r.height >= 17))
        .split(columns[1]);
        record_card(f, areas[0], a, p);
        career(f, areas[1], a, p);
        quests(f, areas[2], a, p);
    }
}
fn record_card(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    let mut settings = a.profile.settings.clone();
    settings.mode = Mode::ALL[a.selected];
    if settings.mode == Mode::Daily {
        settings.seconds = 60;
        settings.punctuation = false;
        settings.numbers = false;
        settings.blind = false;
        settings.no_backspace = false;
        settings.stop_on_error = false;
        settings.difficulty = crate::engine::Difficulty::Normal;
    }
    let best = a.profile.best(&settings);
    let category = crate::store::Profile::category(&settings);
    let record = a
        .profile
        .history
        .iter()
        .filter(|rec| !rec.failed && rec.category == category)
        .max_by(|a, b| a.wpm.total_cmp(&b.wpm));
    f.render_widget(
        panel("PR / PERSONAL RECORD", p).border_style(Style::default().fg(p.accent)),
        r,
    );
    let hero = inner(r);
    let cells = Layout::horizontal([
        Constraint::Length(if hero.height >= 5 {
            22.min(hero.width / 2)
        } else {
            8
        }),
        Constraint::Min(8),
    ])
    .spacing(2)
    .split(hero);
    big_number(
        f,
        cells[0],
        &if record.is_some() {
            format!("{best:.0}")
        } else {
            "—".into()
        },
        p.accent,
    );
    let delta = a.profile.recent_delta(&settings);
    let mut info = vec![
        Line::from(format!(
            "{}  /  {}",
            settings.mode.label(),
            if matches!(settings.mode, Mode::Time | Mode::Daily) {
                format!("{}s", settings.seconds)
            } else if matches!(settings.mode, Mode::Words | Mode::Weak) {
                format!("{} words", settings.words)
            } else {
                "text practice".into()
            }
        ))
        .fg(p.accent)
        .bold(),
        Line::from("words per minute").fg(p.muted),
    ];
    if let Some(record) = record {
        info.push(Line::from(format!("{:.1}% accuracy", record.accuracy)));
        info.push(Line::from(format!("set {}", record.date.format("%d %b %Y"))).fg(p.muted));
    } else {
        info.push(Line::from("No record yet").fg(p.fg));
    }
    if let Some(change) = delta {
        info.push(Line::from(format!("{change:+.1} wpm · last 5 vs prior 5")).fg(p.secondary));
    }
    text(f, cells[1], info, p);
    if hero.height >= 7 && record.is_some() {
        let data: Vec<_> = a
            .profile
            .history
            .iter()
            .rev()
            .filter(|rec| !rec.failed && rec.category == category)
            .take(30)
            .map(|rec| (rec.wpm * 10.) as u64)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        let plot = Rect::new(
            cells[1].x + 8,
            hero.y + 6,
            cells[1].width.saturating_sub(8).min(30),
            1,
        );
        label(f, Rect::new(cells[1].x, plot.y, 7, 1), "recent".into(), p);
        f.render_widget(
            Sparkline::default()
                .data(&data)
                .style(Style::default().fg(p.secondary)),
            plot,
        );
    }
}

const DIGITS: [[&str; 5]; 11] = [
    ["█████", "█   █", "█   █", "█   █", "█████"],
    ["  ▄█ ", "   █ ", "   █ ", "   █ ", "  ███"],
    ["█████", "    █", "█████", "█    ", "█████"],
    ["█████", "    █", " ████", "    █", "█████"],
    ["█   █", "█   █", "█████", "    █", "    █"],
    ["█████", "█    ", "█████", "    █", "█████"],
    ["█████", "█    ", "█████", "█   █", "█████"],
    ["█████", "    █", "   █ ", "  █  ", "  █  "],
    ["█████", "█   █", "█████", "█   █", "█████"],
    ["█████", "█   █", "█████", "    █", "█████"],
    ["     ", "     ", "━━━━━", "     ", "     "],
];
fn big_number(f: &mut Frame, r: Rect, value: &str, color: Color) {
    if r.height < 5 || r.width < (value.chars().count() * 7).saturating_sub(2) as u16 {
        f.render_widget(Paragraph::new(value.to_owned()).fg(color).bold(), r);
        return;
    }
    let lines: Vec<_> = (0..5)
        .map(|row| {
            Line::from(
                value
                    .chars()
                    .map(|c| DIGITS[c.to_digit(10).map(|n| n as usize).unwrap_or(10)][row])
                    .collect::<Vec<_>>()
                    .join("  "),
            )
            .fg(color)
        })
        .collect();
    f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), r);
}
fn segmented(f: &mut Frame, r: Rect, ratio: f64, p: Theme) {
    let n = (r.width / 2).max(1) as usize;
    let lit = (ratio.clamp(0., 1.) * n as f64).floor() as usize;
    f.render_widget(
        Paragraph::new(Line::from(
            (0..n)
                .map(|i| {
                    Span::styled(
                        "▰ ",
                        Style::default().fg(if i < lit { p.accent } else { p.muted }),
                    )
                })
                .collect::<Vec<_>>(),
        )),
        r,
    );
}
fn career(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    f.render_widget(panel("CAREER", p), r);
    let body = inner(r);
    let level = a.profile.level();
    let tier = crate::progression::rank(level);
    let columns = Layout::horizontal([
        Constraint::Length(if body.width >= 38 && body.height >= 5 {
            10
        } else {
            0
        }),
        Constraint::Min(10),
    ])
    .split(body);
    if columns[0].width > 0 {
        text(
            f,
            columns[0],
            vec![
                "   ◆   ".into(),
                "  ╱ ╲  ".into(),
                Line::from(format!(" ◇ {:02} ◇", tier + 1))
                    .fg(p.accent)
                    .bold(),
                "  ╲ ╱  ".into(),
                "   ◆   ".into(),
            ],
            p,
        );
    }
    let right = columns[1];
    text(
        f,
        right,
        vec![
            Line::from(format!(
                "LV {level:02}   {}",
                a.profile.rank().to_uppercase()
            ))
            .fg(p.accent)
            .bold(),
            Line::from(format!(
                "{} / 500 XP  ·  {} day streak",
                a.profile.xp % 500,
                a.profile.streak()
            ))
            .fg(p.muted),
        ],
        p,
    );
    if right.height >= 3 {
        segmented(
            f,
            Rect::new(right.x, right.y + 2, right.width, 1),
            (a.profile.xp % 500) as f64 / 500.,
            p,
        );
    }
    if right.height >= 5 {
        label(
            f,
            Rect::new(right.x, right.y + 4, right.width, 1),
            a.profile.next_rank(),
            p,
        );
    }
}
fn quests(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    f.render_widget(panel("DAILY QUESTS", p), r);
    let body = inner(r);
    let quests = a.profile.quests(Local::now().date_naive());
    if body.height >= 6 && body.width >= 54 {
        let cols = Layout::horizontal([Constraint::Ratio(1, 3); 3])
            .spacing(1)
            .split(body);
        for (i, q) in quests.iter().enumerate() {
            text(
                f,
                cols[i],
                vec![
                    Line::from(format!("{} {}", if q.claimed { "◆" } else { "◇" }, q.id))
                        .fg(p.accent),
                    Line::from(if q.claimed {
                        format!("{} XP earned", q.reward)
                    } else {
                        format!("+{} XP", q.reward)
                    })
                    .fg(p.secondary),
                    Line::from(format!("{} / {}", q.current, q.target)),
                    Line::from(q.label).fg(p.muted),
                ],
                p,
            );
            segmented(
                f,
                Rect {
                    y: cols[i].y + 5,
                    height: 1,
                    ..cols[i]
                },
                q.current as f64 / q.target as f64,
                p,
            );
        }
        if body.height >= 10 {
            let today = Local::now().date_naive();
            let activity = a.profile.activity();
            let count = activity.get(&today).copied().unwrap_or(0);
            let mut week = vec![Span::styled("LAST 7 DAYS   ", Style::default().fg(p.muted))];
            for ago in (0..7).rev() {
                let day = today - Duration::days(ago);
                let done = activity.get(&day).copied().unwrap_or(0) > 0;
                week.push(Span::styled(
                    format!("{} {}  ", day.format("%a"), if done { "◆" } else { "◇" }),
                    Style::default().fg(if done { p.accent } else { p.muted }),
                ));
            }
            text(
                f,
                Rect::new(body.x, body.y + 8, body.width, 2),
                vec![
                    Line::from(week),
                    Line::from(format!(
                        "TODAY {count}/{} sessions · qualifying: ≥10s, ≥25 keys",
                        a.profile.settings.daily_goal
                    ))
                    .fg(p.muted),
                ],
                p,
            );
        }
    } else {
        let lines = quests
            .iter()
            .map(|q| {
                Line::from(format!(
                    "{} {:<10} {:>3}/{:<3}  +{} XP",
                    if q.claimed { "◆" } else { "◇" },
                    q.id,
                    q.current,
                    q.target,
                    q.reward
                ))
                .fg(if q.claimed { p.accent } else { p.muted })
            })
            .collect();
        text(f, body, lines, p);
    }
}
fn typing(f: &mut Frame, r: Rect, a: &mut App, p: Theme) {
    let s = &a.session;
    let focus = s.settings.focus;
    let keyboard_height = if s.settings.show_keyboard && !focus && r.height >= 16 {
        if r.height >= 25 { 13 } else { 7 }
    } else {
        0
    };
    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Min(4),
        Constraint::Length(keyboard_height),
    ])
    .split(r);
    let status = if s.settings.mode.endless() {
        format!("∞ {:.0}s", s.elapsed)
    } else if s.timed() {
        format!("{:.0}s", (s.settings.seconds as f64 - s.elapsed).ceil())
    } else {
        format!("{} / {}", s.typed.len(), s.target.len())
    };
    let mut spans = vec![
        Span::styled(status, Style::default().fg(p.accent).bold()),
        Span::styled(
            format!("    {}", s.settings.mode.label()),
            Style::default().fg(p.muted),
        ),
    ];
    if s.settings.live_stats && !focus {
        spans.push(Span::styled(
            format!("     {:.0} wpm", s.wpm()),
            Style::default().fg(p.fg),
        ));
        if !s.settings.blind {
            spans.push(Span::styled(
                format!("     {:.1}%", s.accuracy()),
                Style::default().fg(p.muted),
            ));
        }
    }
    if s.settings.live_stats && !s.settings.blind && !focus && r.width >= 90 {
        spans.push(Span::styled(
            format!("    combo {}", s.combo),
            Style::default().fg(p.muted),
        ));
    }
    if s.settings.pace > 0 && !focus {
        spans.push(Span::styled(
            format!("    pace {}", s.settings.pace),
            Style::default().fg(p.secondary),
        ));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), rows[0]);
    if s.settings.mode == Mode::Endless {
        let words = s.typed.iter().filter(|c| **c == ' ').count();
        f.render_widget(
            Paragraph::new(format!(
                "∞ LAP {:02}  ·  {} words  ·  Ctrl+D to finish",
                words / 50 + 1,
                words
            ))
            .fg(p.secondary)
            .alignment(Alignment::Right),
            Rect::new(rows[0].x, rows[0].y + 1, rows[0].width, 1),
        );
    }
    let flash = s.settings.error_flash
        && !s.settings.blind
        && a.last_input
            .is_some_and(|(_, good, t)| !good && t.elapsed().as_millis() < 130);
    f.render_widget(
        Gauge::default().ratio(s.progress()).label("").gauge_style(
            Style::default()
                .fg(if flash { p.error } else { p.accent })
                .bg(p.panel),
        ),
        rows[1],
    );
    let width = if s.settings.text_width == 0 {
        rows[2].width
    } else {
        s.settings.text_width as u16
    };
    let viewport = center(rows[2], width).inner(Margin::new(0, 1));
    a.reflow.update(&s.target, viewport.width as usize);
    let cursor = a
        .reflow
        .rows
        .iter()
        .position(|r| r.contains(&s.typed.len()))
        .unwrap_or(a.reflow.rows.len().saturating_sub(1));
    let step = (s.settings.line_gap + 1) as u16;
    let visible = (viewport.height / step).clamp(1, s.settings.visible_lines as u16) as usize;
    let start = cursor.saturating_sub(visible / 2);
    let count = visible.min(a.reflow.rows.len().saturating_sub(start));
    let content_height = (count as u16 * step).saturating_sub(s.settings.line_gap as u16);
    let top = viewport.y + viewport.height.saturating_sub(content_height) / 2;
    let pacer = (s.elapsed * s.settings.pace as f64 / 12.) as usize;
    for (row, range) in a.reflow.rows.iter().skip(start).take(visible).enumerate() {
        let mut spans = Vec::with_capacity(range.len());
        for i in range.clone() {
            let c = s.target[i];
            let typed = s.typed.get(i);
            let mut style = Style::default().fg(p.muted);
            if let Some(t) = typed {
                style = if *t == c || s.settings.blind {
                    Style::default().fg(p.fg)
                } else {
                    Style::default().fg(p.error).underlined()
                };
            }
            if s.settings.pace > 0 && i == pacer {
                style = style.fg(p.secondary).underlined();
            }
            if i == s.typed.len() {
                style = match s.settings.caret {
                    1 => Style::default().fg(p.accent).underlined().bold(),
                    2 => Style::default().fg(p.accent).bg(p.panel).bold(),
                    _ => Style::default().fg(p.on_accent()).bg(p.accent).bold(),
                };
            }
            if s.settings.bold_text {
                style = style.bold();
            }
            let display = if c == '\n' {
                "↵".into()
            } else if c == ' '
                && (i == s.typed.len() || (!s.settings.blind && typed.is_some_and(|t| *t != c)))
            {
                "·".into()
            } else {
                c.to_string()
            };
            spans.push(Span::styled(display, style));
        }
        f.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect::new(viewport.x, top + row as u16 * step, viewport.width, 1),
        );
    }
    if keyboard_height > 0 {
        keyboard(f, rows[3], a, p, false, false);
    }
}
fn chart(
    f: &mut Frame,
    r: Rect,
    net: &[(f64, f64)],
    raw: &[(f64, f64)],
    p: Theme,
    title: &str,
    x_title: &str,
) {
    if r.height < 5 {
        return;
    }
    if net.is_empty() {
        f.render_widget(
            Paragraph::new("\n  Complete a longer session to see the pace curve.")
                .fg(p.muted)
                .block(panel(title, p)),
            r,
        );
        return;
    }
    let ymax = net
        .iter()
        .chain(raw)
        .map(|p| p.1)
        .fold(0., f64::max)
        .max(20.);
    let ymax = (ymax / 20.).ceil() * 20.;
    let xmax = net
        .iter()
        .chain(raw)
        .map(|p| p.0)
        .fold(0., f64::max)
        .max(1.);
    let mut data = vec![
        Dataset::default()
            .name("wpm")
            .marker(match p.graph_style {
                1 => Marker::Dot,
                2 => Marker::Block,
                _ => Marker::Braille,
            })
            .graph_type(GraphType::Line)
            .style(Style::default().fg(p.accent))
            .data(net),
    ];
    if !raw.is_empty() {
        data.push(
            Dataset::default()
                .name("raw")
                .marker(match p.graph_style {
                    1 => Marker::Dot,
                    2 => Marker::Block,
                    _ => Marker::Braille,
                })
                .graph_type(GraphType::Line)
                .style(Style::default().fg(p.secondary))
                .data(raw),
        );
    }
    f.render_widget(
        Chart::new(data)
            .block(panel(title, p))
            .style(Style::default().bg(p.bg))
            .x_axis(
                Axis::default()
                    .title(x_title.to_owned())
                    .style(Style::default().fg(p.muted))
                    .bounds([0., xmax])
                    .labels([
                        "0".to_string(),
                        format!("{:.0}", xmax / 2.),
                        format!("{xmax:.0}"),
                    ]),
            )
            .y_axis(
                Axis::default()
                    .title("wpm")
                    .style(Style::default().fg(p.muted))
                    .bounds([0., ymax])
                    .labels([
                        "0".to_string(),
                        format!("{:.0}", ymax / 2.),
                        format!("{ymax:.0}"),
                    ]),
            ),
        r,
    );
}
fn metrics(f: &mut Frame, r: Rect, items: &[(&str, String)], p: Theme) {
    let cols =
        Layout::horizontal(vec![Constraint::Ratio(1, items.len() as u32); items.len()]).split(r);
    for (i, (name, value)) in items.iter().enumerate() {
        text(
            f,
            cols[i],
            vec![
                Line::from(*name).fg(p.muted),
                Line::from(value.clone()).fg(p.accent).bold(),
            ],
            p,
        );
    }
}
fn results(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    let s = &a.session;
    let roomy = r.height >= 22 && s.settings.large_results;
    let rows = Layout::vertical([
        Constraint::Length(if roomy { 8 } else { 5 }),
        Constraint::Length(if roomy { 3 } else { 2 }),
        Constraint::Min(5),
        Constraint::Length(if roomy { 3 } else { 1 }),
    ])
    .spacing(u16::from(roomy))
    .split(r);
    let title = if s.failed {
        "RUN ENDED"
    } else if a.pb && a.previous_best == 0. {
        "FIRST PR"
    } else if a.pb {
        "NEW PERSONAL RECORD"
    } else {
        "SESSION COMPLETE"
    };
    let banner = if a.pb && s.settings.celebrations {
        format!("◆  {title}  ◆")
    } else {
        title.into()
    };
    f.render_widget(
        panel(&banner, p).border_style(Style::default().fg(if s.failed {
            p.error
        } else {
            p.accent
        })),
        rows[0],
    );
    let body = inner(rows[0]);
    let cols = Layout::horizontal([
        Constraint::Length(if roomy { 23 } else { 12 }),
        Constraint::Min(10),
    ])
    .spacing(2)
    .split(body);
    if roomy {
        big_number(f, cols[0], &format!("{:.0}", s.wpm()), p.accent);
        label(
            f,
            Rect::new(cols[0].x, cols[0].y + 5, cols[0].width, 1),
            format!("{:.1} WPM", s.wpm()),
            p,
        );
    } else {
        text(
            f,
            cols[0],
            vec![
                Line::from(format!("{:.1} WPM", s.wpm()))
                    .fg(p.accent)
                    .bold(),
                Line::from(format!("+{} XP", a.reward)).fg(p.secondary),
            ],
            p,
        );
    }
    let pr = if s.failed {
        "Practice again to set a record".into()
    } else if a.previous_best > 0. {
        format!(
            "{:+.1} vs previous PR ({:.1})",
            s.wpm() - a.previous_best,
            a.previous_best
        )
    } else {
        "first completed benchmark".into()
    };
    let mut lines = vec![
        Line::from(format!(
            "{:.1}% accuracy   ·   {:.1}s",
            s.accuracy(),
            s.elapsed
        )),
        Line::from(pr).fg(p.secondary),
    ];
    if roomy {
        lines.push(
            Line::from(format!(
                "+{} XP  ·  {} quests  ·  {} milestones",
                a.reward, a.quest_bonus, a.milestone_bonus
            ))
            .fg(p.accent),
        );
        lines.push(
            Line::from(format!(
                "{} mistakes   ·   {} best combo",
                s.attempts - s.correct_attempts,
                s.best_combo
            ))
            .fg(p.muted),
        );
    }
    text(f, cols[1], lines, p);
    let progress = if a.profile.level() > a.previous_level {
        format!(
            "{}LEVEL UP  {:02} → {:02}   {}",
            if s.settings.celebrations { "◆ " } else { "" },
            a.previous_level,
            a.profile.level(),
            a.profile.rank()
        )
    } else {
        format!(
            "LV {:02}   {}   ·   {} / 500 XP",
            a.profile.level(),
            a.profile.rank(),
            a.profile.xp % 500
        )
    };
    text(
        f,
        rows[1],
        vec![
            Line::from(progress).fg(p.accent),
            Line::from(if a.unlocks.is_empty() {
                format!(
                    "raw {:.1} wpm   ·   consistency {:.0}%",
                    s.raw(),
                    s.consistency()
                )
            } else {
                format!("UNLOCKED  {}", a.unlocks.join(" · "))
            })
            .fg(p.secondary),
        ],
        p,
    );
    chart(
        f,
        rows[2],
        &s.pace_points,
        &s.raw_points,
        p,
        "PACE / WPM + RAW",
        "seconds",
    );
    let missed = s.missed_words();
    label(
        f,
        rows[3],
        format!(
            "missed: {}",
            if missed.is_empty() {
                "none".into()
            } else {
                missed.join(" · ")
            }
        ),
        p,
    );
}

fn tabs(f: &mut Frame, r: Rect, names: &[&str], selected: usize, p: Theme) {
    f.render_widget(
        Paragraph::new(Line::from(
            names
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    Span::styled(
                        format!(" {name}  "),
                        if i == selected {
                            Style::default().fg(p.accent).bg(p.panel).bold()
                        } else {
                            Style::default().fg(p.muted)
                        },
                    )
                })
                .collect::<Vec<_>>(),
        )),
        r,
    );
}
fn profile(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    let rows = Layout::vertical([Constraint::Length(2), Constraint::Min(10)]).split(r);
    tabs(
        f,
        rows[0],
        if r.width >= 80 {
            &[
                "overview",
                "records",
                "history",
                "key heatmap",
                "badges",
                "journey",
            ]
        } else {
            &["stats", "PRs", "history", "keys", "badges", "journey"]
        },
        a.profile_tab,
        p,
    );
    match a.profile_tab {
        1 => records(f, rows[1], a, p),
        2 => history(f, rows[1], a, p),
        4 => badges(f, rows[1], a, p),
        5 => journey(f, rows[1], a, p),
        3 => {
            let parts = Layout::vertical([
                Constraint::Length(if rows[1].height >= 23 { 13 } else { 7 }),
                Constraint::Min(3),
            ])
            .split(rows[1]);
            keyboard(f, parts[0], a, p, true, false);
            let mut lines = vec![
                Line::from("muted: untested   green: <5% errors   amber: <15%   red: ≥15%")
                    .fg(p.muted),
                Line::from(""),
                Line::from("weak keys     accuracy    attempts").fg(p.muted),
            ];
            for c in a.profile.weak() {
                let k = &a.profile.keys[&c];
                lines.push(Line::from(format!(
                    "    {c}           {:>5.1}%       {}",
                    100. * (1. - k.errors as f64 / k.hits.max(1) as f64),
                    k.hits
                )));
            }
            text(f, parts[1], lines, p);
        }
        _ => overview(f, rows[1], a, p),
    }
}
fn records(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    let entries = a.profile.records();
    if entries.is_empty() {
        f.render_widget(
            Paragraph::new(
                "
  Your first completed test sets your first PR.

  Records are kept separately for each test configuration.",
            )
            .fg(p.muted)
            .block(panel("PERSONAL RECORDS", p)),
            r,
        );
        return;
    }
    let cols = (r.width as usize / 48).clamp(1, 4);
    let height = 5u16;
    let visible = (r.height / height).max(1) as usize * cols;
    for (i, record) in entries
        .iter()
        .skip(a.history_offset)
        .take(visible)
        .enumerate()
    {
        let width = r.width / cols as u16;
        let area = Rect::new(
            r.x + i as u16 % cols as u16 * width,
            r.y + i as u16 / cols as u16 * height,
            width.saturating_sub(1),
            height.saturating_sub(1),
        );
        let pieces: Vec<_> = record.category.split(" / ").collect();
        let mut flags = vec![];
        if record.category.contains("p1") {
            flags.push("punctuation");
        }
        if record.category.contains("n1") {
            flags.push("numbers");
        }
        if record.category.contains("b1") {
            flags.push("blind");
        }
        if record.category.contains("c1") {
            flags.push("no-backspace");
        }
        if record.category.contains("s1") {
            flags.push("stop-on-error");
        }
        let title = format!(
            "PR · {} / {} / {}",
            pieces[0],
            pieces.get(1).unwrap_or(&""),
            pieces.get(2).unwrap_or(&"")
        );
        f.render_widget(
            panel(&title, p).border_style(Style::default().fg(p.accent)),
            area,
        );
        text(
            f,
            inner(area),
            vec![
                Line::from(format!(
                    "{:.1} WPM   {:.1}%   {}",
                    record.wpm, record.accuracy, record.date
                ))
                .fg(p.accent)
                .bold(),
                Line::from(if flags.is_empty() {
                    "plain text".into()
                } else {
                    flags.join(" · ")
                })
                .fg(p.muted),
            ],
            p,
        );
    }
}
fn badges(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    let rows = Layout::vertical([Constraint::Length(2), Constraint::Min(5)]).split(r);
    text(
        f,
        rows[0],
        vec![
            Line::from(format!(
                "{} / {} achievements   ·   ↑↓ explore",
                a.profile.achievements.len(),
                crate::progression::BADGES.len()
            ))
            .fg(p.accent),
            Line::from("g journey: endless badge tiers + XP rewards").fg(p.muted),
        ],
        p,
    );
    let cols = (r.width as usize / 42).clamp(1, 4);
    let count = rows[1].height as usize / 4 * cols;
    for (i, (name, requirement)) in crate::progression::BADGES
        .iter()
        .skip(a.history_offset)
        .take(count.max(1))
        .enumerate()
    {
        let unlocked = a.profile.achievements.iter().any(|v| v == name);
        let width = r.width / cols as u16;
        let area = Rect::new(
            r.x + i as u16 % cols as u16 * width,
            rows[1].y + i as u16 / cols as u16 * 4,
            width.saturating_sub(1),
            3,
        );
        let block = panel(
            &format!(
                "{} {}",
                if unlocked { "◆" } else { "◇" },
                name.to_uppercase()
            ),
            p,
        )
        .border_style(Style::default().fg(if unlocked { p.accent } else { p.muted }));
        f.render_widget(
            Paragraph::new(*requirement)
                .fg(if unlocked { p.fg } else { p.muted })
                .block(block),
            area,
        );
    }
}
fn journey_panel(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    f.render_widget(panel("∞ BADGE TRACKS · g journey", p), r);
    let body = inner(r);
    label(
        f,
        Rect::new(body.x, body.y, body.width, 1),
        format!(
            "{} tiers earned · no final tier",
            a.profile.milestone_tiers()
        ),
        p,
    );
    for (i, track) in a
        .profile
        .milestones()
        .iter()
        .take(body.height.saturating_sub(2) as usize / 3)
        .enumerate()
    {
        let y = body.y + 2 + i as u16 * 3;
        text(
            f,
            Rect::new(body.x, y, body.width, 2),
            vec![
                Line::from(format!(
                    "{} · next tier {}",
                    track.name.to_uppercase(),
                    track.tier() + 1
                ))
                .fg(p.accent),
                Line::from(format!(
                    "{}/{} {} · +{} XP",
                    track.progress(),
                    track.step,
                    track.unit,
                    track.reward
                ))
                .fg(p.muted),
            ],
            p,
        );
        segmented(
            f,
            Rect::new(body.x, y + 2, body.width, 1),
            track.progress() as f64 / track.step as f64,
            p,
        );
    }
}
fn journey(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    let rows = Layout::vertical([Constraint::Length(2), Constraint::Min(5)]).split(r);
    text(
        f,
        rows[0],
        vec![
            Line::from(format!(
                "∞ JOURNEY   {} tiers earned · no final tier",
                a.profile.milestone_tiers()
            ))
            .fg(p.accent),
            Line::from("Qualifying runs: ≥10s + ≥25 correct keys. ↑↓ scroll.").fg(p.muted),
        ],
        p,
    );
    let cols = (r.width as usize / 48).clamp(1, 3);
    let count = (rows[1].height as usize / 7).max(1) * cols;
    for (i, track) in a
        .profile
        .milestones()
        .iter()
        .skip(a.history_offset)
        .take(count)
        .enumerate()
    {
        let width = r.width / cols as u16;
        let area = Rect::new(
            r.x + i as u16 % cols as u16 * width,
            rows[1].y + i as u16 / cols as u16 * 7,
            width.saturating_sub(1),
            6.min(rows[1].height),
        );
        f.render_widget(
            panel(
                &format!(
                    "∞ {} · NEXT TIER {:02}",
                    track.name.to_uppercase(),
                    track.tier() + 1
                ),
                p,
            )
            .border_style(Style::default().fg(p.accent)),
            area,
        );
        let body = inner(area);
        text(
            f,
            body,
            vec![
                Line::from(format!(
                    "{} tiers earned  ·  +{} XP next",
                    track.tier(),
                    track.reward
                ))
                .fg(p.secondary),
                Line::from(format!(
                    "{} / {} {}",
                    track.progress(),
                    track.step,
                    track.unit
                )),
            ],
            p,
        );
        if body.height >= 4 {
            segmented(
                f,
                Rect::new(body.x, body.y + 3, body.width, 1),
                track.progress() as f64 / track.step as f64,
                p,
            );
        }
    }
}
fn overview(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    let rows = Layout::vertical([
        Constraint::Length(if r.height < 15 { 2 } else { 3 }),
        Constraint::Length(10),
        Constraint::Min(0),
    ])
    .split(r);
    metrics(
        f,
        rows[0],
        &[
            ("streak", format!("{} days", a.profile.streak())),
            ("longest", format!("{} days", a.profile.longest_streak())),
            ("sessions", a.profile.history.len().to_string()),
            (
                "level",
                format!("{} · {} XP", a.profile.level(), a.profile.xp),
            ),
        ],
        p,
    );
    calendar(f, rows[1], a, p);
    let category = crate::store::Profile::category(&a.profile.settings);
    let records: Vec<_> = a
        .profile
        .history
        .iter()
        .filter(|r| r.category == category && !r.failed)
        .rev()
        .take(30)
        .collect();
    let points: Vec<_> = records
        .iter()
        .rev()
        .enumerate()
        .map(|(i, r)| ((i + 1) as f64, r.wpm))
        .collect();
    if rows[2].height < 6 {
        if let Some(record) = records.first() {
            label(
                f,
                rows[2],
                format!(
                    "latest matching test: {:.1} WPM · {:.1}% accuracy",
                    record.wpm, record.accuracy
                ),
                p,
            );
        }
        return;
    }
    chart(
        f,
        rows[2],
        &points,
        &[],
        p,
        &format!("{} / matching settings", a.profile.settings.mode.label()),
        "sessions",
    );
}
fn calendar(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    let today = Local::now().date_naive();
    let days = a.profile.activity();
    let weeks = ((r.width.saturating_sub(12)) / 2).clamp(4, 40) as usize;
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let first = monday - Duration::weeks(weeks as i64 - 1);
    let mut lines = vec![
        Line::from(format!(
            "activity   {} – {}",
            first.format("%d %b"),
            today.format("%d %b %Y")
        ))
        .fg(p.muted),
    ];
    for (day, label) in ["M", "T", "W", "T", "F", "S", "S"].iter().enumerate() {
        let mut row = vec![Span::styled(
            format!(" {label}  "),
            Style::default().fg(p.muted),
        )];
        for w in 0..weeks {
            let date = first + Duration::days(w as i64 * 7 + day as i64);
            let n = days.get(&date).copied().unwrap_or(0);
            let glyph = if date > today {
                "  "
            } else {
                match n {
                    0 => "░ ",
                    1 => "▒ ",
                    2..=3 => "▓ ",
                    _ => "█ ",
                }
            };
            row.push(Span::styled(
                glyph,
                Style::default().fg(if n == 0 { p.muted } else { p.accent }),
            ));
        }
        lines.push(Line::from(row));
    }
    lines.push(
        Line::from("    less  ░ ▒ ▓ █  more   ·   successful sessions / local day").fg(p.muted),
    );
    text(f, r, lines, p);
}
fn history(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    let mut lines = vec![
        Line::from("date         mode         wpm     accuracy     duration    XP").fg(p.muted),
        Line::from(""),
    ];
    for rec in a
        .profile
        .history
        .iter()
        .rev()
        .skip(a.history_offset)
        .take(r.height.saturating_sub(2) as usize)
    {
        let mode = rec.category.split(" / ").next().unwrap_or("test");
        lines.push(Line::from(format!(
            "{}   {:<10} {:>6.1}    {:>5.1}%     {:>5.1}s    {}{}",
            rec.date,
            mode,
            rec.wpm,
            rec.accuracy,
            rec.seconds,
            rec.xp,
            if rec.failed { " ×" } else { "" }
        )));
    }
    if a.profile.history.is_empty() {
        lines.push(Line::from(
            "No sessions yet. Press Esc, then Enter to start.",
        ));
    }
    text(f, r, lines, p);
}
fn settings(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    let parts = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(5),
        Constraint::Length(4),
    ])
    .split(r);
    tabs(f, parts[0], &crate::settings::GROUPS, a.settings_group, p);
    let wide = r.width >= 95;
    let columns = Layout::horizontal([
        Constraint::Percentage(if wide { 53 } else { 100 }),
        Constraint::Min(0),
    ])
    .spacing(u16::from(wide) * 2)
    .split(parts[1]);
    f.render_widget(panel("CUSTOMIZE", p), columns[0]);
    let list = inner(columns[0]);
    let options = crate::settings::rows(&a.profile.settings, a.settings_group);
    let step = if list.height >= options.len() as u16 * 2 {
        2
    } else {
        1
    };
    let visible = (list.height / step).max(1) as usize;
    let offset = a.setting.saturating_sub(visible - 1);
    let mut lines = vec![];
    for (i, (name, value, _)) in options.iter().enumerate().skip(offset).take(visible) {
        lines.push(
            Line::from(format!(
                " {}  {:<19} ‹ {} ›",
                if a.setting == i { "▶" } else { " " },
                name,
                value
            ))
            .style(if i == a.setting {
                Style::default().fg(p.on_accent()).bg(p.accent).bold()
            } else {
                Style::default().fg(p.fg)
            }),
        );
        if step == 2 {
            lines.push(Line::from(""));
        }
    }
    text(f, list, lines, p);
    if wide {
        appearance_preview(f, columns[1], a, p);
    }
    let desc = options[a.setting].2;
    text(
        f,
        Rect {
            y: parts[2].y + 1,
            height: parts[2].height.saturating_sub(1),
            ..parts[2]
        },
        vec![
            Line::from(desc).fg(p.fg),
            Line::from(format!(
                "saved automatically   ·   {:02} / {:02}",
                a.setting + 1,
                options.len()
            ))
            .fg(p.muted),
        ],
        p,
    );
}
fn appearance_preview(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    f.render_widget(panel("LIVE PREVIEW", p), r);
    let body = inner(r);
    if body.height < 5 {
        return;
    }
    let options = &a.profile.settings;
    let mut spans = vec![Span::styled("the quick ", Style::default().fg(p.fg))];
    let caret = match options.caret {
        1 => Style::default().fg(p.accent).underlined(),
        2 => Style::default().fg(p.accent).bg(p.panel).bold(),
        _ => Style::default().bg(p.accent).fg(p.on_accent()),
    };
    spans.push(Span::styled("b", caret));
    spans.push(Span::styled("rown fox", Style::default().fg(p.muted)));
    let mut sample = Paragraph::new(vec![
        Line::from(format!("{} · your palette", p.name.to_uppercase())).fg(p.secondary),
        Line::from(""),
        Line::from(spans),
    ]);
    if options.bold_text {
        sample = sample.bold();
    }
    f.render_widget(
        sample,
        Rect::new(body.x + 1, body.y, body.width.saturating_sub(2), 3),
    );
    if body.height >= 11 {
        let number = Rect::new(body.x + 1, body.y + 4, body.width.saturating_sub(2), 5);
        if options.large_results {
            big_number(f, number, "82", p.accent);
        } else {
            text(
                f,
                number,
                vec![
                    Line::from("82.0 WPM").fg(p.accent).bold(),
                    Line::from("98.5% accuracy").fg(p.muted),
                ],
                p,
            );
        }
        label(
            f,
            Rect::new(body.x + 1, body.y + 9, body.width.saturating_sub(2), 1),
            "sample result · 82 WPM".into(),
            p,
        );
    }
    if body.height >= 15 {
        segmented(
            f,
            Rect::new(body.x + 1, body.y + 11, body.width.saturating_sub(2), 1),
            0.65,
            p,
        );
        label(
            f,
            Rect::new(body.x + 1, body.y + 13, body.width.saturating_sub(2), 1),
            format!(
                "{}  ·  {}  ·  {}",
                ["rounded", "sharp", "double", "borderless"][options.panel_style],
                ["keycaps", "tiles", "outline"][options.key_style],
                ["braille", "dots", "blocks"][options.graph_style]
            ),
            p,
        );
    }
    if body.height >= 24 {
        keyboard(
            f,
            Rect::new(body.x, body.y + 16, body.width, body.height - 16),
            a,
            p,
            false,
            false,
        );
    }
}
fn themes(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    let rows = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(2),
    ])
    .split(r);
    let preview = theme::get(a.theme_cursor);
    let applied = theme::get(a.profile.settings.theme);
    text(
        f,
        rows[0],
        vec![
            Line::from(format!(
                "PREVIEW  {:<10}    APPLIED  {}",
                preview.name.to_uppercase(),
                applied.name.to_uppercase()
            ))
            .fg(p.accent)
            .bold(),
            Line::from("Enter saves your choice. Esc keeps the applied theme.").fg(p.muted),
        ],
        p,
    );
    let cols = a.theme_columns;
    let row_height = if rows[1].height >= 16 { 5 } else { 4 };
    let visible = (rows[1].height / row_height).max(1) as usize;
    let selected_row = a.theme_cursor / cols;
    let first_row = selected_row.saturating_sub(visible - 1);
    for row in 0..visible {
        for col in 0..cols {
            let index = (first_row + row) * cols + col;
            let Some(t) = theme::THEMES.get(index) else {
                continue;
            };
            let width = rows[1].width / cols as u16;
            let area = Rect::new(
                rows[1].x + col as u16 * width,
                rows[1].y + row as u16 * row_height,
                width.saturating_sub(1),
                row_height.saturating_sub(1),
            );
            let focused = index == a.theme_cursor;
            let saved = index == a.profile.settings.theme;
            let title = format!(
                "{} {:02} {} {}",
                if focused { "▶" } else { " " },
                index + 1,
                t.name,
                if saved { "[APPLIED]" } else { "" }
            );
            let block = Block::bordered()
                .border_type(if focused {
                    BorderType::Double
                } else {
                    BorderType::Rounded
                })
                .border_style(Style::default().fg(if focused { p.accent } else { t.muted }))
                .title(Span::styled(
                    title,
                    Style::default()
                        .fg(if focused { p.accent } else { t.fg })
                        .bold(),
                ))
                .bg(t.bg);
            f.render_widget(block, area);
            let mut sample = vec![
                Span::styled("██ ", Style::default().fg(t.accent)),
                Span::styled("██ ", Style::default().fg(t.secondary)),
                Span::styled("██ ", Style::default().fg(t.error)),
            ];
            sample.push(Span::styled(
                if focused { "PREVIEW" } else { "Aa" },
                Style::default().fg(t.fg),
            ));
            f.render_widget(Paragraph::new(Line::from(sample)), inner(area));
        }
    }
    label(
        f,
        rows[2],
        format!(
            "{:02} / {}    {}",
            a.theme_cursor + 1,
            theme::THEMES.len(),
            if a.profile.settings.accent > 0 {
                "custom accent enabled · change in Appearance"
            } else {
                "original palette accents"
            }
        ),
        p,
    );
}

fn key_rows(layout: usize) -> [&'static str; 4] {
    match layout % 3 {
        1 => ["1234567890[]", "',.pyfgcrl/=", "aoeuidhtns-", ";qjkxbmwvz"],
        2 => ["1234567890-=", "qwfpgjluy;[]", "arstdhneio'", "zxcvbkm,./"],
        _ => ["1234567890-=", "qwertyuiop[]", "asdfghjkl;'", "zxcvbnm,./"],
    }
}
fn base_key(c: char) -> char {
    let shifted = "!@#$%^&*()_+{}:\"<>?";
    let plain = "1234567890-=[];',./";
    shifted
        .chars()
        .position(|v| v == c)
        .and_then(|i| plain.chars().nth(i))
        .unwrap_or(c.to_ascii_lowercase())
}
fn keyboard(f: &mut Frame, r: Rect, a: &App, p: Theme, heat: bool, lab: bool) {
    let layout = a.profile.settings.keyboard_layout;
    let rows = key_rows(layout);
    let tall = r.height >= 13 && a.profile.settings.key_style != 1;
    let h = if tall { 3 } else { 1 };
    let gap = if tall { 0 } else { 1 };
    let width = if r.width >= 76 { 6 } else { 4 };
    let area = center(r, rows[0].len() as u16 * width + 4);
    let expected = if a.screen == Screen::Typing {
        a.session
            .target
            .get(a.session.typed.len())
            .copied()
            .map(base_key)
    } else {
        None
    };
    for (row, keys) in rows.iter().enumerate() {
        let start = area.x + row as u16;
        for (col, c) in keys.chars().enumerate() {
            let key = Rect::new(
                start + col as u16 * width,
                area.y + row as u16 * (h + gap),
                width - 1,
                h,
            );
            if key.right() > r.right() || key.bottom() > r.bottom() {
                continue;
            }
            let mut fg = p.muted;
            let mut bg = if a.profile.settings.key_style == 2 {
                p.bg
            } else {
                p.panel
            };
            if heat {
                let (hits, errors) = a
                    .profile
                    .keys
                    .iter()
                    .filter(|(key, _)| base_key(**key) == c)
                    .fold((0u64, 0u64), |(h, e), (_, k)| (h + k.hits, e + k.errors));
                if hits > 0 {
                    let error = errors as f64 / hits as f64;
                    fg = if error < 0.05 {
                        Color::Rgb(125, 194, 135)
                    } else if error < 0.15 {
                        Color::Rgb(224, 177, 82)
                    } else {
                        Color::Rgb(231, 110, 124)
                    };
                }
            } else if lab && a.pressed.iter().any(|x| base_key(*x) == c) {
                fg = p.accent;
            }
            if expected == Some(c) {
                fg = p.on_accent();
                bg = p.accent;
            }
            if !heat
                && a.last_input.is_some_and(|(last, _, t)| {
                    base_key(last) == c && t.elapsed().as_millis() < 130
                })
            {
                fg = p.on_accent();
                bg = if a.session.settings.blind || a.last_input.is_some_and(|(_, ok, _)| ok) {
                    p.secondary
                } else {
                    p.error
                };
            }
            let block = if tall {
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(fg))
                    .bg(bg)
            } else {
                Block::default().bg(bg)
            };
            f.render_widget(
                Paragraph::new(c.to_string())
                    .fg(fg)
                    .alignment(Alignment::Center)
                    .block(block),
                key,
            );
        }
    }
    if tall {
        let bar = Rect::new(area.x + 10, area.y + 12, area.width.saturating_sub(20), 1);
        f.render_widget(
            Paragraph::new(if expected == Some(' ') {
                "space  ·  thumb"
            } else {
                "space"
            })
            .centered()
            .fg(if expected == Some(' ') {
                p.accent
            } else {
                p.muted
            })
            .bg(p.panel),
            bar,
        );
    }
}
fn keyboard_lab(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(if r.height >= 20 { 13 } else { 7 }),
        Constraint::Min(2),
    ])
    .split(r);
    label(
        f,
        rows[0],
        "keyboard   ·   press any key to inspect its terminal event".into(),
        p,
    );
    keyboard(f, rows[1], a, p, false, true);
    text(
        f,
        rows[2],
        vec![
            Line::from(format!("\n{}", a.last_key)).fg(p.accent),
            Line::from("Visited keys stay highlighted. Layout is set in Appearance.").fg(p.muted),
            Line::from("Terminal events cannot identify every physical key or release.")
                .fg(p.muted),
        ],
        p,
    );
}
fn mode_info(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    let mode = Mode::ALL[a.selected];
    let area = center(r, 86);
    f.render_widget(panel("◇ MODE GUIDE", p), area);
    let body = inner(area);
    text(
        f,
        body,
        vec![
            Line::from(format!(
                "{}  {:02} / {}   {}",
                mode.icon(),
                a.selected + 1,
                Mode::ALL.len(),
                mode.label().to_uppercase()
            ))
            .fg(p.accent)
            .bold(),
            Line::from(""),
            Line::from(mode.description()),
            Line::from(""),
            Line::from("Use ↑↓ to explore every mode. Enter starts this mode.").fg(p.secondary),
            Line::from("O opens local files. S changes duration, word count and difficulty.")
                .fg(p.muted),
        ],
        p,
    );
}
fn import_picker(f: &mut Frame, r: Rect, a: &App, p: Theme) {
    let clean = |s: String| s.chars().filter(|c| !c.is_control()).collect::<String>();
    let rows = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(4),
        Constraint::Length(3),
    ])
    .spacing(1)
    .split(r);
    let input = if a.picker.input.is_empty() {
        "Type or paste a path, or select a file below".into()
    } else {
        format!("{}▏", clean(a.picker.input.clone()))
    };
    let scroll = if a.picker.input.is_empty() {
        0
    } else {
        unicode_width::UnicodeWidthStr::width(input.as_str())
            .saturating_sub(rows[0].width.saturating_sub(2) as usize) as u16
    };
    f.render_widget(
        Paragraph::new(input)
            .scroll((0, scroll))
            .fg(p.accent)
            .block(panel("↥ LOAD LOCAL TEXT · UTF-8 · max 1 MiB", p)),
        rows[0],
    );
    let block = panel(&clean(a.picker.dir.display().to_string()), p);
    let list = inner(rows[1]);
    f.render_widget(block, rows[1]);
    let visible = list.height.max(1) as usize;
    let offset = a.picker.selected.saturating_sub(visible - 1);
    let lines: Vec<_> = a
        .picker
        .entries
        .iter()
        .enumerate()
        .skip(offset)
        .take(visible)
        .map(|(i, path)| {
            let name = if path == a.picker.dir.parent().unwrap_or(&a.picker.dir) {
                "..".into()
            } else {
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            };
            Line::from(format!(
                "{} {} {}",
                if i == a.picker.selected { "›" } else { " " },
                if path.is_dir() { "▸" } else { "·" },
                clean(name)
            ))
            .style(if i == a.picker.selected {
                Style::default().bg(p.accent).fg(p.on_accent())
            } else {
                Style::default().fg(p.fg)
            })
        })
        .collect();
    f.render_widget(Paragraph::new(lines).fg(p.fg), list);
    text(
        f,
        rows[2],
        vec![
            Line::from(if a.picker.error.is_empty() {
                "Enter opens folders or loads a file and starts practice."
            } else {
                &a.picker.error
            })
            .fg(if a.picker.error.is_empty() {
                p.muted
            } else {
                p.error
            }),
            Line::from("Nothing is uploaded. Your last successful import is cached locally.")
                .fg(p.muted),
        ],
        p,
    );
}
fn help(f: &mut Frame, r: Rect, p: Theme) {
    if r.height < 24 {
        text(
            f,
            r,
            vec![
                Line::from("NAVIGATION · outside tests").fg(p.accent),
                "F2 home · F3 profile · F4 settings · F5 themes".into(),
                "F6 key lab · F1 help · click the header".into(),
                "p/s/t shortcuts · g journey · i mode guide".into(),
                "Letters stay literal in typing and the key lab.".into(),
                Line::from("PRACTICE").fg(p.accent),
                "↑↓ selects · Enter starts · 1–9 starts a mode".into(),
                "Backspace corrects · Ctrl+W erases a word".into(),
                "Ctrl+R restarts · Ctrl+D ends endless/zen".into(),
                "Tab indents code/custom · Enter types ↵".into(),
                "Esc abandons · Ctrl+C quits".into(),
                "Results: r retry · m missed words · w weak keys".into(),
                Line::from("FILES").fg(p.accent),
                "O loads a local file · paste a path there".into(),
                "typedef --export > progress.json".into(),
            ],
            p,
        );
        return;
    }

    text(
        f,
        r,
        vec![
            Line::from("controls").fg(p.accent),
            "".into(),
            "↑↓ / j,k choose · Enter starts · 1–9 jump to a mode".into(),
            "Backspace corrects · Ctrl+W erases a word · Ctrl+R restarts".into(),
            "Enter types ↵ · Tab indents code/custom · Esc abandons".into(),
            "Endless / Zen: Ctrl+D finishes and saves".into(),
            "Results: m drills missed words · w trains weak keys".into(),
            "".into(),
            Line::from("display & feedback").fg(p.accent),
            "p profile · s settings · t themes · g journey · r PRs".into(),
            "F2 home · F3 profile · F4 settings · F5 themes".into(),
            "F6 keys · F1 help · click the header to navigate".into(),
            "Navigation letters stay literal in typing and key lab.".into(),
            "Tab switches settings categories and profile views.".into(),
            "Font size: use terminal zoom, often Cmd+ or Ctrl+Shift+.".into(),
            "i explains every mode · r opens personal records.".into(),
            "".into(),
            Line::from("files & sharing").fg(p.accent),
            "o loads local files · browse or type/paste a path".into(),
            "tpdf --file notes.txt · tpdf --seed 42 --mode words".into(),
            "typedef --url https://… caches text for offline practice".into(),
            "typedef --export > progress.json".into(),
        ],
        p,
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};
    fn snapshot(name: &str, buffer: &ratatui::buffer::Buffer) {
        if std::env::var_os("TYPEDEF_SNAPSHOT").is_none() {
            return;
        }
        let w = buffer.area.width;
        let h = buffer.area.height;
        let text = buffer
            .content
            .chunks(w as usize)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        let cells: Vec<_> = buffer.content.iter().map(|c| serde_json::json!({"s":c.symbol(), "fg": format!("{:?}", c.fg), "bg":format!("{:?}", c.bg)})).collect();
        std::fs::write(
            format!("/tmp/typedef-{name}.json"),
            serde_json::to_vec(&serde_json::json!({"width":w,"height":h,"cells":cells})).unwrap(),
        )
        .unwrap();
        std::fs::write(format!("/tmp/typedef-{name}.txt"), text).unwrap();
    }
    #[test]
    fn all_screens_and_themes_render() {
        let dir = std::env::temp_dir().join(format!("typedef-ui-{}", std::process::id()));
        let store = crate::store::Store::open(Some(dir.clone())).unwrap();
        let mut a = App::new(Default::default(), store, String::new());
        a.fixed_seed = Some(42);
        a.launch(false);
        for i in 0..90 {
            let c = a.session.target[i];
            a.session.input(if i == 12 { 'x' } else { c });
            a.session.tick((i + 1) as f64 / 6.);
        }
        for days in 0..30 {
            a.profile.record(&a.session);
            a.profile.history.last_mut().unwrap().date =
                Local::now().date_naive() - Duration::days(days * 2);
        }
        a.previous_best = 68.;
        a.previous_level = a.profile.level();
        a.pb = true;
        a.reward = 17;
        for (w, h) in [
            (30, 10),
            (60, 20),
            (80, 24),
            (120, 40),
            (160, 40),
            (220, 50),
        ] {
            let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
            for theme in 0..theme::THEMES.len() {
                a.profile.settings.theme = theme;
                for screen in [
                    Screen::Home,
                    Screen::Typing,
                    Screen::Results,
                    Screen::Profile,
                    Screen::Settings,
                    Screen::Themes,
                    Screen::Help,
                    Screen::Keyboard,
                    Screen::Import,
                    Screen::ModeInfo,
                ] {
                    a.screen = screen;
                    terminal.draw(|f| draw(f, &mut a)).unwrap();
                    let buffer = terminal.backend().buffer();
                    let rendered: String = buffer.content.iter().map(|c| c.symbol()).collect();
                    assert!(!rendered.contains("OFFLINE FIRST"));
                    assert!(!rendered.contains("find your flow"));
                    if w >= 60 && screen == Screen::Home {
                        assert!(rendered.contains("custom"));
                        assert!(rendered.contains("endurance"));
                        assert!(rendered.contains("LV 02"));
                        assert_eq!(buffer[(2, 4)].symbol(), "╭");
                        assert_eq!(buffer[(w - 3, 4)].symbol(), "╮");
                    }
                    if w >= 60 && screen == Screen::Results {
                        assert!(rendered.contains("accuracy"));
                        assert!(rendered.contains("WPM"));
                        assert!(rendered.contains("XP"));
                    }
                    if w == 120 && theme == 1 && screen == Screen::Home {
                        snapshot("home-paper", buffer);
                    }
                    if w >= 60 && theme == 0 {
                        let name = match screen {
                            Screen::Home => "home",
                            Screen::Typing => "typing",
                            Screen::Results => "results",
                            Screen::Profile => "profile",
                            Screen::Settings => "settings",
                            Screen::Themes => "themes",
                            Screen::Keyboard => "keyboard",
                            Screen::Help => "help",
                            Screen::Import => "import",
                            Screen::ModeInfo => "modes",
                        };
                        snapshot(&format!("{name}-{w}"), buffer);
                    }
                }
            }
        }
        a.profile.settings.theme = 0;
        for (w, h) in [(60, 20), (80, 24), (120, 40), (160, 40), (220, 50)] {
            let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
            a.screen = Screen::Settings;
            for group in 0..4 {
                a.settings_group = group;
                for row in 0..crate::settings::count(group) {
                    a.setting = row;
                    terminal.draw(|f| draw(f, &mut a)).unwrap();
                    let output: String = terminal
                        .backend()
                        .buffer()
                        .content
                        .iter()
                        .map(|c| c.symbol())
                        .collect();
                    assert!(
                        output.contains(crate::settings::rows(&a.profile.settings, group)[row].0)
                    );
                    if group == 1 && row == 9 {
                        snapshot(&format!("appearance-{w}"), terminal.backend().buffer());
                    }
                }
            }
            a.screen = Screen::Profile;
            for tab in 0..6 {
                a.profile_tab = tab;
                terminal.draw(|f| draw(f, &mut a)).unwrap();
                snapshot(&format!("profile-{tab}-{w}"), terminal.backend().buffer());
            }
            a.screen = Screen::Themes;
            for selected in 0..theme::THEMES.len() {
                a.theme_cursor = selected;
                terminal.draw(|f| draw(f, &mut a)).unwrap();
                let output: String = terminal
                    .backend()
                    .buffer()
                    .content
                    .iter()
                    .map(|c| c.symbol())
                    .collect();
                assert!(output.contains(&format!(
                    "▶ {:02} {}",
                    selected + 1,
                    theme::get(selected).name
                )));
                assert!(output.contains("APPLIED"));
            }
            snapshot(&format!("themes-last-{w}"), terminal.backend().buffer());
            for style in 0..4 {
                a.profile.settings.panel_style = style;
                a.profile.settings.key_style = style % 3;
                a.profile.settings.graph_style = style % 3;
                a.profile.settings.accent = style + 1;
                for screen in [
                    Screen::Home,
                    Screen::Typing,
                    Screen::Results,
                    Screen::Settings,
                ] {
                    a.screen = screen;
                    terminal.draw(|f| draw(f, &mut a)).unwrap();
                }
            }
            a.profile.settings.panel_style = 0;
            a.profile.settings.accent = 0;
            a.profile.settings.key_style = 0;
            a.profile.settings.graph_style = 0;
        }
        a.profile = Default::default();
        a.selected = 0;
        a.screen = Screen::Home;
        for (w, h) in [(60, 20), (120, 40)] {
            let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
            terminal.draw(|f| draw(f, &mut a)).unwrap();
            let output: String = terminal
                .backend()
                .buffer()
                .content
                .iter()
                .map(|c| c.symbol())
                .collect();
            assert!(output.contains("No record yet"));
            assert!(output.contains("LV 01"));
            snapshot(&format!("home-empty-{w}"), terminal.backend().buffer());
        }
        drop(a);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    #[ignore = "Generates deterministic website media; run with TYPEDEF_SNAPSHOT=1"]
    fn capture_site_media() {
        let dir =
            std::env::temp_dir().join(format!("typedef-media-profile-{}", std::process::id()));
        let store = crate::store::Store::open(Some(dir.clone())).unwrap();
        let mut a = App::new(Default::default(), store, String::new());
        a.profile.settings.seconds = 15;
        a.fixed_seed = Some(42);
        a.launch(false);
        // Sample history from generated test sessions, never a user's personal profile.
        for day in (1..=12).rev() {
            let mut session = a.session.clone();
            for i in 0..(55 + day * 2) {
                session.input(session.target[i]);
            }
            session.tick(15.);
            a.profile.record_on(
                &session,
                Local::now().date_naive() - Duration::days(day as i64),
            );
        }
        let mut terminal = Terminal::new(TestBackend::new(120, 36)).unwrap();
        a.screen = Screen::Home;
        terminal.draw(|f| draw(f, &mut a)).unwrap();
        snapshot("site-home", terminal.backend().buffer());
        a.launch(false);
        for frame in 0..180 {
            if frame % 2 == 0 {
                let c = a.session.target[a.session.typed.len()];
                a.session.input(c);
            }
            a.session.tick((frame + 1) as f64 / 12.);
            terminal.draw(|f| draw(f, &mut a)).unwrap();
            snapshot(&format!("video-{frame:04}"), terminal.backend().buffer());
            if frame == 90 {
                snapshot("site-typing", terminal.backend().buffer());
            }
        }
        a.tick().unwrap();
        terminal.draw(|f| draw(f, &mut a)).unwrap();
        snapshot("site-results", terminal.backend().buffer());
        for frame in 180..216 {
            snapshot(&format!("video-{frame:04}"), terminal.backend().buffer());
        }
        a.screen = Screen::Profile;
        a.profile_tab = 5;
        terminal.draw(|f| draw(f, &mut a)).unwrap();
        snapshot("site-journey", terminal.backend().buffer());
        for (name, theme) in [
            ("fern", 0),
            ("cobalt", 13),
            ("rose", 14),
            ("paper", 1),
            ("ember", 5),
        ] {
            a.profile.settings.theme = theme;
            a.screen = Screen::Home;
            terminal.draw(|f| draw(f, &mut a)).unwrap();
            snapshot(&format!("site-{name}"), terminal.backend().buffer());
        }
        drop(a);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn shifted_key_mapping() {
        assert_eq!(base_key('!'), '1');
        assert_eq!(base_key('?'), '/');
        assert_eq!(base_key('A'), 'a');
    }
    #[test]
    fn header_clicks_route_every_screen_and_leave_typing_alone() {
        use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
        let dir = std::env::temp_dir().join(format!("typedef-nav-ui-{}", std::process::id()));
        let store = crate::store::Store::open(Some(dir.clone())).unwrap();
        let mut a = App::new(Default::default(), store, String::new());
        let click = |rect: Rect| MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: rect.x,
            row: rect.y,
            modifiers: KeyModifiers::NONE,
        };
        for (w, h) in [(60, 20), (80, 24), (160, 40)] {
            let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
            for source in [
                Screen::Home,
                Screen::Settings,
                Screen::Profile,
                Screen::Themes,
                Screen::Results,
                Screen::Help,
                Screen::Keyboard,
                Screen::Import,
                Screen::ModeInfo,
            ] {
                a.screen = source;
                terminal.draw(|f| draw(f, &mut a)).unwrap();
                let routes = a.nav_hits.clone();
                assert!(routes.len() >= 4);
                for (rect, destination) in routes {
                    a.screen = source;
                    terminal.draw(|f| draw(f, &mut a)).unwrap();
                    a.mouse(click(rect));
                    assert_eq!(a.screen, destination);
                    assert!(a.nav_hits.is_empty()); // old hit areas cannot be used before a redraw
                }
            }
            a.screen = Screen::Home;
            terminal.draw(|f| draw(f, &mut a)).unwrap();
            let target = a.nav_hits[2].0;
            a.screen = Screen::Typing;
            terminal.draw(|f| draw(f, &mut a)).unwrap();
            assert!(a.nav_hits.is_empty());
            a.mouse(click(target));
            assert_eq!(a.screen, Screen::Typing);
        }
        drop(a);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
