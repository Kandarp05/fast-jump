use crossbeam_channel::{Receiver, Sender, unbounded};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use ignore::{DirEntry, WalkBuilder, WalkState};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::engine::db::FrecencyDB;
use crate::engine::{EngineResult, db, score};

const MIN_SCORE_THRESHOLD: i64 = 10;
const UPDATE_INTERVAL: Duration = Duration::from_millis(25);
type ScoredPath = (i64, String);

pub fn search_disk(
    query: String,
    tx_res: Sender<EngineResult>,
    kill_switch: Arc<AtomicBool>,
    dir: String,
    db: &FrecencyDB,
    max_list_size: u16,
) {
    // Worker threads will send their results to this channel
    let (tx_worker, rx_worker) = unbounded::<(i64, String)>();

    // Spawn a thread to collect the results
    let aggregator_handle = spawn_aggregator(
        rx_worker,
        tx_res.clone(),
        kill_switch.clone(),
        max_list_size,
    );

    db::calc_based_on_frecency(db, &query, &tx_worker);

    // Configure the crawler
    let walker = WalkBuilder::new(&dir)
        .hidden(true)
        .parents(true)
        .ignore(true)
        .build_parallel();

    walker.run(|| {
        let tx_worker_clone = tx_worker.clone();
        let query_clone = query.clone();
        let kill_switch_clone = kill_switch.clone();
        let matcher = SkimMatcherV2::default();

        Box::new(move |result| {
            if kill_switch_clone.load(Ordering::Relaxed) {
                return WalkState::Quit;
            }

            if let Ok(entry) = result {
                process_single_entry(&entry, &query_clone, &matcher, &tx_worker_clone);
            }

            WalkState::Continue
        })
    });

    drop(tx_worker);
    let _ = aggregator_handle.join();

    if !kill_switch.load(Ordering::Relaxed) {
        let _ = tx_res.send(EngineResult::Done);
    }
}

fn process_single_entry(
    entry: &DirEntry,
    query: &str,
    matcher: &SkimMatcherV2,
    tx_worker: &Sender<(i64, String)>,
) {
    // Must be a directory
    if !entry.file_type().is_some_and(|f| f.is_dir()) {
        return;
    }

    // Must be a valid UTF-8 path
    let Some(path_str) = entry.path().to_str() else {
        return;
    };

    if let Some((raw_score, indices)) = matcher.fuzzy_indices(path_str, query) {
        let final_score = score::apply_heuristics(path_str, raw_score, &indices);

        // Just send anything that is not completely useless
        if final_score > 0 {
            let _ = tx_worker.send((final_score, path_str.to_string()));
        }
    }
}

fn spawn_aggregator(
    rx_worker: Receiver<(i64, String)>,
    tx_res: Sender<EngineResult>,
    kill_switch: Arc<AtomicBool>,
    max_list_size: u16,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut top_results: Vec<(i64, String)> = Vec::new();
        let mut last_update = std::time::Instant::now();
        let mut parent_set = HashSet::new();
        let mut seen_path = HashSet::new();

        while let Ok((score, path)) = rx_worker.recv() {
            if kill_switch.load(Ordering::Relaxed) {
                return;
            }

            if !seen_path.insert(path.clone()) {
                continue;
            }

            if score < MIN_SCORE_THRESHOLD
                && score::is_redundant(&path, score, &top_results, &parent_set)
            {
                continue;
            }

            top_results.push((score, path));
            top_results.sort_unstable_by(compare_scored_paths);
            top_results.truncate(max_list_size as usize);

            parent_set = score::build_parent_set(&top_results);

            if last_update.elapsed() > UPDATE_INTERVAL {
                if tx_res
                    .send(EngineResult::Update(collect_paths(&top_results)))
                    .is_err()
                {
                    kill_switch.store(true, Ordering::Relaxed);
                    return;
                }
                last_update = std::time::Instant::now();
            }
        }

        // Final payload delivery
        if !kill_switch.load(Ordering::Relaxed) {
            if tx_res
                .send(EngineResult::Update(collect_paths(&top_results)))
                .is_err()
            {
                kill_switch.store(true, Ordering::Relaxed);
            }
        }
    })
}

fn compare_scored_paths(a: &ScoredPath, b: &ScoredPath) -> std::cmp::Ordering {
    match b.0.cmp(&a.0) {
        std::cmp::Ordering::Equal => a.1.len().cmp(&b.1.len()),
        other => other,
    }
}

fn collect_paths(results: &[ScoredPath]) -> Vec<String> {
    results.iter().map(|(_, path)| path.clone()).collect()
}
