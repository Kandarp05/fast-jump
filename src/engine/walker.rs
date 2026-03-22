use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crossbeam_channel::Sender;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ignore::WalkBuilder;

use crate::engine::score;

const MIN_SCORE_THRESHOLD: i64 = 10;

pub fn search_disk(
    query: String,
    tx_res: Sender<Vec<String>>,
    kill_switch: Arc<AtomicBool>,
    dir: String,
    max_list_size: u16,
) {
    let matcher = SkimMatcherV2::default();
    let mut results: Vec<(i64, String)> = Vec::new();
    let walker = WalkBuilder::new(&dir)
        .hidden(true)
        .parents(true)
        .ignore(true)
        .build();

    for result in walker {
        if kill_switch.load(Ordering::Relaxed) {
            return;
        }

        if let Ok(entry) = result
            && entry.file_type().is_some_and(|f| f.is_dir())
        {
            let path = entry.path().to_string_lossy().to_string();
            process_entry(
                &matcher,
                &path,
                &query,
                &mut results,
                max_list_size,
                &tx_res,
            );
        }
    }
}

fn process_entry(
    matcher: &SkimMatcherV2,
    path: &str,
    query: &str,
    results: &mut Vec<(i64, String)>,
    max_list_size: u16,
    tx_res: &Sender<Vec<String>>,
) {
    if let Some((raw_score, indices)) = matcher.fuzzy_indices(path, query) {
        let final_score = score::apply_heuristics(path, raw_score, &indices);

        if final_score < MIN_SCORE_THRESHOLD && score::is_redundant(path, final_score, results) {
            return;
        }

        results.push((final_score, path.to_string()));
        results.sort_unstable_by(|a, b| match b.0.cmp(&a.0) {
            std::cmp::Ordering::Equal => a.1.len().cmp(&b.1.len()),
            o => o,
        });

        results.truncate(max_list_size as usize);

        let top_results: Vec<String> = results.iter().map(|(_, p)| p.clone()).collect();
        let _ = tx_res.send(top_results);
    }
}
