//! Transparent local progression. Old XP and earned badges are never reset.
use crate::{engine::Session, store::Profile};
use chrono::NaiveDate;
pub const BADGES: [(&str, &str); 16] = [
    ("first flight", "finish a test"),
    ("clean slate", "finish with 100% accuracy"),
    ("mile a minute", "reach 60 wpm"),
    ("century club", "100 correct keys in a row"),
    ("showing up", "practice 3 days in a row"),
    ("ten strong", "finish 10 tests"),
    ("speed of thought", "reach 80 wpm"),
    ("triple digits", "reach 100 wpm"),
    ("precision pilot", "98% accuracy on a 60-second run"),
    ("marathon", "finish a 120-second run"),
    ("week warrior", "build a 7-day streak"),
    ("monthly ritual", "build a 30-day streak"),
    ("fifty flights", "finish 50 tests"),
    ("centurion", "finish 100 tests"),
    ("polyglot", "complete 4 different modes"),
    ("untouchable", "250 correct keys in a row"),
];
pub const RANKS: [(u64, &str); 10] = [
    (1, "seedling"),
    (3, "pathfinder"),
    (6, "flow seeker"),
    (11, "wordsmith"),
    (21, "keyboard sage"),
    (31, "virtuoso"),
    (46, "luminary"),
    (61, "architect"),
    (81, "grandmaster"),
    (101, "legend"),
];
pub fn rank(level: u64) -> usize {
    RANKS
        .iter()
        .rposition(|(min, _)| level >= *min)
        .unwrap_or(0)
}
pub struct Quest {
    pub id: &'static str,
    pub label: &'static str,
    pub current: usize,
    pub target: usize,
    pub reward: u64,
    pub claimed: bool,
}
impl Profile {
    pub fn quests(&self, date: NaiveDate) -> Vec<Quest> {
        let records: Vec<_> = self
            .history
            .iter()
            .filter(|r| r.date == date && r.qualified && !r.failed)
            .collect();
        let precision = records.iter().any(|r| r.accuracy >= 98.) as usize;
        let seconds = records.iter().map(|r| r.seconds).sum::<f64>() as usize;
        [
            ("warmup", "finish 3 qualifying tests", records.len(), 3, 40),
            ("precision", "finish at 98% accuracy", precision, 1, 60),
            ("endurance", "practice for 3 minutes", seconds, 180, 80),
        ]
        .into_iter()
        .map(|(id, label, current, target, reward)| Quest {
            id,
            label,
            current: current.min(target),
            target,
            reward,
            claimed: self
                .quest_claims
                .get(&date)
                .is_some_and(|v| v.iter().any(|s| s == id)),
        })
        .collect()
    }
    pub fn unlock_badges(&mut self, s: &Session, today: NaiveDate) {
        if s.failed {
            return;
        }
        let completed = self.history.iter().filter(|r| !r.failed).count();
        let modes: std::collections::BTreeSet<_> = self
            .history
            .iter()
            .filter(|r| !r.failed)
            .map(|r| r.category.split(" / ").next().unwrap_or(""))
            .collect();
        let conditions = [
            true,
            s.accuracy() == 100.,
            s.wpm() >= 60.,
            s.best_combo >= 100,
            self.streak_on(today) >= 3,
            completed >= 10,
            s.wpm() >= 80.,
            s.wpm() >= 100.,
            s.elapsed >= 60. && s.accuracy() >= 98.,
            s.elapsed >= 120.,
            self.streak_on(today) >= 7,
            self.streak_on(today) >= 30,
            completed >= 50,
            completed >= 100,
            modes.len() >= 4,
            s.best_combo >= 250,
        ];
        // New performance badges require a meaningful completed run; legacy six retain their semantics.
        for (i, (name, _)) in BADGES.iter().enumerate() {
            if conditions[i]
                && (i < 6 || (s.elapsed >= 10. && s.correct() >= 25))
                && !self.achievements.iter().any(|v| v == name)
            {
                self.achievements.push((*name).into());
            }
        }
    }
}

pub const TRACK_COUNT: usize = 6;
/// A repeatable badge track. Only counters and history are needed, not an ever-growing badge list.
pub struct Milestone {
    pub name: &'static str,
    pub unit: &'static str,
    pub total: u64,
    pub step: u64,
    pub reward: u64,
}
impl Milestone {
    pub fn tier(&self) -> u64 {
        self.total / self.step
    }
    pub fn progress(&self) -> u64 {
        self.total % self.step
    }
}
impl Profile {
    pub fn milestones(&self) -> [Milestone; TRACK_COUNT] {
        let (mut runs, mut seconds, mut precise, mut perfect, mut code) = (0, 0., 0, 0, 0);
        let mut days = std::collections::BTreeSet::new();
        for r in self.history.iter().filter(|r| r.qualified && !r.failed) {
            runs += 1;
            seconds += r.seconds;
            precise += u64::from(r.accuracy >= 98.);
            perfect += u64::from(r.accuracy >= 100.);
            code += u64::from(r.category.starts_with("code / "));
            days.insert(r.date);
        }
        [
            Milestone {
                name: "rhythm",
                unit: "qualifying tests",
                total: runs,
                step: 10,
                reward: 50,
            },
            Milestone {
                name: "precision",
                unit: "tests at 98%+",
                total: precise,
                step: 10,
                reward: 60,
            },
            Milestone {
                name: "flawless",
                unit: "perfect tests",
                total: perfect,
                step: 10,
                reward: 80,
            },
            Milestone {
                name: "endurance",
                unit: "seconds practiced",
                total: seconds as u64,
                step: 900,
                reward: 100,
            },
            Milestone {
                name: "ritual",
                unit: "active days",
                total: days.len() as u64,
                step: 5,
                reward: 100,
            },
            Milestone {
                name: "syntax",
                unit: "code tests",
                total: code,
                step: 5,
                reward: 75,
            },
        ]
    }
    pub fn milestone_tiers(&self) -> u64 {
        self.milestones().iter().map(Milestone::tier).sum()
    }
    pub fn legend_cycle(&self) -> u64 {
        if self.level() < 101 {
            0
        } else {
            1 + (self.level() - 101) / 50
        }
    }
    pub fn next_rank(&self) -> String {
        let tier = rank(self.level());
        RANKS
            .get(tier + 1)
            .map(|(level, name)| format!("next: {name} at level {level}"))
            .unwrap_or_else(|| {
                format!(
                    "legend cycle {} at level {}",
                    self.legend_cycle() + 1,
                    101 + self.legend_cycle() * 50
                )
            })
    }
}
