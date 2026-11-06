mod app;
mod content;
mod engine;
mod import;
mod progression;
mod settings;
mod store;
mod text_layout;
mod theme;
mod ui;
use clap::Parser;
use crossterm::{
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event, KeyEventKind,
    },
    execute,
};
use engine::Mode;
use std::{
    io::{self, IsTerminal},
    path::PathBuf,
    time::Duration,
};
#[derive(Parser)]
#[command(version, about = "A terminal typing playground.")]
struct Cli {
    #[arg(long, value_enum)]
    mode: Option<Mode>,
    /// Reproducible content seed (share with the same version and settings)
    #[arg(long)]
    seed: Option<u64>,
    #[arg(long,value_parser=clap::value_parser!(u64).range(1..=3600))]
    seconds: Option<u64>,
    #[arg(long,value_parser=clap::value_parser!(u32).range(1..=10000))]
    words: Option<u32>,
    /// UTF-8 text or source code to practice (at most 1 MiB)
    #[arg(long, conflicts_with = "url")]
    file: Option<PathBuf>,
    /// Explicitly download an HTTPS text file and cache it for offline practice (requires curl)
    #[arg(long)]
    url: Option<String>,
    /// Print your local profile as JSON, without starting the TUI
    #[arg(long)]
    export: bool,
    /// Override the platform-native profile directory
    #[arg(long)]
    data_dir: Option<PathBuf>,
}
// Drop also runs during unwinding; terminal capture modes must not leak after a panic.
struct InputModes;
impl Drop for InputModes {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), DisableMouseCapture, DisableBracketedPaste);
    }
}
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let store = store::Store::open(cli.data_dir)?;
    let mut profile = store.load()?;
    if cli.export {
        println!("{}", serde_json::to_string_pretty(&profile)?);
        return Ok(());
    }
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err("typedef needs an interactive terminal. Use --help or --export for non-interactive commands.".into());
    }
    let mut custom = String::new();
    if let Some(url) = cli.url {
        if !url.starts_with("https://") {
            return Err("Online imports require an https:// URL".into());
        }
        let temp = store.dir.join("download.tmp");
        let status = std::process::Command::new("curl")
            .args([
                "--fail",
                "--silent",
                "--show-error",
                "--location",
                "--proto",
                "=https",
                "--proto-redir",
                "=https",
                "--max-time",
                "20",
                "--max-filesize",
                "1048576",
                "--output",
            ])
            .arg(&temp)
            .arg("--")
            .arg(url)
            .status()?;
        if !status.success() {
            let _ = std::fs::remove_file(&temp);
            return Err("Download failed. Your previous cached text is unchanged.".into());
        }
        let result = content::read_text(&temp);
        let _ = std::fs::remove_file(&temp);
        custom = result?;
        std::fs::write(store.dir.join("import.txt"), &custom)?;
        profile.settings.mode = Mode::Custom;
    } else if let Some(file) = cli.file {
        custom = content::read_text(&file)?;
        profile.settings.mode = Mode::Custom;
    } else if store.dir.join("import.txt").exists() {
        custom = content::read_text(&store.dir.join("import.txt"))?;
    }
    if let Some(mode) = cli.mode {
        profile.settings.mode = mode;
    }
    if let Some(seconds) = cli.seconds {
        profile.settings.seconds = seconds;
    }
    if let Some(words) = cli.words {
        profile.settings.words = words as usize;
    }
    // Treat manually edited profile settings defensively.
    profile.settings.seconds = profile.settings.seconds.clamp(1, 3600);
    profile.settings.words = profile.settings.words.clamp(1, 10000);
    profile.settings.sanitize();
    let mut a = app::App::new(profile, store, custom);
    a.fixed_seed = cli.seed;

    let mut terminal = ratatui::init();
    // Ratatui installs a panic hook that restores raw mode and the alternate screen.
    let result = (|| -> io::Result<()> {
        let _modes = InputModes;
        execute!(io::stdout(), EnableBracketedPaste, EnableMouseCapture)?;
        while !a.quit {
            a.tick()?;
            terminal.draw(|f| ui::draw(f, &mut a))?;
            if event::poll(Duration::from_millis(if a.screen == app::Screen::Typing {
                33
            } else {
                200
            }))? {
                match event::read()? {
                    Event::Key(k) if k.kind != KeyEventKind::Release => {
                        a.tick()?;
                        a.key(k)?;
                    }
                    Event::Mouse(event) => a.mouse(event),
                    Event::Paste(text) => a.paste_path(&text),
                    _ => {}
                }
            }
        }
        a.store.save(&a.profile)
    })();
    ratatui::restore();
    result?;
    Ok(())
}
