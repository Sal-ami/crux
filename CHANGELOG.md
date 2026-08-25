# Changelog

## 0.1.0

First public release.

- `crux who`: binary-searches any command's output across a commit range and
  names the exact commit that flipped the behavior
- `--fast` mode: search only, no post-analysis
- `--parallel`: worktree-based parallel probing
- minimized causal diff (delta-debugging over hunks)
- interaction-fault detection: catches two-commit flips that plain bisect
  misattributes
- upstream evidence: links flips to dependency changes (Cargo.lock, vendored
  manifests), optional blobless-clone deep mode
- guardians: watch named commands, auto-blame drift, markdown reports
- pinned replay: hermetic re-run of the flip commit, byte-compared twice
- environment signatures: code/env hashes classify flip causes
- ranked fallback for non-monotone history
- rewrite forensics via patch-id matching against the reflog
- JSONL time series in `.crux/history.jsonl`
- `crux doctor`, `crux diff`, `crux replay`, `crux report`, `crux predict`,
  `crux index`, `crux guardian`, `crux completions`
- single 2.04 MB static-ish binary, zero runtime deps beyond git
