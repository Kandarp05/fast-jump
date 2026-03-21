
const DEPTH_PENALTY: i64 = 3;
const CHILD_OVERRIDE_MARGIN: i64 = 25;

pub fn apply_heuristics(path: &str, raw_score: i64) -> i64 {
    let depth = path.bytes().filter(|&b| b == b'/').count() as i64;

    raw_score - (depth * DEPTH_PENALTY)
}

pub fn is_redundant(path: &str, score: i64, current_results: &[(i64, String)]) -> bool {
    for(existing_score, existing_path) in current_results {
        if path.starts_with(existing_path) {
            if score <= existing_score + CHILD_OVERRIDE_MARGIN {
                return true;
            }
        }
    }
    false
}