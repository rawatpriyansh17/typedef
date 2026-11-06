use crate::engine::{Mode, Settings};
use rand::{Rng, SeedableRng, rngs::StdRng};
pub const WORDS: &str = include_str!("../assets/english.txt");
const QUOTES: [&str; 6] = [
    "A little practice, repeated with care, becomes a skill you can carry anywhere. Slow down, find your rhythm, and let the speed arrive.",
    "Beyond the window, rain traced quiet paths through the evening light. Inside, a warm cup of tea and a blank page waited patiently.",
    "Build something small. Make it useful. Share what you learn. The best way to understand a new idea is to give it a place in the world.",
    "The trail turned toward the sea, where silver waves folded into the shore. We walked without a map and found exactly what we needed.",
    "There is no shortcut to a steady hand. There is only the next deliberate movement, the next small correction, and the choice to begin again.",
    "Curiosity opens the door; patience keeps it open. Ask a better question, try a different path, and leave enough room to be surprised.",
];
const CODE: [&str; 4] = [
    "fn main() {\n    let message = \"hello, typedef\";\n    println!(\"{}\", message);\n}",
    "fn sum(values: &[i32]) -> i32 {\n    values.iter().filter(|n| **n > 0).sum()\n}",
    "def greet(name):\n    message = f\"Hello, {name}!\"\n    return message\n",
    "const double = (values) => {\n  return values.map((value) => value * 2);\n};",
];
pub fn generate(s: &Settings, seed: u64, weak: &[char], custom: &str) -> String {
    let mut rng = StdRng::seed_from_u64(seed);
    match s.mode {
        Mode::Quote => QUOTES[rng.random_range(0..QUOTES.len())].into(),
        Mode::Code => CODE[rng.random_range(0..CODE.len())].trim_end().into(),
        Mode::Custom => {
            if custom.is_empty() {
                "Bring your own words. Launch typedef with --file path/to/text.txt to practice any local text.".into()
            } else {
                custom.into()
            }
        }
        _ => {
            let all: Vec<&str> = WORDS.split_whitespace().collect();
            let focused: Vec<&str> = all
                .iter()
                .copied()
                .filter(|w| weak.iter().any(|c| w.contains(*c)))
                .collect();
            let pool = if s.mode == Mode::Weak && !focused.is_empty() {
                &focused
            } else {
                &all
            };
            let count = if matches!(s.mode, Mode::Words | Mode::Weak) {
                s.words
            } else {
                250
            };
            (0..count)
                .map(|i| {
                    let mut w = pool[rng.random_range(0..pool.len())].to_string();
                    if s.numbers && i % 7 == 6 {
                        w = rng.random_range(0..1000).to_string();
                    }
                    if s.punctuation && i % 6 == 5 {
                        w.push([',', '.', '!', '?'][rng.random_range(0..4)]);
                    }
                    w
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}
/// Normalize terminal controls, tabs and CRLF without allowing escape injection.
pub fn normalize(s: &str) -> String {
    s.replace("\r\n", "\n")
        .replace('\t', "    ")
        .chars()
        .filter(|c| !c.is_control() || *c == '\n')
        .collect::<String>()
        .trim_end_matches('\n')
        .to_string()
}
/// Bounded UTF-8 input shared by the CLI and in-app file picker.
pub fn read_text(path: &std::path::Path) -> std::io::Result<String> {
    use std::io::{self, Read};
    if !std::fs::metadata(path)?.is_file() {
        return Err(io::Error::other("Choose a regular text file"));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(1_048_577)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 1_048_576 {
        return Err(io::Error::other("Text exceeds the 1 MiB limit"));
    }
    let text = normalize(
        &String::from_utf8(bytes).map_err(|_| io::Error::other("Text must be valid UTF-8"))?,
    );
    if text.trim().is_empty() {
        return Err(io::Error::other("Text file is empty"));
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seeded_challenges_repeat() {
        let s = Settings::default();
        assert_eq!(generate(&s, 42, &[], ""), generate(&s, 42, &[], ""));
    }
    #[test]
    fn input_is_safe() {
        assert_eq!(normalize("\u{1b}[31mhi\r\n\tx"), "[31mhi\n    x");
        assert_eq!(normalize("\tfirst\r\n  next\n"), "    first\n  next");
    }
}
