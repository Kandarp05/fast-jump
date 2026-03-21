use crate::engine::score;
use crossbeam_channel::Sender;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ignore::WalkBuilder;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub fn search_disk(
    query: String,
    tx_res: Sender<Vec<String>>,
    kill_switch: Arc<AtomicBool>,
    dir: String,
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

        if let Ok(entry) = result {
            if entry.file_type().map_or(false, |f| f.is_dir()) {
                let path = entry.path().to_string_lossy().to_string();

                if let Some((raw_score, indices)) = matcher.fuzzy_indices(&path, &query) {
                    let final_score = score::apply_heuristics(&path, raw_score, &indices);

                    if final_score > 10 && !score::is_redundant(&path, final_score, &results) {
                        results.push((final_score, path));
                        results.sort_unstable_by(|a, b| match b.0.cmp(&a.0) {
                            std::cmp::Ordering::Equal => a.1.len().cmp(&b.1.len()),
                            o => o,
                        });

                        results.truncate(5);
                        let top_5: Vec<String> = results.iter().map(|(_, p)| p.clone()).collect();

                        let _ = tx_res.send(top_5);
                    }
                }
            }
        }
    }
}
