# CRUX build plan

Every step in order. Do not skip ahead. Each step is the minimum that unlocks the next.

---

## Step 1: Workspace scaffold

**What:** Cargo workspace, `crux --help` compiles and prints.

**Files:**
- `Cargo.toml` — workspace root
- `src/main.rs` — clap dispatch
- `src/cli.rs` — clap derive structs
- `src/error.rs` — one error type
- `rust-toolchain.toml` — pinned nightly or stable
- `.cargo/config.toml` — release profile for size

**Release profile (size first):**
```toml
[profile.release]
strip = true
lto = true
codegen-units = 1
panic = "abort"
opt-level = "z"
```

**Binary size trick:** musl static + strip + LTO + `opt-level = "z"` + `panic = "abort"` gets a clap binary under 1MB.

**Verify:** `cargo build --release` → `ls -lh target/release/crux` → under 1MB. `crux --help` prints.

---

## Step 2: Git read

**What:** Read commit history from any repo. No write. No clone. Read only.

**Files:**
- `src/git/mod.rs` — facade
- `src/git/log.rs` — enumerate commits in a range
- `src/git/graph.rs` — parent/child traversal

**Crate:** `gix` (pure Rust, no C deps, fast). Not libgit2. Size wins: no C library linked.

**Key function:** `log(range: &str) -> Vec<Commit>` where `Commit` is `{ hash, message, files_changed, timestamp }`.

**Verify:** `crux log v1.0.0..HEAD` prints commit list for any repo.

---

## Step 3: Adapter — test runner

**What:** Run a command, check if it passes or fails. That's it.

**Files:**
- `src/adapter/mod.rs` — dispatch by target type
- `src/adapter/test.rs` — run test command, capture exit code

**No crate needed.** `std::process::Command` is enough. Run the command. Exit 0 = pass. Non-zero = fail.

**Key function:** `run_target(cmd: &str) -> bool` returns pass/fail.

**Verify:** `crux who --cmd "cargo test" --from HEAD~10` finds the commit that flipped test pass/fail.

---

## Step 4: Basic blame (behavior bisect)

**What:** Binary search over commit range. At each step run the target. Find the flip commit.

**Files:**
- `src/blame/mod.rs` — facade
- `src/blame/single.rs` — binary bisect on behavior

**Algorithm:** Standard binary bisect but running the adapter (step 3) instead of checking build success.

**Verify:** On a fixture repo with a known breaking commit, `crux who` returns that exact commit.

---

## Step 5: Delta debugging minimizer

**What:** After step 4 finds the commit, minimize the diff to essential lines.

**Files:**
- `src/min/mod.rs` — facade
- `src/min/ddmin.rs` — classic ddmin over commit sets
- `src/min/hunks.rs` — minimize within diff hunks

**Algorithm:** Zeller's ddmin. Split the diff into hunks. Remove half. Check if behavior still changes. Repeat until minimal.

**Verify:** A 200-line commit minimizes to 7 lines. The 7 lines are the ones that actually changed behavior.

---

## Step 6: Backward slicing

**What:** Before ddmin, narrow the candidate lines using backward program slicing. Don't minimize the whole diff — minimize the slice.

**Files:**
- `src/slice/mod.rs` — facade
- `src/slice/ast.rs` — tree-sitter AST extraction
- `src/slice/cfg.rs` — control flow graph
- `src/slice/dfg.rs` — data flow graph
- `src/slice/filter.rs` — drop renames, reorders, comments

**Crate:** `tree-sitter` (C library via FFI, tiny). Grammars loaded on demand per detected language.

**Language detection:** Read file extensions from the diff. Map to grammar. Load grammar once per session.

**Verify:** A 500-line commit with 40 lines that touch the behavior path slices down to those 40 lines before ddmin even runs.

---

## Step 7: Diff output

**What:** Print the minimized result. Terminal human-readable + JSON for tooling.

**Files:**
- `src/report/mod.rs` — facade
- `src/report/terminal.rs` — human output
- `src/report/json.rs` — machine output

**No crate for terminal.** ANSI escape codes direct. Bold, dim, green, red. No `colored` crate. No `indicatif`. Minimal.

**Verify:** `crux who` prints the report with commit hash, file, line range, minimized diff, and environment snapshot.

---

## Step 8: Interaction faults

**What:** Two commits that are fine alone but break together.

**Files:**
- `src/blame/interaction.rs` — pair/tuple detection

**Algorithm:** Flip pairs of commits instead of one. ddmin over commit sets. This is O(n²) worst case. Rayon parallelizes the runs.

**Crate:** `rayon` for parallel bisect across cores.

**Verify:** Fixture repo where commit A and commit B are fine alone, broken together. `crux who` finds both.

---

## Step 9: Ranked blame

**What:** When the cause is ambiguous, return a ranked list with scores instead of one guess.

**Files:**
- `src/blame/ranked.rs` — probabilistic blame

**Algorithm:** Likelihood from slice overlap. Each candidate commit gets a score based on how many independent reproductions it explains.

**Verify:** On a gradual drift, returns 2-3 candidates with percentages instead of one wrong answer.

---

## Step 10: Dependency blame

**What:** Your code didn't change. The library three layers down did. Follow the trail.

**Files:**
- `src/git/blame.rs` — cross-repo blame
- Extend `src/blame/single.rs` — detect dependency boundary, follow into upstream

**Mechanism:** When the slice hits a vendored or pinned dependency, follow the lockfile to the artifact. Trace the artifact to its upstream commit if provenance is available. Blame there.

**Verify:** Fixture repo with a vendored library. The blame lands on the upstream commit, not the vendored copy.

---

## Step 11: Environment vs code separation

**What:** Distinguish "code changed" from "environment changed."

**Files:**
- `src/env/mod.rs` — facade
- `src/env/code.rs` — code fingerprint
- `src/env/system.rs` — config, toolchain, env fingerprint

**Mechanism:** Hash the code state and the environment state separately. Diff each. Report which one moved.

**Verify:** A behavior change caused by a config value change returns "environment" not "code."

---

## Step 12: History rewrite resilience

**What:** Blame through squashes and rebases back to original commits.

**Files:**
- `src/git/rewrite.rs` — patch identity matching

**Mechanism:** Match commit patches by content hash across rewrite boundaries. The squash commit is a wrapper. The original commit is the cause.

**Verify:** Fixture repo with a squashed PR. `crux who` blames the original commit, not the squash.

---

## Step 13: Deterministic replay

**What:** Every finding reproduces bit-identically in a pinned world.

**Files:**
- `src/sandbox/mod.rs` — facade
- `src/sandbox/local.rs` — local sandbox (process isolation)
- `src/sandbox/snapshot.rs` — pin toolchain, seed, inputs
- `src/sandbox/verify.rs` — bit-identical check

**Mechanism:** Before running a behavior, snapshot the environment (toolchain version, env vars, seed). Store the snapshot. `crux replay <fingerprint>` restores the snapshot and reruns.

**Verify:** `crux replay <hash>` produces the exact same output as the original run.

---

## Step 14: Signature store

**What:** Local SQLite DB. Every run and conclusion stored. Same question twice = lookup.

**Files:**
- `src/store/mod.rs` — facade
- `src/store/schema.rs` — table definitions
- `src/store/hash.rs` — blake3 content-addressed hashing
- `src/store/query.rs` — lookup, diff, export

**Crate:** `rusqlite` (bundled, statically linked). SQLite is ~600KB and battle-tested.

**Schema:** `signatures` (hash, behavior, state, timestamp), `runs` (signature_hash, commit_hash, result, fingerprint), `conclusions` (run_hash, blame_hash, lines, confidence).

**Verify:** `crux index list` shows stored signatures. Second `crux who` on the same behavior is instant (lookup, not search).

---

## Step 15: Init command

**What:** Scan repo, detect languages, suggest behavior targets.

**Files:**
- `src/init.rs` — scan + detect

**Mechanism:** Walk the repo tree. Detect languages by extension. Find test files by name pattern. Suggest `cargo test`, `pytest`, `go test`, `npm test` based on what's found.

**Verify:** `crux init` in any repo prints suggested targets.

---

## Step 16: Diff command

**What:** Compare two revisions. List which behaviors changed. Minimized diff for each.

**Files:**
- Extend `src/cli.rs` — add `diff` subcommand
- Extend `src/blame/single.rs` — run blame for each changed behavior

**Verify:** `crux diff v1.0.0..v1.1.0` lists all behaviors that changed with minimized diffs.

---

## Step 17: Replay command

**What:** Rerun a finding in a pinned world.

**Files:**
- Extend `src/cli.rs` — add `replay` subcommand
- Extend `src/sandbox/` — replay logic

**Verify:** `crux replay <fingerprint>` reproduces the original result.

---

## Step 18: Doctor command

**What:** Health check. Harness reproducibility. Slicing coverage. Index integrity.

**Files:**
- Extend `src/cli.rs` — add `doctor` subcommand

**Verify:** `crux doctor` runs all checks and prints pass/fail.

---

## Step 19: Fuzz targets

**What:** Fuzz the parsers, the git reader, and the minimizer.

**Files:**
- `fuzz/targets/git_parse.rs`
- `fuzz/targets/slice_parse.rs`
- `fuzz/targets/ddmin.rs`

**Tool:** `cargo-fuzz` (libFuzzer).

**Verify:** `cargo fuzz run git_parse` runs for 10 minutes without crashes.

---

## Step 20: Benchmarks

**What:** Measure ddmin speed, slicing speed, hashing speed.

**Files:**
- `benches/min_bench.rs`
- `benches/slice_bench.rs`
- `benches/hash_bench.rs`

**Tool:** `criterion`.

**Verify:** `cargo bench` runs. Numbers are in the output. No regressions between commits.

---

## Step 21: Integration tests

**What:** End-to-end tests on fixture repos.

**Files:**
- `tests/who_test.rs`
- `tests/diff_test.rs`
- `tests/min_test.rs`
- `tests/slice_test.rs`
- `tests/store_test.rs`
- `tests/replay_test.rs`
- `tests/fixtures/` — small git repos with known breaking commits

**Fixture repos:** Three small repos (20 commits each) with known behavior changes. One simple, one with interaction faults, one with dependency chain.

**Verify:** `cargo test` passes. All fixture repos produce expected output.

---

## Step 22: Cross-compile + static linking

**What:** Build for Linux, macOS, Windows. Static musl on Linux.

**Files:**
- Extend `.cargo/config.toml` — cross-compile targets
- GitHub Actions workflow for CI

**Targets:**
- `x86_64-unknown-linux-musl` — static, no runtime deps
- `aarch64-unknown-linux-musl` — ARM static
- `x86_64-apple-darwin` — macOS
- `x86_64-pc-windows-msvc` — Windows

**Verify:** `cross build --release --target x86_64-unknown-linux-musl` produces a static binary under 2MB.

---

## Step 23: Release signing

**What:** Sign binaries. Publish checksums.

**Mechanism:** `minisign` or `cosign`. SHA256 checksums for each binary.

**Verify:** Download binary. Verify signature. Binary runs.

---

## Build order summary

| Step | Unlocks | Binary size impact |
|---|---|---|
| 1 | everything | baseline ~800KB |
| 2 | git reading | +200KB (gix) |
| 3 | behavior checking | +0 (std only) |
| 4 | basic bisect | +0 |
| 5 | minimization | +0 |
| 6 | slicing | +300KB (tree-sitter) |
| 7 | output | +0 |
| 8 | interaction faults | +50KB (rayon) |
| 9 | ranked blame | +0 |
| 10 | dependency blame | +0 |
| 11 | env vs code | +0 |
| 12 | rewrite resilience | +0 |
| 13 | replay | +0 |
| 14 | signature store | +300KB (SQLite) |
| 15 | init | +0 |
| 16 | diff | +0 |
| 17 | replay cmd | +0 |
| 18 | doctor | +0 |
| 19 | fuzz | +0 (dev only) |
| 20 | benches | +0 (dev only) |
| 21 | tests | +0 (dev only) |
| 22 | cross-compile | -200KB (musl strip) |
| 23 | release signing | +0 |

**Estimated final binary:** ~1.5MB static musl. Under 2MB target.
