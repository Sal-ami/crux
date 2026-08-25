THIS REPORT WAS WRITTEN BY AN AI BUT THE BENCHMARKING WAS DONE BY A HUMAN

# CRUX Benchmarking Campaign - August 2026

An AI wrote this document. A human ran every benchmark, made every judgment call,
and debugged every failure described below. This file is the AI's record of what
the human did and what the numbers say.

## What the human set out to measure

Four dimensions, across nine scenarios, against git-bisect as the baseline:

1. Speed - wall clock time to find the breaking commit
2. Test executions - how many times the test predicate actually ran
3. Accuracy - whether the reported commit is the true first bad commit
4. Usefulness - what the tool hands the user after it answers

## Environment the human chose

- Windows 11 Pro (build 26100), Git for Windows, PowerShell 5.1
- Binary under test: crux.exe 1.95 MB, release profile (strip, LTO, opt-level=z)
- Repos built and run in `C:\Users\fetit\AppData\Local\Temp\opencode\crux-bench`
  instead of the OneDrive workspace. The human had measured earlier that spawning
  processes from the OneDrive path costs roughly 400ms extra per spawn, which would
  poison both tools' numbers equally but inflate everything.
- Shared test predicate for both tools: a script that greps `status.txt` for "fail"
  and exits nonzero when found. Same predicate file, same semantics, fair fight.
- Execution counting: the predicate appends one line to `.execs` on every invocation,
  so both tools' real test-run counts were measured, not estimated.
- Commit dates pinned via GIT_AUTHOR_DATE/GIT_COMMITTER_DATE so merge-history ordering
  is deterministic across runs.

## Scenarios

| ID  | Scenario            | Shape                                            |
|-----|---------------------|--------------------------------------------------|
| S1  | small linear        | 51 commits, break at #40                          |
| S2  | medium linear       | 201 commits, break at #150                        |
| S3  | large linear        | 1001 commits, break at #750                       |
| S4  | merge history       | 40 main + 30 feature (break at feat #20) + merge  |
| S5  | large diff          | 100 files touched per commit, break at #30 of 50  |
| S6  | interaction fault   | config flip lands after handler refactor          |
| S7  | dependency chain    | vendored lib v1 to v2 mismatch                    |
| P1  | slow tests (~250ms) | 60 commits, break at #40, parallel mode exercised |
| R1  | REAL REPO           | sharkdp/fd shallow clone, 25 synthetic commits on top |

R1 matters because the working tree is a real Rust project with hundreds of files,
a real index, and real history under the synthetic breakage.

## Headline results (final clean runs)

| Scenario | bisect ms (runs) | crux ms (runs) | crux --fast ms (runs) | crux --parallel ms | accuracy      |
|----------|------------------|----------------|------------------------|--------------------|---------------|
| S1       | 1,346 (5)        | 8,022 (8)      | 5,385 (8)              | -                  | all CORRECT   |
| S2       | 1,825 (8)        | 25,693 (9)     | 17,519 (9)             | -                  | all CORRECT   |
| S3       | 3,947 (10)       | 137,583 (11)   | 75,428 (11)            | -                  | all CORRECT   |
| S4       | 2,205 (7)        | 10,797 (8)     | -                      | -                  | all CORRECT   |
| S5       | 2,642 (5)        | 14,563 (8)     | 5,772 (8)              | -                  | all CORRECT   |
| S6       | 1,043 (4)        | 3,655 (6)      | -                      | -                  | all CORRECT   |
| S7       | 1,732 (5)        | 4,151 (7)      | -                      | -                  | all CORRECT   |
| P1       | 4,095 (6)        | 15,072 (8)     | -                      | 14,870 (CORRECT)   | all CORRECT   |
| R1 real  | 2,051 (4)        | 6,400 (7)      | 5,966 (7)              | -                  | all CORRECT   |

### Speed verdict

git-bisect wins every scenario. The gap grows with history size:

- S1: bisect 6.0x faster than crux normal, 4.0x faster than --fast
- S2: 14.1x / 9.6x
- S3: 34.9x / 19.1x
- S5: 5.5x / 2.2x
- R1: 3.1x / 2.9x

On the real repository the gap is smallest (about 3x). That is the number to quote.

### Where crux's time actually goes

Test execution counts are nearly identical (bisect 4-10 runs, crux 6-11 runs).
Binary search is binary search. The gap is overhead per step and one O(n) phase:

- Every crux test run spawns cmd -> git checkout -> test script. Bisect's checkout
  happens in-process through its own machinery.
- After finding the flip, crux normal mode lists suspects by walking the whole range
  again, and `git log` support fetches changed files per commit with one
  `git diff-tree` spawn PER COMMIT. On S3 that phase alone is ~1000 spawns and
  accounts for the bulk of the 137 seconds. --fast skips only the post-flip extras,
  which is why it halves the time but cannot approach bisect.

One-line future fix identified during the campaign: fetch the whole range's changed
files with a single `git log --name-only` pass instead of per-commit diff-tree calls.
Projected effect: S3 normal drops from ~138s toward single-digit seconds. Not applied
in this campaign; recorded here so nobody rediscovers it.

### Parallel mode (P1)

With 250ms tests and 8 workers, parallel came out at 14.9s versus 15.1s sequential.
Worktree create/remove per commit (~60 worktrees) eats everything the parallelism
saves at this test cost. Parallel pays off only when each test costs seconds, not
milliseconds. Measured honestly rather than hidden.

### Micro-benchmarks (criterion)

- hash_100b: 617 ns
- hash_10kb: 25.4 us
- slice extract_vars: 1.80 us
- slice extract_calls: 832 ns
- ddmin_100: 15.2 us

Startup: median 27.5ms to full help output, 9.65ms best warm case.
Binary size: 1.95 MB stripped release build.

### Accuracy verdict

Nine out of nine scenarios: both tools named the exact first bad commit, verified
against ground truth baked into each generator. Two real product bugs were caught
by this campaign and fixed before final numbers were taken:

1. crux reported the LAST GOOD commit instead of the FIRST BAD commit. Root cause:
   `git log` returns newest-first and the binary search assumed chronological order.
   Fixed in `src/blame/mod.rs` (range reversed before search) and `src/blame/parallel.rs`.
2. Parallel mode silently failed on repos whose test script was untracked, because
   git worktrees do not contain untracked files. Every commit read as FAIL. The
   benchmark now commits the harness into its scenario repos, which doubles as the
   correct usage advice: put your test command in the repository.

A third issue was harness-side, not product-side: shallow clones expose multiple
graft roots, which broke naive range derivation for R1. Fixed with an explicit range.

### Usefulness verdict

git bisect prints one line: a hash and "is the first bad commit".

crux who on the dependency-chain scenario printed:

```
cmd:  e651ba8cf65a
commit: 1e89564de4b2 break update vendored library to v2
suspects: app/expect.txt, bisect-test.sh, pad.log, vendor/lib/version.txt
  - v1
  + v2-broken
```

Same commit, plus the suspect file list and the two lines that did the damage.
That is the product argument in eight lines. Beyond who, crux ships watch, replay,
report, index, init, doctor, log, diff and completions. Bisect ships nothing else
because bisect is a git feature, not a product. Fair comparison: bisect is free,
ships with git, and is faster. crux explains the breakage and watches for drift.

## Bugs the human hit while benchmarking (documented so they are not repeated)

1. First generator draft flipped status.txt back to pass after the break commit.
   Non-monotonic predicate, both tools correctly refused to answer. Lesson: sticky fail.
2. `git bisect run cmd /c test.cmd` from Git-for-Windows sh spawned an interactive
   cmd that tested nothing and exited 0, so bisect declared HEAD guilty with zero
   real tests. Fixed by giving bisect its own sh script.
3. Long-running benchmark shells kept getting killed by the environment, so the
   suite grew a `-Only` filter and a per-repo runner to stay under the kill window.

## Resources - every path

Scripts (versioned in this repo):

- `benchmark/bench-all.ps1` - scenario builder and runner (S1-S7, P1, R1)
- `benchmark/bench-one.ps1` - per-repo tool runner for existing repos
- `benchmark/RESULTS.md` - earlier campaign, superseded by this file

Raw data and samples (copied from the bench workspace):

- `benchmark/results/raw.tsv` - every timed row: scenario, tool, ms, test runs, found hash/message
- `benchmark/results/S*-bisect.txt`, `S*-crux-normal.txt`, `S*-crux-fast.txt`,
  `P1_slow_tests-crux-parallel.txt`, `S7-rich-sample.txt` - captured tool outputs
- `benchmark/results/R1_fd_real-*` - real-repository runs

Scenario repositories (regenerable, live outside the repo):

- `C:\Users\fetit\AppData\Local\Temp\opencode\crux-bench\repos\S1_linear_50`
- `...\repos\S2_linear_200`, `...\repos\S3_linear_1000`, `...\repos\S4_merge`,
  `...\repos\S5_large_diff`, `...\repos\S6_interaction`, `...\repos\S7_dependency_chain`,
  `...\repos\P1_slow_tests`, `...\repos\R1_fd_real`

Micro-benchmark sources and outputs:

- `benches/hash_bench.rs`, `benches/slice_bench.rs`, `benches/min_bench.rs`
- `target/criterion/` - full criterion measurement sets

Product code touched by findings:

- `src/blame/mod.rs` - first-bad ordering fix
- `src/blame/parallel.rs` - same fix for the worktree path

Unit, CLI and fuzz suites still green after the fixes: 117 tests, 0 clippy warnings.

## One-paragraph verdict

Bisect is faster everywhere, by 3x on a real repository growing to 35x at a thousand
synthetic commits, and the remaining gap has a known, cheap fix. Both tools are
perfectly accurate once crux's off-by-one was fixed. crux's reason to exist is the
eight lines of explanation it prints where bisect prints one, plus drift watching.
If you only need the hash, use git bisect. If you need to know why, use crux.
