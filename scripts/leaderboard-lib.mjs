// Shared leaderboard logic for CI validation and the Astro build.
//
// The taskset digest must stay byte-identical to `src/taskset.rs`. If you change
// the hashing scheme, change both and bump TASKSET_SCHEME so older submissions
// are re-checked instead of being compared against a different identity.

import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const TASKSET_SCHEME = "v1";
export const TASKSET_FILES = ["prompt.txt", "setup.sh", "test.sh", "meta.sh"];
export const SUBMISSION_SCHEMA_VERSION = 1;
export const TIERS = ["community", "verified"];

/** Ranking rule, documented in one place and mirrored by `spacetime leaderboard`. */
export const RANKING_DESCRIPTION =
  "pass rate, then cost per resolved task, then average task time";

function sha256(buffer) {
  return createHash("sha256").update(buffer).digest("hex");
}

function isTaskDir(dir) {
  return ["prompt.txt", "meta.sh", "meta.toml"].some((file) =>
    fs.existsSync(path.join(dir, file)),
  );
}

/**
 * Identity of the benchmark suite currently in `tasks/`.
 * Missing files hash as "-", so adding a previously absent script changes the id.
 */
export function computeTaskset(tasksDir) {
  const names = fs
    .readdirSync(tasksDir, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .filter((name) => isTaskDir(path.join(tasksDir, name)))
    .sort();

  const rows = names.map((name) => {
    let row = name;
    for (const file of TASKSET_FILES) {
      const filePath = path.join(tasksDir, name, file);
      const digest = fs.existsSync(filePath) ? sha256(fs.readFileSync(filePath)) : "-";
      row += `:${digest}`;
    }
    return row;
  });

  const manifest = rows.map((row) => `${row}\n`).join("");
  const hash = sha256(Buffer.from(manifest, "utf8"));

  return {
    id: `${TASKSET_SCHEME}-${names.length}-${hash.slice(0, 8)}`,
    hash,
    size: names.length,
    taskIds: names,
  };
}

export function recount(run) {
  const results = Array.isArray(run?.results) ? run.results : [];
  const total = results.length;
  const passed = results.filter((result) => result?.passed === true).length;
  return {
    total,
    passed,
    passRate: total === 0 ? 0 : (passed / total) * 100,
  };
}

function isFiniteNumber(value) {
  return typeof value === "number" && Number.isFinite(value);
}

/**
 * Validate one submission against the current suite. Denormalized totals are
 * never trusted: they must agree with the per-task results in the same file.
 */
export function validateSubmission(submission, taskset) {
  const errors = [];
  const warnings = [];

  if (!submission || typeof submission !== "object") {
    return { errors: ["file is not a JSON object"], warnings };
  }
  if (submission.schema_version !== SUBMISSION_SCHEMA_VERSION) {
    errors.push(`schema_version must be ${SUBMISSION_SCHEMA_VERSION}`);
  }
  if (typeof submission.submitted_by !== "string" || !submission.submitted_by.trim()) {
    errors.push("submitted_by must be a non-empty string");
  }
  if (!TIERS.includes(submission.tier)) {
    errors.push(`tier must be one of ${TIERS.join(", ")}`);
  }
  if (submission.source_url != null && typeof submission.source_url !== "string") {
    errors.push("source_url must be a string or null");
  }

  const run = submission.run;
  if (!run || typeof run !== "object") {
    errors.push("run must be the report produced by `spacetime eval-all`");
    return { errors, warnings };
  }

  if (typeof run.harness !== "string" || !run.harness.trim()) {
    errors.push("run.harness is required so the row can be ranked per agent");
  }
  if (!isFiniteNumber(run.total_duration_secs) || run.total_duration_secs < 0) {
    errors.push("run.total_duration_secs must be a non-negative number");
  }
  for (const field of ["total_cost_usd", "cost_per_resolved_task"]) {
    if (run[field] != null && !isFiniteNumber(run[field])) {
      errors.push(`run.${field} must be a number when present`);
    }
  }

  const counts = recount(run);
  if (counts.total === 0) {
    errors.push("run.results is empty");
    return { errors, warnings };
  }

  if (counts.total !== run.total_tasks) {
    errors.push(`run.total_tasks is ${run.total_tasks} but results has ${counts.total} entries`);
  }
  if (counts.passed !== run.passed_tasks) {
    errors.push(
      `run.passed_tasks is ${run.passed_tasks} but ${counts.passed} results are marked passed`,
    );
  }
  if (!isFiniteNumber(run.pass_rate) || Math.abs(counts.passRate - run.pass_rate) > 0.01) {
    errors.push(`run.pass_rate is ${run.pass_rate} but results compute to ${counts.passRate.toFixed(2)}`);
  }

  const seen = new Set();
  for (const result of run.results) {
    const id = result?.task_id;
    if (typeof id !== "string" || !id) {
      errors.push("every result needs a task_id");
      break;
    }
    if (seen.has(id)) {
      errors.push(`task ${id} appears more than once`);
      break;
    }
    seen.add(id);
  }
  for (const id of seen) {
    if (!taskset.taskIds.includes(id)) {
      errors.push(`task ${id} is not part of taskset ${taskset.id}`);
      break;
    }
  }

  if (run.taskset_hash !== taskset.hash || run.taskset_id !== taskset.id) {
    if (run.taskset_id === taskset.id) {
      errors.push(
        `run reports taskset ${taskset.id} but a different digest: the tasks changed after this run, so rerun the suite on this commit`,
      );
    } else {
      errors.push(
        `run was produced against taskset ${run.taskset_id || "(none)"} but this repo is ${taskset.id}; rerun the suite on this commit`,
      );
    }
  } else if (counts.total !== taskset.size) {
    errors.push(`suite has ${taskset.size} tasks but the run reports ${counts.total}`);
  }

  if (!run.spacetime_version) {
    warnings.push("run has no spacetime_version; it may have been produced by an older CLI");
  }

  return { errors, warnings };
}

/** Load every submission under a directory, sorted for stable output. */
export function loadSubmissions(submissionsDir) {
  if (!fs.existsSync(submissionsDir)) return [];

  const files = [];
  const walk = (dir) => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) walk(full);
      else if (entry.name.endsWith(".json")) files.push(full);
    }
  };
  walk(submissionsDir);
  files.sort();

  return files.map((file) => {
    const parseErrors = [];
    let data = null;
    try {
      data = JSON.parse(fs.readFileSync(file, "utf8"));
    } catch (error) {
      parseErrors.push(`invalid JSON: ${error.message}`);
    }
    return { file, relative: path.relative(submissionsDir, file), data, parseErrors };
  });
}

function costKey(run) {
  // Unknown cost sorts after every known cost instead of pretending to be free.
  const cost = run?.cost_per_resolved_task;
  return isFiniteNumber(cost) && cost > 0 ? cost : Number.POSITIVE_INFINITY;
}

export function compareRows(a, b) {
  if (b.pass_rate !== a.pass_rate) return b.pass_rate - a.pass_rate;
  if (costKey(a) !== costKey(b)) return costKey(a) - costKey(b);
  return a.avg_task_secs - b.avg_task_secs;
}

export function toRow(entry, taskset) {
  const { data, relative } = entry;
  const run = data.run;
  const counts = recount(run);
  const cost = isFiniteNumber(run.cost_per_resolved_task) ? run.cost_per_resolved_task : null;

  return {
    agent: run.harness,
    model: run.model || null,
    label: run.model ? `${run.harness} (${run.model})` : run.harness,
    tier: data.tier,
    passed: counts.passed,
    total: counts.total,
    pass_rate: Number(counts.passRate.toFixed(2)),
    cost_per_resolved_task: cost,
    avg_task_secs: counts.total === 0 ? 0 : Number((run.total_duration_secs / counts.total).toFixed(2)),
    total_secs: Number((run.total_duration_secs ?? 0).toFixed(1)),
    total_tokens: run.total_tokens ?? null,
    submitted_by: data.submitted_by,
    source_url: data.source_url ?? null,
    notes: data.notes ?? null,
    timestamp: run.timestamp ?? null,
    taskset_id: run.taskset_id,
    submission_file: relative,
  };
}

/**
 * Build the aggregate the site renders: one group per taskset, each ranked.
 * Runs from different tasksets are never compared with each other.
 */
export function buildLeaderboard(submissionsDir, taskset, { now = new Date() } = {}) {
  const entries = loadSubmissions(submissionsDir).filter(
    (entry) => entry.data && !entry.parseErrors.length,
  );

  const groups = new Map();
  for (const entry of entries) {
    const id = entry.data?.run?.taskset_id;
    if (typeof id !== "string" || !id) continue;
    if (!groups.has(id)) groups.set(id, []);
    groups.get(id).push(toRow(entry, taskset));
  }

  const ordered = [...groups.entries()]
    .map(([taskset_id, rows]) => ({
      taskset_id,
      is_current: taskset_id === taskset.id,
      size: rows[0]?.total ?? 0,
      rows: rows.sort(compareRows),
    }))
    .sort((a, b) => {
      if (a.is_current !== b.is_current) return a.is_current ? -1 : 1;
      return a.taskset_id < b.taskset_id ? -1 : 1;
    });

  const allRows = ordered.flatMap((group) => group.rows);

  return {
    generated_at: now.toISOString(),
    ranking: RANKING_DESCRIPTION,
    current_taskset: { id: taskset.id, hash: taskset.hash, size: taskset.size },
    totals: {
      runs: allRows.length,
      verified: allRows.filter((row) => row.tier === "verified").length,
      community: allRows.filter((row) => row.tier === "community").length,
      agents: new Set(allRows.map((row) => `${row.agent}::${row.model ?? ""}`)).size,
    },
    groups: ordered,
  };
}
