//! Identity of the benchmark suite a result was produced against.
//!
//! Leaderboard rows are only comparable when they were produced by the same
//! set of tasks with the same setup, assertions and limits. A bare task count
//! is not enough: editing one `test.sh` makes an older 50/50 mean something
//! different. So every report carries a digest of the suite it ran.
//!
//! The digest must stay byte-identical to `scripts/leaderboard-lib.mjs`, which
//! recomputes it when a submission is validated in CI. Change both together and
//! bump `TASKSET_SCHEME` so existing submissions are re-checked rather than
//! silently compared against a different identity.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

use crate::types::BenchmarkTask;

pub const TASKSET_SCHEME: &str = "v1";

/// Files that define a task's difficulty, in the order they are hashed.
pub const TASKSET_FILES: [&str; 4] = ["prompt.txt", "setup.sh", "test.sh", "meta.sh"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TasksetIdentity {
    /// Human-readable id, e.g. `v1-50-1a2b3c4d`. What the leaderboard sorts by.
    pub id: String,
    /// Full hex digest of the suite manifest.
    pub hash: String,
    /// Number of tasks in the suite.
    pub size: usize,
}

pub fn hex_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

/// Digest of one task file, or `-` when the file is absent. Absence is part of
/// the identity: adding a previously missing `setup.sh` must change the hash.
fn file_digest(path: &Path) -> String {
    match fs::read(path) {
        Ok(bytes) => hex_digest(&bytes),
        Err(_) => "-".to_string(),
    }
}

/// The manifest string that is hashed. Sorted by directory name so the digest
/// does not depend on filesystem iteration order.
pub fn manifest(tasks: &[BenchmarkTask]) -> Result<String> {
    let mut rows: Vec<String> = Vec::with_capacity(tasks.len());
    for task in tasks {
        let mut row = task
            .task_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(task.id.as_str())
            .to_string();
        for file in TASKSET_FILES {
            row.push(':');
            row.push_str(&file_digest(&task.task_dir.join(file)));
        }
        rows.push(row);
    }

    rows.sort();
    Ok(rows.into_iter().map(|row| format!("{}\n", row)).collect())
}

pub fn compute(tasks: &[BenchmarkTask]) -> Result<TasksetIdentity> {
    let manifest = manifest(tasks)?;
    let hash = hex_digest(manifest.as_bytes());
    Ok(TasksetIdentity {
        id: format!("{}-{}-{}", TASKSET_SCHEME, tasks.len(), &hash[..8]),
        hash,
        size: tasks.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::load_all_tasks;

    /// Locks the digest to the value `scripts/leaderboard-lib.mjs` computes for the
    /// same suite. CI runs both; if they drift, submissions start being rejected.
    /// Update this constant only together with a TASKSET_SCHEME bump.
    const EXPECTED_TASKSET_HASH: &str =
        "7ecd749976c413ce1f51daf4dd09ca9e987548468c4bcf40430aad44582f3f16";

    #[test]
    fn taskset_digest_matches_validator() {
        let tasks = load_all_tasks(Path::new("tasks")).expect("load tasks");
        let identity = compute(&tasks).expect("compute taskset");
        assert_eq!(identity.hash, EXPECTED_TASKSET_HASH);
        assert_eq!(identity.size, 50);
        assert_eq!(identity.id, "v1-50-7ecd7499");
    }

    #[test]
    fn digest_is_order_independent() {
        let tasks = load_all_tasks(Path::new("tasks")).expect("load tasks");
        let mut reversed = tasks.clone();
        reversed.reverse();
        assert_eq!(
            compute(&tasks).unwrap().hash,
            compute(&reversed).unwrap().hash
        );
    }
}
