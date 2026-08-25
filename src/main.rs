use crux::cli::{Command, IndexCommand};

fn main() {
    if std::env::args()
        .skip(1)
        .any(|a| a == "--version" || a == "-V")
    {
        println!("crux {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let cli: crux::cli::Cli = argp::parse_args_or_exit(argp::DEFAULT);
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    match cli.command {
        Command::Who(args) => cmd_who(args, &cwd),
        Command::Diff(args) => {
            let entries = crux::diff::diff(&args.range, &cwd);
            let out = crux::report::render_diff(&entries, args.output.as_deref());
            print!("{}", out);
            if entries.is_empty() {
                eprintln!("no changes in range");
            }
        }
        Command::Init(_) => crux::init::init(&cwd),
        Command::Log(args) => match crux::git::log(&args.range, &cwd) {
            Ok(commits) => {
                for c in &commits {
                    println!("{} {}", &c.hash[..12], c.message);
                }
                eprintln!("{} commits", commits.len());
            }
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        },
        Command::Replay(args) => cmd_replay(args, &cwd),
        Command::Doctor(_) => crux::doctor::doctor(&cwd),
        Command::Report(args) => cmd_report(args, &cwd),
        Command::Index(args) => match args.command {
            IndexCommand::List(_) => cmd_index_list(&cwd),
            IndexCommand::Export(_) => cmd_index_export(&cwd),
        },
        Command::Watch(args) => cmd_watch(args, &cwd),
        Command::Predict(args) => cmd_predict(args, &cwd),
        Command::Completions(args) => cmd_completions(&args.shell),
        Command::Guardian(args) => cmd_guardian(args, &cwd),
    }
}

fn cmd_guardian(args: crux::cli::GuardianArgs, cwd: &std::path::Path) {
    use crux::cli::GuardianCommand;
    match args.command {
        GuardianCommand::Add(a) => {
            let expect = a.expect.unwrap_or_else(|| "pass".into());
            if !["pass", "fail"].contains(&expect.as_str()) {
                eprintln!("expect must be pass or fail");
                std::process::exit(1);
            }
            match crux::guardian::add(
                cwd,
                crux::guardian::Guardian {
                    name: a.name.clone(),
                    behavior: a.cmd.clone(),
                    expect,
                },
            ) {
                Ok(()) => eprintln!("guardian stored: {}", a.name),
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
        }
        GuardianCommand::List(_) => {
            let gs = crux::guardian::list(cwd);
            if gs.is_empty() {
                eprintln!("no guardians declared");
                return;
            }
            for g in &gs {
                println!("{} -> {} (expect {})", g.name, g.behavior, g.expect);
            }
        }
        GuardianCommand::Rm(a) => match crux::guardian::remove(cwd, &a.name) {
            Ok(true) => eprintln!("guardian removed: {}", a.name),
            Ok(false) => eprintln!("no such guardian: {}", a.name),
            Err(e) => eprintln!("error: {e}"),
        },
    }
}

fn cmd_who(args: crux::cli::WhoArgs, cwd: &std::path::Path) {    let range = args.from.as_deref().unwrap_or("HEAD~10..HEAD");
    let fp = crux::fingerprint::fingerprint(&args.cmd);
    eprintln!("cmd:  {}", &fp.hash[..12]);

    if args.parallel {
        let n_cpus = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
        eprintln!("parallel: {n_cpus} workers");
        let results = crux::blame::parallel::parallel_bisect(&args.cmd, range, cwd, n_cpus);
        match results {
            Ok(results) => {
                if let Some(flip) = crux::blame::parallel::find_flip(&results) {
                    println!(
                        "flip: {} {}",
                        &flip.commit.hash[..12.min(flip.commit.hash.len())],
                        flip.commit.message,
                    );
                } else {
                    for r in &results {
                        let status = if r.passed { "PASS" } else { "FAIL" };
                        println!(
                            "[{status}] {} {}",
                            &r.commit.hash[..12.min(r.commit.hash.len())],
                            r.commit.message,
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

    match crux::blame::blame(&args.cmd, range, args.no_merges, cwd) {
        Ok(b) => {
            println!(
                "commit: {} {}",
                &b.commit.hash[..12.min(b.commit.hash.len())],
                b.commit.message,
            );
            if args.fast {
                return;
            }
            let mut range_commits: Vec<crux::git::Commit> = Vec::new();
            let changed: Vec<String> = if args.follow {
                let files: Vec<String> = b.commit.files_changed.clone();
                let mut all = Vec::new();
                for file in &files {
                    if let Ok(followed) = crux::git::log_follow(file, range, cwd) {
                        for c in followed {
                            for f in c.files_changed {
                                if !all.contains(&f) {
                                    all.push(f);
                                }
                            }
                        }
                    }
                }
                all
            } else {                let rc = crux::git::log(range, cwd).unwrap_or_default();
                let changed: Vec<String> = rc
                    .iter()
                    .flat_map(|c| c.files_changed.iter().cloned())
                    .collect();
                range_commits = rc;
                changed
            };            let mut uniq: Vec<&String> = changed.iter().collect();
            uniq.sort();
            uniq.dedup();
            if !uniq.is_empty() {
                eprintln!("suspects: {}", uniq.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
            }
            // F2: minimized causal diff
            let parent = format!("{}^", b.commit.hash);
            let head_before = head_hash(cwd);
            let diff_text = crux::min::essential::flip_diff(&parent, &b.commit.hash, cwd);
            match crux::min::essential::minimize(&diff_text, &parent, &args.cmd, cwd, 64) {
                Some(m) => {
                    eprintln!(
                        "essential: {} of {} hunks ({} probes)",
                        m.kept.len(),
                        crux::min::hunks::parse_pieces(&diff_text).len(),
                        m.iterations
                    );
                    for p in &m.kept {
                        for line in &p.body {
                            if (line.starts_with('+') || line.starts_with('-'))
                                && !line.starts_with("+++")
                                && !line.starts_with("---")
                            {
                                eprintln!("  {line}");
                            }
                        }
                    }
                }
                None => {
                    let n = crux::min::hunks::parse_pieces(&diff_text).len();
                    if n > 0 {
                        eprintln!("essential: skipped ({n} hunks, cap 64)");
                    }
                }
            }
            if let Some(h) = &head_before {
                crux::blame::restore_head(h, cwd);
            }
            // F4: dependency-transitive evidence
            let parent_for_dep = format!("{}^", b.commit.hash);
            for ev in crux::blame::upstream::evidence(&b.commit.hash, &parent_for_dep, cwd) {
                eprintln!(
                    "dependency: {} [{}] {} -> {} ({})",
                    ev.name,
                    ev.kind,
                    if ev.old_version.is_empty() { "?" } else { &ev.old_version },
                    ev.new_version,
                    ev.changed_files.join(", ")
                );
                if let Some(u) = &ev.url {
                    eprintln!("  upstream: {u}");
                }
                if args.upstream_deep {
                    let commits = crux::blame::upstream::deep_commits(&ev, 5);
                    if commits.is_empty() {
                        eprintln!("  deep attribution: unavailable (no tags or network)");
                    } else {
                        eprintln!("  upstream history {}..{}:", ev.old_version, ev.new_version);
                        for c in commits {
                            eprintln!("    {c}");
                        }
                    }
                }
            }
            // F3: interaction-fault hunt (opt-in, expensive)
            if args.interactions
                && let Some(pos) = range_commits.iter().position(|c| c.hash == b.commit.hash)
            {
                let earlier: Vec<crux::git::Commit> = range_commits[pos + 1..].to_vec();
                match crux::blame::interaction::detect(&args.cmd, &b.commit, &earlier, cwd) {
                    Some(ix) if ix.participants.len() > 1 => {
                        eprintln!("interaction fault ({} probes):", ix.probes);
                        for p in &ix.participants {
                            eprintln!(
                                "  {} {}",
                                &p.hash[..12.min(p.hash.len())],
                                p.message
                            );
                        }
                    }
                    _ => eprintln!("no interaction fault: flip alone reproduces"),
                }
            }
            // F9: history-rewrite resilience
            if let Some(orig) = crux::git::rewrite::find_original(&b.commit.hash, cwd)
                && orig != b.commit.hash {
                    eprintln!(
                        "rewrite detected: squash/rebase of original {}",
                        &orig[..12.min(orig.len())]
                    );
                }
            if b.confidence < 1.0 {
                eprintln!("confidence: {:.0}%", b.confidence * 100.0);
            }
        }
        Err(e) => {
            // F7: ranked fallback when search is inconclusive
            if !args.fast
                && let Ok(ranked) = crux::blame::ranked(&args.cmd, range, args.no_merges, cwd)
                && !ranked.is_empty()
            {
                eprintln!("ambiguous history, ranked candidates:");
                for r in &ranked {
                    eprintln!(
                        "  {:>3.0}% {} {}",
                        r.score,
                        &r.commit.hash[..12.min(r.commit.hash.len())],
                        r.commit.message
                    );
                }
                return;
            }
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_replay(args: crux::cli::ReplayArgs, cwd: &std::path::Path) {
    let store = match crux::store::Store::open(cwd) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error opening store: {e}");
            std::process::exit(1);
        }
    };
    let sig = match store.lookup(&args.fingerprint) {
        Ok(Some(s)) => s,
        Ok(None) => {
            eprintln!("no signature found for: {}", args.fingerprint);
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };
    eprintln!("replaying: {} ({})", sig.behavior, sig.state);
    let pinned = store
        .history(&sig.behavior)
        .ok()
        .and_then(|h| h.into_iter().rev().find(|r| !r.env.is_empty()));
    let (run1, run2) = match pinned {
        Some(rec) => {
            eprintln!(
                "pinned world: code={} env={} ({} vars)",
                rec.code_hash,
                rec.env_hash,
                rec.env.len()
            );
            (
                crux::sandbox::replay_pinned(&sig.behavior, cwd, &rec.env),
                crux::sandbox::replay_pinned(&sig.behavior, cwd, &rec.env),
            )
        }
        None => (
            crux::sandbox::replay(&sig.behavior, cwd),
            crux::sandbox::replay(&sig.behavior, cwd),
        ),
    };
    if run1 == run2 {
        eprintln!("repro deterministic: yes");
    } else {
        let diverge = run1
            .bytes()
            .zip(run2.bytes())
            .position(|(a, b)| a != b)
            .unwrap_or(run1.len().min(run2.len()));
        eprintln!(
            "repro deterministic: NO (outputs diverge at byte {diverge}; {} vs {} bytes)",
            run1.len(),
            run2.len()
        );
    }
    eprintln!("replay output:\n{run1}");
}

fn cmd_report(args: crux::cli::ReportArgs, cwd: &std::path::Path) {
    let store = match crux::store::Store::open(cwd) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };
    let sig = match store.lookup(&args.target) {
        Ok(Some(s)) => s,
        Ok(None) => {
            eprintln!("no signature found for: {}", args.target);
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };
    let range = "HEAD~10..HEAD";
    let blame_result = match crux::blame::blame(&sig.behavior, range, false, cwd) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };
    let changed: Vec<String> = crux::git::log(range, cwd)
        .unwrap_or_default()
        .iter()
        .flat_map(|c| c.files_changed.iter().cloned())
        .collect();
    let mut uniq: Vec<&String> = changed.iter().collect();
    uniq.sort();
    uniq.dedup();
    let suspects: Vec<String> = uniq.into_iter().cloned().collect();
    // F12: root causes are what the blamed commit actually touched
    let roots: Vec<&String> = blame_result.commit.files_changed.iter().collect();
    let mut deps: Vec<(&String, Vec<String>)> = Vec::new();
    for s in &suspects {
        if let Some(info) = crux::blame::dep::trace_upstream(s, cwd) {
            deps.push((s, vec![info.file]));
        }
    }
    let deps_refs: Vec<(&String, &Vec<String>)> = deps.iter().map(|(k, v)| (*k, v)).collect();
    let report = crux::report::Report {
        hash: &blame_result.commit.hash,
        message: &blame_result.commit.message,
        suspects: &suspects,
        roots,
        deps: deps_refs,
        iterations: 0,
    };
    let output = crux::report::render(&report, None);
    print!("{output}");
    eprintln!("repro: crux replay \"{}\"", sig.behavior);
}

fn cmd_index_list(cwd: &std::path::Path) {
    let store = match crux::store::Store::open(cwd) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            return;
        }
    };
    match store.list_signatures() {
        Ok(sigs) => {
            if sigs.is_empty() {
                eprintln!("no signatures stored");
                return;
            }
            for sig in &sigs {
                println!("{} {} {} (t={})", &sig.hash[..12], sig.behavior, sig.state, sig.timestamp);
            }
        }
        Err(e) => eprintln!("error: {e}"),
    }
}

fn cmd_index_export(cwd: &std::path::Path) {
    let store = match crux::store::Store::open(cwd) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            return;
        }
    };
    match store.list_signatures() {
        Ok(sigs) => {
            for sig in &sigs {
                println!("{}", serde_json::to_string(sig).unwrap_or_default());
            }
        }
        Err(e) => eprintln!("error: {e}"),
    }
}

fn cmd_watch(args: crux::cli::WatchArgs, cwd: &std::path::Path) {
    let mut targets: Vec<String> = match &args.cmd {
        Some(c) => vec![c.clone()],
        None => crux::guardian::list(cwd).iter().map(|g| g.behavior.clone()).collect(),
    };
    targets.dedup();
    if targets.is_empty() {
        eprintln!("no behavior given and no guardians declared");
        std::process::exit(1);
    }
    let mut store = match crux::store::Store::open(cwd) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error opening store: {e}");
            std::process::exit(1);
        }
    };
    let mut violations = 0usize;
    let mut drifts = 0usize;
    for cmd in &targets {
        match watch_one(cmd, cwd, &mut store) {
            WatchOutcome::Violation => violations += 1,
            WatchOutcome::Drift => drifts += 1,
            _ => {}
        }
    }
    if violations > 0 || drifts > 0 {
        eprintln!("watch failed: {violations} violations, {drifts} drifts");
        std::process::exit(1);
    }
}

enum WatchOutcome {
    Baseline,
    Stable,
    Violation,
    Drift,
}

fn truncated(s: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = s.lines().take(max_lines).collect();
    let mut out = lines.join("\n");
    if s.lines().count() > max_lines {
        out.push_str("\n...");
    }
    out
}

fn watch_one(behavior: &str, cwd: &std::path::Path, store: &mut crux::store::Store) -> WatchOutcome {
    let prev_record = store
        .history(behavior)
        .ok()
        .and_then(|h| h.last().cloned());
    let prev_state = store.lookup(behavior).ok().flatten().map(|s| s.state);
    let code = crux::sig::code_hash(cwd);
    let env_h = crux::sig::env_hash();
    // exit code is the truth: Some=output of a passing run, None=failed run
    let ran = crux::sandbox::local::run_in_sandbox(behavior, cwd);
    let now_output = ran.clone().unwrap_or_default();
    let state = if ran.is_some() { "pass" } else { "fail" };
    let _ = store.store_signature(behavior, state);
    let _ = store.append(&crux::store::RunRecord {
        ts: now_secs(),
        behavior: behavior.to_string(),
        state: state.to_string(),
        code_hash: code.clone(),
        env_hash: env_h.clone(),
        env: crux::sig::capture_env(),
    });
    let mut violated = false;
    let guardian = crux::guardian::list(cwd).into_iter().find(|g| g.behavior == behavior);
    if let Some(g) = &guardian
        && g.expect != state
    {
        eprintln!(
            "guardian violation [{}]: expected {}, got {}",
            g.name, g.expect, state
        );
        violated = true;
    }
    match prev_state {
        Some(prev) if prev != state => {
            eprintln!("drift detected [{behavior}]: {prev} -> {state}");
            let cause = match &prev_record {
                Some(p) if p.code_hash == code && p.env_hash != env_h => "environment",
                Some(p) if p.code_hash != code => "code",
                _ => "code",
            };
            eprintln!("cause class: {cause}");
            if cause == "code" && let Ok(b) = crux::blame::blame(behavior, &full_range(cwd), false, cwd) {
                eprintln!(
                    "commit: {} {}",
                    &b.commit.hash[..12.min(b.commit.hash.len())],
                    b.commit.message
                );
                // F5: auto minimized causal diff + now-vs-then evidence
                let head_before = head_hash(cwd);
                let parent = format!("{}^", b.commit.hash);
                let diff_text =
                    crux::min::essential::flip_diff(&parent, &b.commit.hash, cwd);
                let mut essential_md = String::new();
                if let Some(m) =
                    crux::min::essential::minimize(&diff_text, &parent, behavior, cwd, 64)
                {
                    eprintln!(
                        "essential: {} of {} hunks",
                        m.kept.len(),
                        crux::min::hunks::parse_pieces(&diff_text).len()
                    );
                    for p in &m.kept {
                        for line in &p.body {
                            if (line.starts_with('+') || line.starts_with('-'))
                                && !line.starts_with("+++")
                                && !line.starts_with("---")
                            {
                                eprintln!("  {line}");
                                essential_md.push_str(&format!("`{line}`  \n"));
                            }
                        }
                    }
                }
                if let Some(h) = &head_before {
                    crux::blame::restore_head(h, cwd);
                }
                // now-vs-then: rerun the behavior pinned at the pre-flip world
                let then_output = std::process::Command::new("git")
                    .args(["rev-parse", "--quiet", "--verify", &parent])
                    .current_dir(cwd)
                    .output()
                    .ok()
                    .filter(|o| o.status.success())
                    .map(|_| capture_at(&parent, behavior, cwd))
                    .unwrap_or_default();
                let ts = now_secs();
                let report_dir = cwd.join(".crux").join("reports");
                let _ = std::fs::create_dir_all(&report_dir);
                let name = guardian.as_ref().map(|g| g.name.clone()).unwrap_or_else(|| behavior.chars().filter(|c| c.is_alphanumeric()).take(24).collect());
                let path = report_dir.join(format!("{name}-{ts}.md"));
                let md = format!(
                    "# crux drift report\n\n- behavior: `{behavior}`\n- transition: {prev} -> {state}\n- cause class: {cause}\n- commit: {} {}\n\n## essential causal diff\n\n{essential_md}\n## now ({state})\n\n```\n{}\n```\n\n## then ({prev}, at {})\n\n```\n{}\n```\n",
                    &b.commit.hash[..12.min(b.commit.hash.len())],
                    b.commit.message,
                    truncated(&now_output, 20),
                    &parent[..12.min(parent.len())],
                    truncated(&then_output, 20),
                );
                let _ = std::fs::write(&path, &md);
                eprintln!("report: {}", path.display());
                if violated {
                    return WatchOutcome::Violation;
                }
                return WatchOutcome::Drift;
            }
            if violated {
                return WatchOutcome::Violation;
            }
            WatchOutcome::Drift
        }
        Some(_) => {
            eprintln!("behavior stable [{behavior}]: {state}");
            if violated { WatchOutcome::Violation } else { WatchOutcome::Stable }
        }
        None => {
            eprintln!("baseline stored [{behavior}]: {state}");
            if violated { WatchOutcome::Violation } else { WatchOutcome::Baseline }
        }
    }
}

/// Run a behavior with the repo checked out at `hash`, capturing output
/// and restoring the original HEAD afterwards.
fn capture_at(hash: &str, cmd: &str, cwd: &std::path::Path) -> String {
    let head = head_hash(cwd);
    let out = crux::blame::run_at_output(hash, cmd, cwd);
    if let Some(h) = &head {
        crux::blame::restore_head(h, cwd);
    }
    out
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn cmd_predict(args: crux::cli::PredictArgs, cwd: &std::path::Path) {
    let range = args.from.as_deref().unwrap_or("main..HEAD");
    let commits = crux::git::log(range, cwd).unwrap_or_default();
    if commits.is_empty() {
        eprintln!("no commits in range");
        return;
    }
    let first = &commits.last().unwrap().hash;
    let last = &commits.first().unwrap().hash;
    let first_passes = crux::blame::run_at(first, &args.cmd, cwd);
    let last_passes = crux::blame::run_at(last, &args.cmd, cwd);
    let state = if last_passes { "pass" } else { "fail" };
    if let Ok(s) = crux::store::Store::open(cwd) {
        let _ = s.append(&crux::store::RunRecord {
            ts: now_secs(),
            behavior: args.cmd.clone(),
            state: format!("predicted:{state}"),
            code_hash: crux::sig::code_hash(cwd),
            env_hash: crux::sig::env_hash(),
            env: crux::sig::capture_env(),
        });
    }
    if first_passes == last_passes {
        eprintln!("no behavior change predicted");
    } else {
        eprintln!("behavior change predicted: {} -> {}",
            if first_passes { "pass" } else { "fail" },
            if last_passes { "pass" } else { "fail" });
        if let Ok(b) = crux::blame::blame(&args.cmd, range, false, cwd) {
            println!(
                "most likely: {} {}",
                &b.commit.hash[..12.min(b.commit.hash.len())],
                b.commit.message
            );
        }
    }
}

fn head_hash(cwd: &std::path::Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(cwd)
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

/// Range covering all reachable commits, safe for shallow/short histories.
fn full_range(cwd: &std::path::Path) -> String {
    crux::git::full_range(cwd)
}

fn cmd_completions(shell: &str) {
    match shell {
        "bash" => print!(
            r#"_crux() {{
    local cur prev words cword
    _init_completion || return
    case $prev in
        -c|-f|-o) return ;;
    esac
    case $cur in
        -*) COMPREPLY=($(compgen -W "--cmd --from --output --parallel --follow --no-merges --fast --help" -- "$cur")) ;;
        *) COMPREPLY=($(compgen -W "who diff init log replay doctor report index watch predict completions guardian" -- "$cur")) ;;
    esac
}}
complete -F _crux crux"#
        ),
        "zsh" => print!(
            r#"#compdef crux
_crux() {{
    _arguments \
        '1:command:(who diff init log replay doctor report index watch predict completions guardian)' \
        '*::arg:->args'
}}
_crux "$@""#
        ),
        "fish" => print!(
            r#"complete -c crux -f
complete -c crux -n '__fish_use_subcommand' -a who -d 'find the commit that changed a behavior'
complete -c crux -n '__fish_use_subcommand' -a diff -d 'compare two revisions'
complete -c crux -n '__fish_use_subcommand' -a init -d 'scan repo and suggest targets'
complete -c crux -n '__fish_use_subcommand' -a log -d 'list commits in a range'
complete -c crux -n '__fish_use_subcommand' -a replay -d 'rerun a finding'
complete -c crux -n '__fish_use_subcommand' -a doctor -d 'health check'
complete -c crux -n '__fish_use_subcommand' -a report -d 'render causal-chain report'
complete -c crux -n '__fish_use_subcommand' -a index -d 'manage signature DB'
complete -c crux -n '__fish_use_subcommand' -a watch -d 'watch a behavior'
complete -c crux -n '__fish_use_subcommand' -a predict -d 'predict if a PR will break'
complete -c crux -n '__fish_use_subcommand' -a completions -d 'generate shell completions'"#
        ),
        _ => {
            eprintln!("unsupported shell: {shell} (use bash, zsh, or fish)");
            std::process::exit(1);
        }
    }
}

