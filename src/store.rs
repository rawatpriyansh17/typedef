//! Local persistence, with an exclusive process lock and atomic replacement.
use crate::engine::{KeyStat, Session, Settings};
use chrono::{Local, NaiveDate};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::PathBuf,
};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub date: NaiveDate,
    pub category: String,
    pub wpm: f64,
    pub raw: f64,
    pub accuracy: f64,
    pub seconds: f64,
    pub xp: u64,
    pub failed: bool,
    #[serde(default)]
    pub qualified: bool,
}
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub settings: Settings,
    pub history: Vec<Record>,
    pub keys: BTreeMap<char, KeyStat>,
    pub xp: u64,
    pub achievements: Vec<String>,
    pub quest_claims: BTreeMap<NaiveDate, Vec<String>>,
}
impl Profile {
    pub fn weak(&self) -> Vec<char> {
        let mut keys: Vec<_> = self
            .keys
            .iter()
            .filter(|(c, k)| !c.is_whitespace() && k.errors > 0)
            .collect();
        keys.sort_by(|a, b| {
            let rate = |k: &KeyStat| k.errors as f64 / k.hits.max(1) as f64;
            rate(b.1).total_cmp(&rate(a.1))
        });
        keys.iter().take(5).map(|(c, _)| **c).collect()
    }
    pub fn level(&self) -> u64 {
        1 + self.xp / 500
    }
    pub fn rank(&self) -> String {
        if self.legend_cycle() > 0 {
            format!("legend · cycle {}", self.legend_cycle())
        } else {
            crate::progression::RANKS[crate::progression::rank(self.level())]
                .1
                .into()
        }
    }
    pub fn streak(&self) -> u64 {
        self.streak_on(Local::now().date_naive())
    }
    pub(crate) fn streak_on(&self, today: NaiveDate) -> u64 {
        let mut day = today;
        let days: std::collections::BTreeSet<_> = self
            .history
            .iter()
            .filter(|r| !r.failed)
            .map(|r| r.date)
            .collect();
        let has = |d| days.contains(&d);
        if !has(day) {
            day = day.pred_opt().unwrap_or(day);
        }
        let mut n = 0;
        while has(day) {
            n += 1;
            let Some(previous) = day.pred_opt() else {
                break;
            };
            day = previous;
        }
        n
    }
    pub fn activity(&self) -> BTreeMap<NaiveDate, usize> {
        let mut days = BTreeMap::new();
        for r in &self.history {
            if !r.failed {
                *days.entry(r.date).or_default() += 1;
            }
        }
        days
    }
    pub fn longest_streak(&self) -> usize {
        let mut previous: Option<NaiveDate> = None;
        let (mut run, mut best) = (0, 0);
        for day in self.activity().keys() {
            run = if previous.and_then(|p| p.succ_opt()) == Some(*day) {
                run + 1
            } else {
                1
            };
            best = best.max(run);
            previous = Some(*day);
        }
        best
    }
    pub fn category(s: &Settings) -> String {
        let base = format!(
            "{} / {} / {} / p{} n{}",
            s.mode.label(),
            if matches!(
                s.mode,
                crate::engine::Mode::Time | crate::engine::Mode::Daily
            ) {
                s.seconds as usize
            } else {
                s.words
            },
            s.difficulty.label(),
            s.punctuation as u8,
            s.numbers as u8
        );
        if s.blind || s.no_backspace || s.stop_on_error {
            format!(
                "{base} / b{} c{} s{}",
                s.blind as u8, s.no_backspace as u8, s.stop_on_error as u8
            )
        } else {
            base
        }
    }
    pub fn best(&self, s: &Settings) -> f64 {
        let category = Self::category(s);
        self.history
            .iter()
            .filter(|r| r.category == category && !r.failed)
            .map(|r| r.wpm)
            .fold(0., f64::max)
    }
    #[cfg(test)]
    pub fn record(&mut self, s: &Session) -> (u64, bool) {
        self.record_on(s, Local::now().date_naive())
    }
    pub(crate) fn record_on(&mut self, s: &Session, today: NaiveDate) -> (u64, bool) {
        let previous_tracks = self.milestones();
        let pb = !s.failed && s.wpm() > self.best(&s.settings);
        let xp = if s.failed {
            0
        } else {
            (s.correct() as f64 / 5. * (s.accuracy() / 100.)).floor() as u64
        };
        self.xp += xp;
        for (c, k) in &s.keys {
            let total = self.keys.entry(*c).or_default();
            total.hits += k.hits;
            total.errors += k.errors;
        }
        self.history.push(Record {
            date: today,
            category: Self::category(&s.settings),
            wpm: s.wpm(),
            raw: s.raw(),
            accuracy: s.accuracy(),
            seconds: s.elapsed,
            xp,
            failed: s.failed,
            qualified: !s.failed && s.elapsed >= 10. && s.correct() >= 25,
        });
        self.unlock_badges(s, today);
        let mut bonus = 0;
        if self.history.last().is_some_and(|r| r.qualified) {
            for q in self.quests(today) {
                if !q.claimed && q.current >= q.target {
                    self.quest_claims
                        .entry(today)
                        .or_default()
                        .push(q.id.into());
                    bonus += q.reward;
                }
            }
        }
        // Historical tiers are recognized, but only newly crossed thresholds pay XP.
        // Deriving the before/after tiers from persisted history prevents reclaims after restart.
        if self.history.last().is_some_and(|r| r.qualified) {
            for (before, after) in previous_tracks.iter().zip(self.milestones()) {
                bonus += (after.tier() - before.tier()) * after.reward;
            }
        }
        self.xp += bonus;
        if let Some(record) = self.history.last_mut() {
            record.xp += bonus;
        }
        (xp + bonus, pb)
    }
    pub fn records(&self) -> Vec<&Record> {
        let mut records: BTreeMap<&str, &Record> = BTreeMap::new();
        for record in self.history.iter().filter(|r| !r.failed) {
            let best = records.entry(record.category.as_str()).or_insert(record);
            if record.wpm > best.wpm {
                *best = record;
            }
        }
        records.into_values().collect()
    }
    pub fn recent_delta(&self, s: &Settings) -> Option<f64> {
        let category = Self::category(s);
        let recent: Vec<_> = self
            .history
            .iter()
            .rev()
            .filter(|r| !r.failed && r.category == category)
            .take(10)
            .collect();
        if recent.len() < 10 {
            return None;
        }
        Some(
            recent[..5].iter().map(|r| r.wpm).sum::<f64>() / 5.
                - recent[5..].iter().map(|r| r.wpm).sum::<f64>() / 5.,
        )
    }
}
pub struct Store {
    pub dir: PathBuf,
    _lock: File,
}
impl Store {
    pub fn open(override_dir: Option<PathBuf>) -> io::Result<Self> {
        let dir = override_dir
            .or_else(|| std::env::var_os("TYPEDEF_DATA_DIR").map(PathBuf::from))
            .or_else(|| {
                directories::ProjectDirs::from("dev", "typedef", "typedef")
                    .map(|p| p.data_local_dir().to_path_buf())
            })
            .ok_or_else(|| {
                io::Error::other("Cannot locate data directory; set TYPEDEF_DATA_DIR")
            })?;
        fs::create_dir_all(&dir)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(dir.join("profile.lock"))?;
        lock.try_lock_exclusive().map_err(|_|io::Error::other("Another typedef process is using this profile. Use --data-dir for a separate profile."))?;
        Ok(Self { dir, _lock: lock })
    }
    pub fn load(&self) -> io::Result<Profile> {
        match fs::read(self.dir.join("profile.json")) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| {
                io::Error::other(format!(
                    "Profile could not be read; original file preserved at {}: {e}",
                    self.dir.display()
                ))
            }),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Profile::default()),
            Err(e) => Err(e),
        }
    }
    pub fn save(&self, p: &Profile) -> io::Result<()> {
        let bytes = serde_json::to_vec_pretty(p)?;
        let tmp = self.dir.join("profile.tmp");
        let mut file = File::create(&tmp)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(tmp, self.dir.join("profile.json"))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_runs_award_nothing() {
        let mut p = Profile::default();
        let mut s = Session::new("abc".into(), Settings::default());
        s.input('a');
        s.failed = true;
        assert_eq!(p.record(&s), (0, false));
        assert!(p.achievements.is_empty());
    }
    #[test]
    fn personal_bests_are_separate() {
        let mut p = Profile::default();
        let mut s = Session::new("abc".into(), Settings::default());
        s.input('a');
        s.tick(1.);
        p.record(&s);
        let other = Settings {
            seconds: 60,
            ..Settings::default()
        };
        assert_eq!(p.best(&other), 0.);
    }
}

#[cfg(test)]
mod persistence_tests {
    use super::*;
    #[test]
    fn roundtrip_lock_and_corruption_preservation() {
        let dir = std::env::temp_dir().join(format!("typedef-store-{}", std::process::id()));
        let store = Store::open(Some(dir.clone())).unwrap();
        assert!(Store::open(Some(dir.clone())).is_err());
        let p = Profile {
            xp: 1234,
            ..Profile::default()
        };
        store.save(&p).unwrap();
        assert_eq!(store.load().unwrap().xp, 1234);
        std::fs::write(dir.join("profile.json"), b"broken").unwrap();
        assert!(store.load().is_err());
        assert_eq!(std::fs::read(dir.join("profile.json")).unwrap(), b"broken");
        drop(store);
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[cfg(test)]
mod activity_tests {
    use super::*;
    #[test]
    fn activity_excludes_failed_and_longest_handles_gaps() {
        let day = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let mut p = Profile::default();
        for (d, failed) in [(0, false), (0, false), (1, false), (2, true), (4, false)] {
            p.history.push(Record {
                date: day + chrono::Duration::days(d),
                category: "time".into(),
                wpm: 30.,
                raw: 40.,
                accuracy: 90.,
                seconds: 30.,
                xp: 10,
                failed,
                qualified: false,
            });
        }
        assert_eq!(p.activity().get(&day), Some(&2));
        assert_eq!(p.activity().len(), 3);
        assert_eq!(p.longest_streak(), 2);
    }
    #[test]
    fn modifiers_do_not_mix_with_legacy_personal_bests() {
        let normal = Settings::default();
        let blind = Settings {
            blind: true,
            ..Settings::default()
        };
        assert_ne!(Profile::category(&normal), Profile::category(&blind));
        assert_eq!(Profile::category(&normal), "time / 30 / normal / p0 n0");
    }
}

#[cfg(test)]
mod progression_tests {
    use super::*;
    fn run(seconds: u64, correct: usize) -> Session {
        let mut s = Session::new(
            "a".repeat(correct + 1),
            Settings {
                seconds,
                ..Settings::default()
            },
        );
        for _ in 0..correct {
            s.input('a');
        }
        s.tick(seconds as f64);
        assert!(s.finished);
        s
    }
    #[test]
    fn quests_award_once_survive_reload_and_reset_each_day() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 5).unwrap();
        let mut p = Profile::default();
        let s = run(60, 300);
        assert_eq!(p.record_on(&s, today).0, 120); // 60 base + 60 precision
        assert_eq!(p.record_on(&s, today).0, 60);
        assert_eq!(p.record_on(&s, today).0, 180); // 60 base + 40 warmup + 80 endurance
        assert_eq!(p.quest_claims[&today].len(), 3);
        let mut p: Profile = serde_json::from_slice(&serde_json::to_vec(&p).unwrap()).unwrap();
        assert_eq!(p.record_on(&s, today).0, 60);
        assert_eq!(p.xp, p.history.iter().map(|r| r.xp).sum::<u64>());
        let tomorrow = today.succ_opt().unwrap();
        assert!(
            p.quests(tomorrow)
                .iter()
                .all(|q| q.current == 0 && !q.claimed)
        );
        assert_eq!(p.record_on(&s, tomorrow).0, 120);
        assert_eq!(p.quest_claims[&tomorrow], vec!["precision"]);
    }
    #[test]
    fn short_tiny_or_failed_runs_cannot_farm_new_rewards() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 5).unwrap();
        let mut p = Profile::default();
        for s in [run(1, 300), run(120, 1)] {
            p.record_on(&s, today);
        }
        let mut failed = run(120, 300);
        failed.failed = true;
        assert_eq!(p.record_on(&failed, today), (0, false));
        assert!(p.quests(today).iter().all(|q| q.current == 0 && !q.claimed));
        assert!(!p.achievements.contains(&"triple digits".into()));
        assert!(!p.achievements.contains(&"marathon".into()));
    }
    #[test]
    fn recurring_tiers_cross_once_and_continue_after_reload() {
        let day = NaiveDate::from_ymd_opt(2026, 11, 5).unwrap();
        let mut p = Profile::default();
        let s = run(30, 150);
        for _ in 0..9 {
            p.record_on(&s, day);
        }
        assert_eq!(p.milestones()[0].tier(), 0);
        let reward = p.record_on(&s, day).0;
        assert_eq!(reward, 30 + 50 + 60 + 80); // base + rhythm + precision + flawless
        assert_eq!(p.milestone_tiers(), 3);
        let mut p: Profile = serde_json::from_slice(&serde_json::to_vec(&p).unwrap()).unwrap();
        assert_eq!(p.record_on(&s, day).0, 30);
        for _ in 0..8 {
            p.record_on(&s, day);
        }
        assert_eq!(p.record_on(&s, day).0, 30 + 50 + 60 + 80);
        assert_eq!(p.milestones()[0].tier(), 2);
        assert_eq!(p.milestones()[0].progress(), 0);
        assert_eq!(p.xp, p.history.iter().map(|r| r.xp).sum::<u64>());
        let mut failed = run(3600, 300);
        failed.failed = true;
        assert_eq!(p.record_on(&failed, day).0, 0);
        assert_eq!(p.milestones()[0].tier(), 2);
    }
    #[test]
    fn tiers_have_no_catalog_ceiling_and_old_progress_is_not_repaid() {
        let day = NaiveDate::from_ymd_opt(2026, 11, 5).unwrap();
        let mut p = Profile::default();
        p.record_on(&run(30, 150), day);
        let record = p.history[0].clone();
        p.history = vec![record; 10_009];
        p.quest_claims.insert(
            day,
            vec!["warmup".into(), "precision".into(), "endurance".into()],
        );
        assert_eq!(p.milestones()[0].tier(), 1000);
        assert_eq!(p.milestones()[0].progress(), 9);
        assert_eq!(p.record_on(&run(30, 150), day).0, 220);
        assert_eq!(p.milestones()[0].tier(), 1001);
        p.xp = 75_000;
        assert_eq!(p.rank(), "legend · cycle 2");
        assert_eq!(p.next_rank(), "legend cycle 3 at level 201");
    }
    #[test]
    fn endurance_can_cross_multiple_tiers_and_ritual_counts_distinct_days() {
        let day = NaiveDate::from_ymd_opt(2026, 11, 5).unwrap();
        let mut p = Profile::default();
        assert_eq!(p.record_on(&run(3600, 300), day).0, 60 + 60 + 80 + 400);
        assert_eq!(p.milestones()[3].tier(), 4);
        assert_eq!(p.milestones()[4].total, 1);
        p.record_on(&run(60, 300), day);
        assert_eq!(p.milestones()[4].total, 1);
        for i in 1..5 {
            p.record_on(&run(60, 300), day + chrono::Duration::days(i));
        }
        assert_eq!(p.milestones()[4].tier(), 1);
        let mut code = run(60, 300);
        code.settings.mode = crate::engine::Mode::Code;
        for _ in 0..5 {
            p.record_on(&code, day);
        }
        assert_eq!(p.milestones()[5].tier(), 1);
    }
    #[test]
    fn legacy_profiles_keep_progress_and_discard_audio_settings() {
        let mut p: Profile = serde_json::from_str(r#"{"settings":{"theme":2,"sound":3,"volume":80},"xp":2100,"achievements":["first flight"],"history":[{"date":"2026-11-05","category":"time / 30 / normal / p0 n0","wpm":50,"raw":55,"accuracy":98,"seconds":30,"xp":20,"failed":false}]}"#).unwrap();
        p.settings.sanitize();
        assert_eq!(p.xp, 2100);
        assert_eq!(p.level(), 5);
        assert_eq!(p.settings.theme, 2);
        assert_eq!(p.achievements, vec!["first flight"]);
        assert_eq!(p.history.len(), 1);
        assert!(!p.history[0].qualified);
        assert_eq!(p.best(&Settings::default()), 50.);
        assert!(p.settings.large_results);
        let saved = serde_json::to_value(&p).unwrap();
        assert!(saved["settings"].get("sound").is_none());
        assert!(saved["settings"].get("volume").is_none());
    }
    #[test]
    fn personal_records_and_trends_use_successful_matching_settings() {
        let day = NaiveDate::from_ymd_opt(2026, 11, 5).unwrap();
        let mut p = Profile::default();
        for n in 1..=10 {
            p.record_on(&run(30, n * 5), day);
        }
        let mut failed = run(30, 500);
        failed.failed = true;
        p.record_on(&failed, day);
        p.record_on(&run(60, 300), day);
        assert_eq!(p.records().len(), 2);
        assert_eq!(p.best(&Settings::default()), 20.);
        assert_eq!(p.recent_delta(&Settings::default()), Some(10.));
        assert!(
            p.recent_delta(&Settings {
                seconds: 60,
                ..Settings::default()
            })
            .is_none()
        );
    }
    #[test]
    fn badges_unlock_once_and_ranks_keep_advancing() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 5).unwrap();
        let mut p = Profile::default();
        let s = run(60, 500);
        for i in 0..7 {
            p.record_on(&s, today + chrono::Duration::days(i));
        }
        assert!(p.achievements.contains(&"triple digits".into()));
        assert!(p.achievements.contains(&"week warrior".into()));
        assert_eq!(
            p.achievements
                .iter()
                .filter(|b| *b == "triple digits")
                .count(),
            1
        );
        assert_eq!(p.streak_on(today + chrono::Duration::days(6)), 7);
        p.xp = 50_000;
        assert_eq!((p.level(), p.rank()), (101, "legend · cycle 1".into()));
        p.xp += 500;
        assert_eq!((p.level(), p.rank()), (102, "legend · cycle 1".into()));
    }
}
