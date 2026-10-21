#!/usr/bin/env node
// Validate leaderboard submissions and optionally emit the aggregate the site builds from.
//
//   node scripts/validate-leaderboard.mjs
//   node scripts/validate-leaderboard.mjs --out website/src/data/leaderboard.json
//   node scripts/validate-leaderboard.mjs --dir /tmp/fixtures --tasks tasks
//
// Exits non-zero when any submission is invalid, so CI blocks the PR.

import fs from "node:fs";
import path from "node:path";
import process from "node:process";

import {
  buildLeaderboard,
  computeTaskset,
  loadSubmissions,
  validateSubmission,
} from "./leaderboard-lib.mjs";

const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");

function parseArgs(argv) {
  const args = {
    dir: path.join(repoRoot, "leaderboard/submissions"),
    tasks: path.join(repoRoot, "tasks"),
    out: null,
    quiet: false,
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--dir") args.dir = path.resolve(argv[++i]);
    else if (arg === "--tasks") args.tasks = path.resolve(argv[++i]);
    else if (arg === "--out") args.out = path.resolve(argv[++i]);
    else if (arg === "--quiet") args.quiet = true;
    else if (arg === "--help" || arg === "-h") {
      console.log("usage: validate-leaderboard.mjs [--dir DIR] [--tasks DIR] [--out FILE] [--quiet]");
      process.exit(0);
    } else {
      console.error(`unknown argument: ${arg}`);
      process.exit(2);
    }
  }
  return args;
}

const args = parseArgs(process.argv.slice(2));

const emit = (message) => {
  if (!args.quiet) console.log(message);
};

if (!fs.existsSync(args.tasks)) {
  console.error(`tasks directory not found: ${args.tasks}`);
  process.exit(2);
}

const taskset = computeTaskset(args.tasks);
emit(`taskset ${taskset.id} (${taskset.size} tasks)`);

const entries = loadSubmissions(args.dir);
const failures = [];
const warnings = [];
const identities = new Map();

for (const entry of entries) {
  if (entry.parseErrors.length) {
    failures.push({ file: entry.relative, errors: entry.parseErrors });
    continue;
  }

  const { errors, warnings: entryWarnings } = validateSubmission(entry.data, taskset);
  if (errors.length) {
    failures.push({ file: entry.relative, errors });
    continue;
  }
  for (const warning of entryWarnings) {
    warnings.push({ file: entry.relative, message: warning });
  }

  // One row per agent + model + taskset, or the ranking has undefined order.
  const run = entry.data.run;
  const key = `${run.taskset_id}::${run.harness.toLowerCase()}::${(run.model ?? "").toLowerCase()}`;
  if (identities.has(key)) {
    failures.push({
      file: entry.relative,
      errors: [`duplicates ${identities.get(key)} for the same agent, model and taskset`],
    });
  } else {
    identities.set(key, entry.relative);
  }
}

for (const warning of warnings) {
  console.warn(`warning: ${warning.file}: ${warning.message}`);
}

if (failures.length) {
  console.error(`\n${failures.length} invalid submission(s):\n`);
  for (const failure of failures) {
    console.error(`  ${failure.file}`);
    for (const error of failure.errors) console.error(`    - ${error}`);
  }
  console.error("");
  process.exit(1);
}

emit(`${entries.length} submission(s) valid`);

if (args.out) {
  const leaderboard = buildLeaderboard(args.dir, taskset);
  fs.mkdirSync(path.dirname(args.out), { recursive: true });
  fs.writeFileSync(args.out, `${JSON.stringify(leaderboard, null, 2)}\n`);
  emit(`wrote ${path.relative(repoRoot, args.out) || args.out}`);
}
