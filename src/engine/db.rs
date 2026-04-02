use crate::engine::score;
use crossbeam_channel::Sender;
use dirs::data_local_dir;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, SystemTime};

// Max possible size of the database
const MAX_DB_SIZE: usize = 750;

// Target size of the database after pruning
const PRUNE_TARGET: usize = 500;

// Frequency limits
const FRECENCY_VISIT_CAP: u32 = 50; // Visits needed to hit the max frequency score
const MAX_FRECENCY_BONUS: i64 = 75;

// Time brackets (in Unix Epoch seconds)
const TIME_ONE_HOUR: u64 = 3_600;
const TIME_ONE_DAY: u64 = 86_400;
const TIME_ONE_WEEK: u64 = 604_800;
const TIME_ONE_MONTH: u64 = 2_592_000;

// Recency weights (0 to 100 scale)
const WEIGHT_HOUR: u64 = 100;
const WEIGHT_DAY: u64 = 80;
const WEIGHT_WEEK: u64 = 50;
const WEIGHT_MONTH: u64 = 20;
const WEIGHT_OLDER: u64 = 5;

#[derive(Serialize, Deserialize, Clone)]
pub struct FrecencyEntry {
    pub visits: u32,
    pub last_visited: u64,
}

impl FrecencyEntry {
    pub fn new() -> Self {
        Self {
            visits: 0,
            last_visited: 0,
        }
    }
}

pub type FrecencyDB = HashMap<String, FrecencyEntry>;

pub fn load_db() -> FrecencyDB {
    let Some(local_dir) = data_local_dir() else {
        return HashMap::new();
    };

    let path = local_dir.join("fj").join("db.json");

    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn calc_based_on_frecency(db: &FrecencyDB, query: &str, tx_worker: &Sender<(i64, String)>) {
    let matcher = SkimMatcherV2::default();

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();

    for (path, entry) in db {
        if let Some((raw_score, indices)) = matcher.fuzzy_indices(path, query) {
            let frecency_rank = calculate_frecency_rank(entry, &now);
            let heuristic_score = score::apply_heuristics(path, raw_score, &indices);
            let bonus = (frecency_rank as i64) * MAX_FRECENCY_BONUS / 100;

            let score = heuristic_score + bonus;
            let _ = tx_worker.send((score, path.clone()));
        }
    }
}

fn calculate_frecency_rank(entry: &FrecencyEntry, now: &u64) -> u64 {
    let age_sec = now.saturating_sub(entry.last_visited);

    // Calculate recency score: 0 to 100
    let recency_score = if age_sec < TIME_ONE_HOUR {
        WEIGHT_HOUR
    } else if age_sec < TIME_ONE_DAY {
        WEIGHT_DAY
    } else if age_sec < TIME_ONE_WEEK {
        WEIGHT_WEEK
    } else if age_sec < TIME_ONE_MONTH {
        WEIGHT_MONTH
    } else {
        WEIGHT_OLDER
    };

    // Calculate frecency score: 0 to 100
    let visits = entry.visits.min(FRECENCY_VISIT_CAP);
    let freq_score = (100 * visits) / FRECENCY_VISIT_CAP;

    (recency_score + freq_score as u64) / 2
}

pub fn update_and_save_db(mut db: FrecencyDB, selected_path: String) {
    let Some(local_dir) = data_local_dir() else {
        return;
    };

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();

    let entry = db.entry(selected_path).or_insert(FrecencyEntry::new());
    entry.visits += 1;
    entry.last_visited = now;

    if db.len() > MAX_DB_SIZE {
        let mut entries = db.into_iter().collect::<Vec<_>>();

        // Sort by frecency in descending order
        entries.sort_by_key(|(_, entry)| Reverse(calculate_frecency_rank(entry, &now)));
        entries.truncate(PRUNE_TARGET);
        db = entries.into_iter().collect();
    }

    let path = local_dir.join("fj").join("db.json");

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Ok(json) = serde_json::to_string(&db) {
        let _ = fs::write(path, json);
    }
}
