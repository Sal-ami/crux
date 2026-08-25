# crux docs content. imported by gen.py (static pages) & test-docs/gen-md.py (markdown).
# each entry: (slug, title, group, html)

PAGES = [
    ("", "Quick start", "getting started", """
    <p><b>crux</b> is a behavior blame tool for git repositories. You describe the behavior as a command, give it a commit range such as HEAD~20..HEAD, and it runs the command at both ends of the range before binary searching the middle, which isolates the exact flip commit in about 12 runs on a 1,000 commit range.</p>
    <p>When the flip commit is found the search is only half done. crux collects the suspects, meaning every file changed anywhere in the range, then minimizes the commit down to its causal diff by applying partial hunks onto the parent & re-running your command until only the breaking lines remain. A typical run on a small repository finishes in a few seconds.</p>
    <img src="img/banner.svg" alt="crux beats git bisect wall-clock on most benchmark scenarios while naming the same culprit commit">
    <h2 id="example">example output</h2>
    <pre><code>cmd:   e651ba8cf65a
commit: 7f31ac rework feeder pipeline
suspects: src/config.py, src/feeder.py
essential: 1 of 3 hunks (4 probes)
  -chunks = ids + tail
  +ids = chunks.split()</code></pre>
    <h2 id="reading">reading the output</h2>
    <p>The cmd line is a sha256 fingerprint of your target command and stays stable across runs, which is how the history log ties observations together. The commit line names the first commit where the behavior fails. The suspects line lists the blast radius, meaning every file changed anywhere in the range. The essential lines are the verified cause, with the probe count in parentheses showing what the minimization cost.</p>
    <p>Your working tree comes back the way you left it. Every path that mutates it restores the original HEAD afterwards, and minimization probes apply patches onto throwaway parent checkouts that clean up after themselves, so a ctrl+c mid-probe is fixed by checking out your branch again.</p>
    <p>The sidebar links the rest of the documentation, including how the search & minimization work internally, guardians, replay, and the full command reference.</p>
    """),
    ("install", "Installation", "getting started", """
    <p>macOS & linux:</p>
    <div class="pill"><span class="dollar">$</span><code>curl -fsSL https://crux.rweb.site/install.sh | bash</code><button type="button" data-copy="curl -fsSL https://crux.rweb.site/install.sh | bash">copy</button><span class="ok">copied</span></div>
    <p>The installer detects your platform, pulls the release binary from GitHub, checks its sha256 checksum, and drops it in <code>~/.local/bin</code>. If that directory isn't on your PATH, it prints the exact export line for your shell profile. Read the script before piping it to a shell. That's what it's for.</p>
    <p>windows powershell:</p>
    <div class="pill"><span class="dollar">$</span><code>powershell -ExecutionPolicy Bypass -File install.ps1</code><button type="button" data-copy="powershell -ExecutionPolicy Bypass -File install.ps1">copy</button><span class="ok">copied</span></div>
    <p>with a rust toolchain already installed:</p>
    <div class="pill"><span class="dollar">$</span><code>cargo install crux-finder</code><button type="button" data-copy="cargo install crux-finder">copy</button><span class="ok">copied</span></div>
    <p>The crate is <code>crux-finder</code> because <code>crux</code> was taken on crates.io; the binary it puts on your PATH is still <code>crux</code>.</p>
    <p>npm, pnpm, bun or yarn, with npx if you want nothing installed at all:</p>
    <div class="pill"><span class="dollar">$</span><code>npx crux-finder who -c "cargo test" -f HEAD~20..HEAD</code><button type="button" data-copy="npx crux-finder who -c &quot;cargo test&quot; -f HEAD~20..HEAD">copy</button><span class="ok">copied</span></div>
    <div class="pill"><span class="dollar">$</span><code>npm i -g crux-finder</code><button type="button" data-copy="npm i -g crux-finder">copy</button><span class="ok">copied</span></div>
    <p>The npm package ships a small launcher that fetches the platform binary from GitHub Releases on first run, so pnpm &amp; bun work even with install scripts blocked.</p>
    <p>package managers:</p>
    <div class="pill"><span class="dollar">$</span><code>brew install Emran-goat/crux/crux</code><button type="button" data-copy="brew install Emran-goat/crux/crux">copy</button><span class="ok">copied</span></div>
    <div class="pill"><span class="dollar">$</span><code>scoop install crux</code><button type="button" data-copy="scoop install crux">copy</button><span class="ok">copied</span></div>
    <div class="pill"><span class="dollar">$</span><code>winget install Emran-goat.crux</code><button type="button" data-copy="winget install Emran-goat.crux">copy</button><span class="ok">copied</span></div>
    <p>Scoop needs the bucket added first (<code>scoop bucket add crux https://github.com/Emran-goat/crux</code>); brew taps it on first install. Chocolatey (<code>choco install crux</code>) and AUR (<code>crux-bin</code>) follow once the packages clear review. A deb &amp; rpm attach to every release for distro packaging.</p>
    <p>The binary is 2.04mib stripped. Its only dependency is git on PATH, because crux drives git through the same commands you'd type yourself. There's no daemon, no config file, nothing listening. Verify from any repo:</p>
    <div class="pill"><span class="dollar">$</span><code>crux doctor</code><button type="button" data-copy="crux doctor">copy</button><span class="ok">copied</span></div>
    <p>Doctor runs 4 checks: git access, history readability, a test runner manifest on disk, and its own analysis pass. One line each. It exits 1 on failure, so it works as a CI sanity step before longer crux runs.</p>
    """),
    ("concepts", "Concepts", "getting started", """
    <p>5 ideas cover the whole tool. Each maps to something visible in the output or the .crux/ directory.</p>
    <h2 id="behavior-target">behavior target</h2>
    <p>The command you pass with <code>-c</code>. Exit 0 means the behavior holds. Anything else means it changed. Output gets captured so you can read it in replay & reports, but classification never parses text. That rule is why crux can blame any observable behavior, not just builds.</p>
    <h2 id="range">range</h2>
    <p>Ordinary git syntax such as <code>HEAD~20..HEAD</code> or <code>v1.0.0..v1.1.0</code>. The range bounds everything: search cost, suspect count, probe count. Start wide when you have no clue. Start tight when you know it broke this week.</p>
    <h2 id="flip-commit">flip commit</h2>
    <p>The oldest commit in the range where the behavior fails. Binary search finds it in ceil(log2(n)) runs: 11 probes on 1,000 commits, 7 on 100. The search assumes 1 flip that sticks.</p>
    <img src="img/bisect-search.gif" alt="animation of the binary search narrowing a 200 commit history onto the flip commit in 8 probes">
    <h2 id="essential-diff">essential diff</h2>
    <p>The smallest set of hunks from the flip commit that still reproduces the failure on the parent commit alone. Every reduction is verified by running your command. It's the difference between "this commit touched 400 lines" and "this line broke it".</p>
    <h2 id="guardian-concept">guardian</h2>
    <p>A named behavior with an expected state, declared once, stored in .crux/guardians.json, committed with the repo. Watch mode checks all of them.</p>
    <h2 id="signatures">signatures</h2>
    <p>Every observation records 2 hashes. The code signature is sha256 over the HEAD tree, so it moves exactly when committed content moves. The environment signature is sha256 over OS, architecture & tool versions. Comparing them classifies a change as code or environment before any search runs, which stops you from blaming a commit for what a toolchain update did.</p>
    """),
    ("first-investigation", "Your first investigation", "getting started", """
    <p>2 test runs tell crux whether a flip exists. 8 more find it in a 100 commit range. Start in a repository where something regressed, with a command that checks the behavior & a range that contains the break.</p>
    <div class="pill"><span class="dollar">$</span><code>crux who -c "cargo test --quiet" -f HEAD~20..HEAD</code><button type="button" data-copy="crux who -c &quot;cargo test --quiet&quot; -f HEAD~20..HEAD">copy</button><span class="ok">copied</span></div>
    <h2 id="what-happens">what happens</h2>
    <p>crux reads the commit list, checks out the newest commit, runs your command, and records the result. Same at the oldest commit. If both ends agree, there's nothing to find and it says so immediately, having spent exactly 2 runs. If they disagree, it checks out the middle, runs again, and throws away the half that can't contain the flip. Repeat until 1 commit remains.</p>
    <img src="img/bisect-search.gif" alt="the search window narrowing over a 200 commit history, red bar marking each probe, converging in 8 probes">
    <p>The flip isn't the end of it. crux collects suspects: every file changed anywhere in the range. Then it minimizes the flip commit down to the lines that matter, which is the part worth waiting for. Small repository, small diff: the whole thing finishes in a few seconds.</p>
    <h2 id="example">example output</h2>
    <pre><code>cmd:   e651ba8cf65a
commit: 7f31ac rework feeder pipeline
suspects: src/config.py, src/feeder.py
essential: 1 of 3 hunks (4 probes)
  -chunks = ids + tail
  +ids = chunks.split()</code></pre>
    <h2 id="working-tree">your working tree</h2>
    <p>Every path that mutates the working tree restores your original HEAD afterwards. Minimization probes apply patches onto throwaway parent checkouts and clean up. Interrupted a run with ctrl+c mid-probe. A git checkout of your branch fixes any residue.</p>
    """),
    ("reading-output", "Reading the output", "getting started", """
    <p>Every line in a who result means one specific thing. Read them in order and the report turns from noise into a story.</p>
    <h2 id="cmd-line">the cmd line</h2>
    <p>A sha256 fingerprint of your target command. Same command, same fingerprint, every time. That's how crux ties observations together across runs in the history log.</p>
    <h2 id="commit-line">the commit line</h2>
    <p>The answer. The first commit in the range where the behavior fails, verified the same way bisect verifies its result. Two differences: it's automatic, and it checks your behavior, not a build.</p>
    <h2 id="suspects-line">the suspects line</h2>
    <p>Every file touched by any commit in the range, not just the flip commit. It's deliberately coarse: blast radius, not cause. Short list, read it as context. Long list, skip to the essential lines.</p>
    <h2 id="essential-block">the essential block</h2>
    <p>Bisect stops at the commit. crux split the flip commit's diff into pieces at hunk boundaries & asked one question per candidate subset: applied alone onto the parent, does the behavior still break? Passing subsets get discarded. What survives is the smallest set that keeps the failure alive. In the example, 3 hunks reduced to 1: the field assembly order was the whole story. The count in parentheses is the probe cost.</p>
    <h2 id="upstream-line">the upstream line</h2>
    <p>Touch a vendored dependency in the flip commit and an extra line appears:</p>
    <pre><code>dependency: libc [vendored-crate] 0.1.0 -> 0.2.0 (vendor/libc/Cargo.toml)
  upstream: https://github.com/example/libc</code></pre>
    <p>crux read the dependency's manifest at both the parent & the flip commit and reported what moved. That's evidence, not a guess. With <code>--upstream-deep</code> it resolves the version tags on the declared repository and lists upstream commits between them. Missing tags or no network, and it prints that attribution was unavailable. It doesn't invent a culprit.</p>
    """),
    ("features", "Features", "features", """
    <p>crux implements 12 features. Each exists to answer a question git bisect leaves open. This section documents what each one does, how it works internally, and where it stops.</p>
    <img src="img/campaign-bars.png" alt="grouped bars of wall time per scenario: crux fast mode edges out git bisect on most of the 9 benchmark scenarios">
    <h2 id="the-list">the list</h2>
    <p><a href="feature-blame.html">Causal blame</a>. The core search: the exact commit where a behavior changed, found by running your command across history.</p>
    <p><a href="feature-essential-diff.html">Minimized causal diff</a>. The blamed commit reduced to the specific lines that cause the change, verified by re-execution.</p>
    <p><a href="feature-interactions.html">Interaction fault detection</a>. Pairs of commits that are harmless alone & broken together, named as pairs.</p>
    <p><a href="feature-dependency-blame.html">Dependency blame</a>. Version transitions, manifests read at both revisions, upstream attribution when the ecosystem allows it.</p>
    <p><a href="feature-guardians.html">Behavior guardians</a>. Protected behaviors declared once & shared with the team.</p>
    <p><a href="feature-signature-store.html">Signature store</a>. An append-only time-series of every observation, with code & environment signatures.</p>
    <p><a href="feature-ranked-blame.html">Ranked blame</a>. Evidence-scored candidates when history is ambiguous, so you're not stuck with 1 wrong answer.</p>
    <p><a href="feature-predict.html">Prediction mode</a>. Whether a branch would change a behavior, tested before the merge.</p>
    <p><a href="feature-rewrite-resilience.html">Rewrite resilience</a>. Blame through squashes & rebases onto the original commits.</p>
    <p><a href="feature-replay.html">Deterministic replay</a>. A finding re-run in its recorded environment, twice, byte compared.</p>
    <p><a href="feature-env-separation.html">Environment vs code</a>. Whether the behavior moved because the code moved or the world did.</p>
    <p><a href="feature-report.html">Causal chain report</a>. Commit, lines, cause class & repro command in one artifact.</p>
    """),
    ("feature-blame", "Causal blame", "features", """
    <p>The core feature. You describe a behavior as a command & a range of commits. crux runs the command at the newest & oldest commits. Different results and it binary searches for the oldest commit where the behavior fails, then reports it.</p>
    <h2 id="mechanism">how it works</h2>
    <p>crux reads the range with git log, reverses it into chronological order, and probes commits by checking them out & running your command. 2 probes establish that a flip exists. Binary search converges in ceil(log2(n)) more: 12 total on 1,000 commits. Every path that touches the working tree restores your original HEAD afterwards.</p>
    <img src="img/speedup.png" alt="speedup ratio of git bisect time over fastest crux mode per scenario, red line marking parity; crux leads on 5 of 9 scenarios including a real repository">
    <h2 id="contract">the exit code contract</h2>
    <p>The behavior target is a command. Exit 0 means the behavior holds. Anything else means it changed. That one rule is what makes crux behavior-driven instead of build-driven: a build can succeed while the behavior is broken, and crux would still find your culprit.</p>
    <h2 id="limits">limits</h2>
    <p>The search assumes 1 flip that persists. Multiple flips or gradual drift break that assumption, and crux switches to ranked candidates rather than answer wrongly. Untracked files your command depends on don't exist at probe commits. Commit the harness.</p>
    """),
    ("feature-essential-diff", "Minimized causal diff", "features", """
    <p>A blamed commit is a suspect, not a cause. The essential diff is the smallest set of hunks from that commit which still reproduces the failure on its own. It turns "this 400 line commit broke it" into "these 3 lines broke it".</p>
    <h2 id="mechanism">how it works</h2>
    <p>The commit diff is split into pieces at hunk boundaries, keeping each piece's file header so any subset applies as a standalone patch. crux checks out the parent commit, applies a candidate subset, and runs your behavior command on the result. Subsets that keep the behavior broken are interesting and get subdivided. Subsets that pass are discarded. The loop stops when no piece can be removed without the behavior passing again.</p>
    <p>This is delta debugging over diff hunks. Each probe is a real execution of your command, and the probe count is printed so you know what the answer cost.</p>
    <img src="img/divan.svg" alt="divan micro-benchmark table showing the ddmin minimizer at about 18 microseconds for a 100 item set">
    <h2 id="caps">caps & refusals</h2>
    <p>Diffs above 64 hunks skip minimization and print in full, because the probe count would eat your afternoon. If the complete diff doesn't reproduce the failure on the parent, minimization refuses to run rather than minimize a diff that never broke anything. Binary files and mode changes survive as conservative keeps: a piece that can't apply cleanly is treated as required.</p>
    """),
    ("feature-interactions", "Interaction fault detection", "features", """
    <p>Some failures need 2 authors. One commit removes a code path while nothing uses it. A later commit starts using it. Now the suite is red. Bisect blames the later commit and stays silent about the earlier one, which sends you debugging the wrong change.</p>
    <img src="img/campaign-table.svg" alt="campaign table where the interaction scenario shows git bisect answering wrong while crux flags the fault as a feature">
    <h2 id="mechanism">how it works</h2>
    <p>Run who with <code>--interactions</code>. After the normal search identifies the flip commit, crux checks it out and reverts the diff of each earlier candidate commit one at a time, re-running your behavior after every revert. Reverting one specific commit makes the behavior pass again. That commit is required for the failure: a partner. crux names both.</p>
    <p>If the failure survives every revert, crux reports that the flip alone reproduces. You're not left wondering. That negative result is worth as much as the pair.</p>
    <h2 id="limits">limits</h2>
    <p>Candidates are the 16 most recent commits before the flip, so cost is bounded at roughly 30 extra runs. Reverts that fail to apply cleanly mark their commit as forced-present. Opt-in, because it multiplies test executions.</p>
    """),
    ("feature-dependency-blame", "Dependency blame", "features", """
    <p>When the breaking change lives inside vendored code, the flip commit in your repository is only the messenger. crux identifies the dependency, its version transition, and where possible the upstream commits behind it.</p>
    <h2 id="evidence">local evidence</h2>
    <p>Files under <code>vendor/</code>, <code>third_party/</code>, <code>deps/</code>, <code>extern/</code> or <code>node_modules/</code> trigger manifest reads at both the parent & flip commits. A vendored crate reports its name & version from Cargo.toml at each revision. A Cargo.lock diff gets parsed for package version transitions, including the case where package names appear only as shared context lines between changed blocks. Output:</p>
    <pre><code>dependency: libc [vendored-crate] 0.1.0 -> 0.2.0
  upstream: https://github.com/example/libc</code></pre>
    <h2 id="deep">upstream attribution</h2>
    <p>With <code>--upstream-deep</code>, crux resolves the old & new version tags on the repository URL declared in the manifest, clones the upstream repo blobless, and lists the commits between the 2 tags. That's the honest form of cross-repo blame: the history you inherited, listed. Tags don't resolve or the network is down, and it prints that attribution was unavailable. A version bump alone is never reported as an upstream cause.</p>
    """),
    ("feature-guardians", "Behavior guardians", "features", """
    <p>A guardian is a behavior you declare once with a name & an expected state. Declarations live in .crux/guardians.json inside the repository, so committing the file shares the same protected behaviors across the team & CI.</p>
    <h2 id="usage">usage</h2>
    <div class="pill"><span class="dollar">$</span><code>crux guardian add checkout-total -c "node checkout.mjs fixture.json | diff - golden.txt" --expect pass</code><button type="button" data-copy="crux guardian add checkout-total -c &quot;node checkout.mjs fixture.json | diff - golden.txt&quot; --expect pass">copy</button><span class="ok">copied</span></div>
    <p>Default expectation is pass. <code>--expect fail</code> is equally valid for canary behaviors that should keep failing, like asserting a deprecated path stays rejected. guardian list prints declarations. guardian rm removes one.</p>
    <h2 id="why">why declare</h2>
    <p>Watch mode checks every guardian without arguments, so the same command works locally & in CI. The expectation field turns watch into a gate: a guardian whose current state differs from its expectation fails the run even without drift, catching violations the moment they appear.</p>
    """),
    ("feature-signature-store", "Signature store", "features", """
    <p>Every observation crux makes gets recorded. The store is what turns repeated questions into lookups & single observations into a time-series.</p>
    <h2 id="layout">layout</h2>
    <p>2 files under .crux/. store.json keeps the latest observed state per behavior for fast drift comparison. history.jsonl is an append-only log: 1 json object per observation with timestamp, behavior, state, code signature, environment signature & the full environment variable map used for replay.</p>
    <h2 id="why-jsonl">why jsonl and not a database</h2>
    <p>An append-only text file adds 0 bytes to the binary, stays greppable, survives corruption 1 line at a time, and diffs cleanly if you commit it. The queries crux needs (latest per behavior, full history per behavior) are single scans. A database would add hundreds of kilobytes for no capability the tool uses.</p>
    <h2 id="export">export</h2>
    <p>crux index list prints the latest signatures. crux index export writes every stored signature as json lines for backup or other tooling.</p>
    """),
    ("feature-ranked-blame", "Ranked blame", "features", """
    <p>Binary search gives 1 answer when history has 1 flip. When history has several, or drifts gradually, that answer is a lie. Ranked blame is crux refusing to lie.</p>
    <h2 id="mechanism">how it works</h2>
    <p>When the normal search finds no single transition, crux tests every commit in the range & scores each failing commit on evidence: 50 points for being a flip boundary (the commit right before the behavior passes again), up to 30 for recency in the range, 20 for touching 5 or fewer files. Top 5 print with scores.</p>
    <h2 id="cost">cost</h2>
    <p>The scan is linear: 1 test run per commit. That's the price of honesty when the monotone assumption fails, and it only triggers after the cheap search already failed to answer.</p>
    <h2 id="reading">reading it</h2>
    <p>Scores are rankings, not probabilities. 90% has more evidence than 60%, but the number isn't a confidence interval. Work top down.</p>
    """),
    ("feature-predict", "Prediction mode", "features", """
    <p>predict answers a question before the expensive answer exists: if this branch merges, does the behavior change?</p>
    <h2 id="mechanism">how it works</h2>
    <p>It runs your behavior target at both ends of a range, default main..HEAD. Matching results mean nothing will change, said in 1 line, costing 2 test runs. Different results and it prints the transition direction plus a full blame naming the most likely commit.</p>
    <pre><code>behavior change predicted: pass -> fail
most likely: 7f31ac rework feeder pipeline</code></pre>
    <h2 id="record">the record</h2>
    <p>Every prediction is appended to the history log with a predicted: prefix on its state, so the record distinguishes forecasts from observations.</p>
    <h2 id="use">where it fits</h2>
    <p>Run it in CI on pull requests against the base branch. A predicted fail before merge is the cheapest moment to learn about a regression, and the blame output attached to it is the review comment.</p>
    """),
    ("feature-rewrite-resilience", "Rewrite resilience", "features", """
    <p>Squash merges & rebases rewrite history. The commit bisect finds after a rewrite is a wrapper: a new hash containing many original changes, none of which exist as commits anymore. Blaming the wrapper tells you which PR broke it, not which change did.</p>
    <h2 id="mechanism">how it works</h2>
    <p>After a search, crux checks 2 signals. The flip commit's message matching squash, rebase, amend or wip patterns. Or the reflog containing rewrite events. Either signal fires and crux computes the stable patch-id of the flip commit, then compares it against patch-ids of up to 200 unique reflog commits. A patch-id is git's content identity for a diff: identical patch-ids before & after a rewrite prove both commits carry the same change.</p>
    <p>A match means crux prints the original hash, so you can read the real author, date & message of the change that matters.</p>
    <h2 id="limits">limits</h2>
    <p>The reflog is local & expires: git garbage-collects unreachable entries after about 90 days. Detection works best soon after the rewrite, on the machine where it happened. It's a forensic bonus on top of the search, never a requirement.</p>
    """),
    ("feature-replay", "Deterministic replay", "features", """
    <p>A finding you can't reproduce is a rumor. Replay re-executes a behavior inside the exact environment captured when it was observed, then proves the result is stable by running it twice.</p>
    <h2 id="mechanism">how it works</h2>
    <p>Every observation records the full environment variable map. Replay spawns your command as a child process with a cleared environment containing only the recorded variables. No process-global mutation, no unsafe code: the pin lives entirely in the child. The command runs twice & the outputs are compared byte for byte.</p>
    <pre><code>pinned world: code=e71d4265756eefdd env=7b2ae6a286cd9586 (64 vars)
repro deterministic: yes</code></pre>
    <h2 id="verdict">the verdict</h2>
    <p>yes means the behavior is a property of the recorded world. no means something outside the recording participates: wall clock, network, ports, randomness. The divergence byte offset is printed so you can see how deep the instability runs. A non-deterministic finding isn't discarded, but it's labeled, because a flaky reproduction can't anchor a blame.</p>
    <h2 id="limits">limits</h2>
    <p>The pin covers environment variables, the working directory & the command. It doesn't cover filesystem state outside the repository, containers, or system state. Honest ceilings for a single 2mib binary.</p>
    """),
    ("feature-env-separation", "Environment vs code", "features", """
    <p>The most common wrong blame in debugging is attributing to a commit what a toolchain update did. crux separates the 2 suspects before searching.</p>
    <h2 id="signatures">2 signatures</h2>
    <p>Every observation records both. The code signature is sha256 over the HEAD tree hash: it changes exactly when committed content changes. The environment signature is sha256 over the OS, architecture & version outputs of rustc, python, node & go when present.</p>
    <h2 id="classification">classification on drift</h2>
    <p>Watch detects a behavior change & compares signatures with the previous observation. Same code hash, different environment hash: cause class environment, and the search is skipped, because no commit can be responsible. Different code hash: cause class code, and blame proceeds. The classification line prints before any commit is named, so you never watch crux hunt a culprit that doesn't exist.</p>
    <h2 id="limits">limits</h2>
    <p>The environment signature covers tool versions & platform. It doesn't cover locale, time zone, or free memory. Targets depending on those should pin them inside the command.</p>
    """),
    ("feature-report", "Causal chain report", "features", """
    <p>The report is the artifact that outlives the investigation: behavior, commit, root cause files, dependency links & the repro command in 1 place.</p>
    <h2 id="contents">what it contains</h2>
    <p>crux report takes a behavior with a stored signature & renders the chain. The blamed commit comes from a search over the recent range. Root causes are the files that commit actually touched, not the whole suspect list. Dependency links appear when root causes hit vendored paths. The repro line prints the exact crux replay command for the finding.</p>
    <h2 id="formats">formats</h2>
    <p>Terminal output uses bold labels & reads like a witness statement. json carries the same fields for tooling. Watch writes the richest variant automatically: markdown drift reports under .crux/reports/ add the essential diff & now-versus-then output excerpts, ready to paste into a pull request.</p>
    <h2 id="principle">the principle</h2>
    <p>Every claim links to evidence crux produced: a search result, a verified minimization, a replay verdict. Nothing in the report is inferred from a version number or file name alone.</p>
    """),
    ("benchmarks", "Benchmarks", "how it works", """
    <p>Every number on this page comes from one campaign: 9 scenarios, git bisect versus crux, same predicate commands, same machine, raw data in the repository under benchmark/results/raw.tsv. Accuracy column means the tool named the actual breaking commit.</p>
    <img src="img/banner.svg" alt="summary panel: crux fast mode beats git bisect on 5 of 9 scenarios, hyperfine head to head crux 1.183s vs bisect 1.215s, and only crux flags the interaction fault">
    <h2 id="wall-time">wall time</h2>
    <p>9 scenarios: linear histories at 50, 200 & 1,000 commits, a merge, a large diff, an interaction fault, a dependency chain, a slow-test suite, and a real repository (fd, 25 synthetic commits on top of upstream history). Green is bisect, amber is crux fast, purple is crux full.</p>
    <img src="img/bars.png" alt="grouped bar chart of wall seconds per scenario for git bisect, crux fast and crux full">
    <img src="img/log-bars.png" alt="the same bars on a log scale, showing the ordering holds at every magnitude">
    <p>crux --fast wins 5 of 9 outright (S1, S3, S4, S5, R1) and loses 3 (dependency chain, slow tests, and one noisy S2 harness row that a 10-run hyperfine measurement puts back in crux's favor). Full mode pays extra probes for the explanations, which is the honest trade.</p>
    <h2 id="head-to-head">hyperfine head-to-head</h2>
    <p>10 timed runs each on the 200-commit scenario, warmup run first: crux --fast 1.183s vs git bisect 1.215s. Every individual run as a dot:</p>
    <img src="img/hyperfine-strip.png" alt="strip plot of every hyperfine run on a log scale with means marked, crux fast slightly ahead of git bisect">
    <h2 id="speedup">speedup factor</h2>
    <img src="img/speedup.png" alt="speedup ratio per scenario with a red parity line; bars above the line are crux wins, below are bisect wins">
    <p>The ratio is bisect time over fastest crux mode. S3 (1,000 commits) is the clearest win at 1.38x: more commits means more probes, and crux's probe pipeline batches what bisect spawns one process at a time.</p>
    <h2 id="probes">test executions</h2>
    <p>Every probe runs your command for real, so probe count is what multiplies into minutes on slow suites. crux probes 6-10 times across these scenarios versus bisect's 4-8; the counts are close because both are binary searching, and crux spends its extra probes verifying instead of trusting your range bounds.</p>
    <img src="img/probes.png" alt="test executions per scenario, git bisect versus crux">
    <h2 id="scaling">scaling with history size</h2>
    <img src="img/scaling-surface.png" alt="3d surface of wall seconds across 50, 200 and 1000 commit linear repos for both tools">
    <p>Both tools grow logarithmically, as they should. The surface shows the gap staying proportional at every size: whatever wins at 200 commits still wins at 1,000.</p>
    <h2 id="per-scenario">per-scenario detail</h2>
    <img src="img/small-multiples.png" alt="nine small panels, one per scenario, comparing bisect, crux fast and crux full wall times">
    <h2 id="micro">micro-benchmarks</h2>
    <p>The search dominates wall time, so the in-process primitives barely matter. They're measured anyway: ddmin minimization over a 100-item set runs in 18 microseconds, content hashing 10 KB in 26 microseconds. The bottleneck is your test command, not crux.</p>
    <img src="img/divan.svg" alt="divan micro-benchmark table for ddmin, variable and call extraction, and content hashing">
    <img src="img/criterion.svg" alt="criterion means for the same primitives, matching the divan numbers">
    <h2 id="reproduce">reproducing</h2>
    <p>The harness lives in benchmark/ (bench-all.ps1 regenerates every repo & reruns all tools), the chart generators in visuals/generate.py, and the raw numbers in benchmark/results/raw.tsv. Nothing on this page is hand-typed.</p>
    """),
    ("how-search-works", "How the search works", "how it works", """
    <p>The search assumes 1 thing: somewhere in the range the behavior flipped & stayed flipped. Old commits pass, new commits fail, exactly 1 transition. Under that assumption binary search is optimal & exact.</p>
    <p>Real history breaks the assumption 2 ways. The behavior flipped more than once, fixed & re-broken. Or it drifted gradually with no single commit responsible. crux detects both. Multiple transitions and it switches to ranked mode: every commit gets tested, each failing commit scored on evidence (flip boundary, recency, size), and you get a ranked list. A wrong single answer is worse than an honest list.</p>
    <img src="img/bisect-search.gif" alt="binary search animation: the candidate window shrinks each step until only the flip commit remains">
    <h2 id="cost">what it costs</h2>
    <p>2 runs to establish the flip, then 1 run per binary search step. Cost grows logarithmically: 10,000 commits is about 15 runs. The parallel flag trades the logarithmic search for full coverage across worktrees, worth it only when 1 test run costs seconds.</p>
    <img src="img/probes.png" alt="bar chart of test executions per scenario for git bisect versus crux across the benchmark campaign">
    <blockquote>
      <b>Your tree comes back the way you left it</b>
      <p>Every path that mutates the working tree restores the original HEAD. Minimization probes apply patches onto throwaway parent checkouts and clean up. ctrl+c mid-probe, and a git checkout of your branch fixes any residue.</p>
    </blockquote>
    """),
    ("minimization", "Minimization", "how it works", """
    <p>A blamed commit with a 400 line diff isn't an answer. The change that mattered might be 3 lines hiding between a dependency bump & a comment reflow. crux finds them with delta debugging, the same algorithm family that minimizes failing test inputs.</p>
    <h2 id="mechanism">the mechanism</h2>
    <p>The flip commit's diff is split into pieces at hunk boundaries, keeping each piece's file header so it applies standalone. crux checks out the parent, applies a candidate subset, and runs your behavior command. Still broken with only that subset applied, and the subset is interesting: the cause lives inside it. Half the pieces get removed & the question repeats against smaller subsets until removing anything more makes the behavior pass. What remains is the essential diff.</p>
    <p>Each probe is a real run of your command, which is why the probe count prints. Diffs above 64 hunks skip minimization rather than eat your afternoon. Full diff prints instead; narrow the range & retry.</p>
    <img src="img/divan.svg" alt="divan micro-benchmarks: the ddmin set minimizer handles a 100 item set in about 18 microseconds, so the overhead per probe decision is negligible">
    <h2 id="trust">why to trust it</h2>
    <p>2 properties. Probes test the actual working tree after applying the patch, never through a path that would discard it. And the final set is verified: if the full diff doesn't reproduce the failure, minimization refuses to guess.</p>
    """),
    ("interaction-faults", "Interaction faults", "how it works", """
    <p>Some breakages have no single author. Commit A rewrites a module & drops a code path, and every test still passes because nothing uses that path. Commit B enables the feature that needs it. Now the suite is red. Bisect reports B, which is technically where the failure appeared & completely misleading about where the fault lives.</p>
    <img src="img/campaign-table.svg" alt="benchmark table row for the interaction scenario: git bisect marked wrong, crux marking the fault as a detected feature">
    <p><code>--interactions</code> interrogates the flip point. crux checks out the flip commit, reverts each earlier candidate commit one at a time, and re-runs your behavior after every revert. Reverting 1 specific commit makes it pass. That commit is a required partner, and crux names the pair:</p>
    <pre><code>interaction fault (6 probes):
  5e4d56c16cee B enable advanced mode
  0714e6b0bc62 A refactor drops legacy flag support</code></pre>
    <p>If the failure survives every revert, crux prints that the flip alone reproduces. Pair theory ruled out. Candidates cap at the 16 most recent earlier commits, so cost is bounded. Opt-in, because it multiplies runs.</p>
    """),
    ("dependencies", "Dependencies", "how it works", """
    <p>A large share of "it broke for no reason" isn't your code. A vendored library moved. crux watches for exactly this.</p>
    <p>Flip commit or suspects touching <code>vendor/</code>, <code>third_party/</code>, <code>deps/</code>, <code>extern/</code> or <code>node_modules/</code>, and crux reads the dependency's manifest at the parent & at the flip commit and reports the transition:</p>
    <pre><code>dependency: libc [vendored-crate] 0.1.0 -> 0.2.0
  upstream: https://github.com/example/libc</code></pre>
    <p>Lockfiles get the same treatment. A Cargo.lock diff is parsed for package version transitions, handling the way name lines appear as shared context between changed blocks.</p>
    <p><code>--upstream-deep</code> takes the repository URL from the dependency manifest, resolves the old & new version tags on that remote, fetches a blobless clone, and lists the upstream commits between the 2 tags. Tags don't resolve or the network is down, and it says attribution was unavailable. It never guesses an upstream cause from a version number alone.</p>
    """),
    ("behavior-targets", "Behavior targets", "how it works", """
    <p>Every result crux produces is only as honest as the command you pass with <code>-c</code>. 3 rules separate a target that works from one that produces confident nonsense.</p>
    <h2 id="exit-code">the exit code is the entire contract</h2>
    <p>0 holds. Anything else drifted. crux captures stdout & stderr so you can read them in replay & reports, but classification never reads text. Output parsing is where tools start lying.</p>
    <h2 id="committed">commit everything the command touches</h2>
    <p>This is the rule people break first. Probes check out old commits, and worktrees in parallel mode contain tracked files only. A check script that exists only in your working tree works at HEAD & vanishes in every probe, which makes every historical commit look broken.</p>
    <blockquote>
      <b>Commit the harness</b>
      <p>Write check.sh or check.cmd, commit it, point -c at the committed path. 1 commit removes an entire class of false results.</p>
    </blockquote>
    <h2 id="deterministic">the command must be deterministic</h2>
    <p>Pin the seed. Skip the clock & the network. A target that flakes 1 time in 50 will eventually flip a probe mid-search and land the result on an innocent commit, presented with full confidence. Not sure about your target. Run it through replay & read the verdict.</p>
    <h2 id="examples">examples</h2>
    <div class="pill"><span class="dollar">$</span><code>crux who -c "cargo test --quiet" -f v1.2.0..HEAD</code><button type="button" data-copy="crux who -c &quot;cargo test --quiet&quot; -f v1.2.0..HEAD">copy</button><span class="ok">copied</span></div>
    <div class="pill"><span class="dollar">$</span><code>crux who -c "node render.mjs fixture.json | diff - golden.txt" -f HEAD~30..HEAD</code><button type="button" data-copy="crux who -c &quot;node render.mjs fixture.json | diff - golden.txt&quot; -f HEAD~30..HEAD">copy</button><span class="ok">copied</span></div>
    <p>The second is the purest form: program output against a golden file. Any observable behavior can be written this way.</p>
    """),
    ("guardians", "Guardians", "guarding behaviors", """
    <p>Investigation is reactive. Guardians are the protective half. A guardian is a behavior you declare once with a name & an expected state:</p>
    <div class="pill"><span class="dollar">$</span><code>crux guardian add checkout-total -c "node checkout.mjs fixture.json | diff - golden.txt" --expect pass</code><button type="button" data-copy="crux guardian add checkout-total -c &quot;node checkout.mjs fixture.json | diff - golden.txt&quot; --expect pass">copy</button><span class="ok">copied</span></div>
    <p>Stored in <code>.crux/guardians.json</code>. Commit it & the whole team shares the same protected behaviors.</p>
    <p>Declaring beats typing the command each time. Watch checks every guardian without arguments, and CI runs the same check your machine runs. Expectations are explicit: <code>--expect fail</code> is just as valid for canaries asserting a deprecated path stays rejected. guardian list prints them. guardian rm removes.</p>
    """),
    ("watch", "Watch", "guarding behaviors", """
    <div class="pill"><span class="dollar">$</span><code>crux watch</code><button type="button" data-copy="crux watch">copy</button><span class="ok">copied</span></div>
    <p>Watch runs each declared guardian once, records the observation, and compares it with the previous recorded state. Passing -c checks 1 behavior without a declaration. 3 outcomes, each meaning something precise.</p>
    <h2 id="outcomes">the 3 outcomes</h2>
    <p>baseline stored: first observation, nothing to compare yet. behavior stable: state matches last run, nothing caught fire. drift detected: state changed since last time, and this is where watch earns its keep.</p>
    <h2 id="classification">cause classification</h2>
    <p>On drift, watch compares the recorded code & environment signatures with the current ones. Code hash unchanged, environment hash moved: cause class environment. No commit will ever blame correctly, so the search is skipped. Code hash changed: cause class code, and the full blame pipeline runs automatically.</p>
    <h2 id="reports">drift reports</h2>
    <p>The pipeline produces 3 things. The flip commit, found by the same search who uses. The essential diff, minimized & verified. And a markdown report at <code>.crux/reports/&lt;name&gt;-&lt;timestamp&gt;.md</code> with the transition, the commit, the essential lines & output excerpts from the broken present and the working past. It's written to be pasted into a PR without editing.</p>
    <p>Any guardian violation or any drift exits 1. Put crux watch in the pipeline & a behavior change fails the build at the PR that introduced it, evidence attached, instead of surfacing 3 days later in production.</p>
    """),
    ("replay", "Replay", "guarding behaviors", """
    <div class="pill"><span class="dollar">$</span><code>crux replay "&lt;command&gt;"</code><button type="button" data-copy="crux replay">copy</button><span class="ok">copied</span></div>
    <p>A finding you can't reproduce is a rumor. Replay turns findings back into facts.</p>
    <p>Watch records the full environment alongside every result. Replay loads that recording & spawns your command inside it: the child gets exactly the recorded variables, not the rest of your shell. Then it runs the command a second time in the same pinned world & compares outputs byte for byte.</p>
    <pre><code>pinned world: code=e71d4265756eefdd env=7b2ae6a286cd9586 (64 vars)
repro deterministic: yes
replay output:
...</code></pre>
    <p>yes: the behavior is a property of the recorded world. no: something outside the recording participates, wall clock or network or a port, and the divergence byte offset prints. The finding isn't worthless, but it's weaker, and the report says so. Containers are out of scope for now; the pin covers environment variables, working directory & the exact command.</p>
    """),
    ("changelog", "Changelog", "reference", """
    <h2 id="010">0.1.0</h2>
    <p>First public release.</p>
    <p><b>who</b>: binary-searches any command's output across a commit range and names the exact commit that flipped the behavior. <code>--fast</code> runs the search only. <code>--parallel</code> probes worktrees concurrently.</p>
    <p><b>explanations</b>: minimized causal diff via delta debugging over hunks; interaction-fault detection that names both commits when two are required; dependency blame reading vendored manifests and Cargo.lock transitions, with upstream attribution on request; ranked candidates when history is not monotone; rewrite resilience through squashes via patch-id matching.</p>
    <p><b>guarding</b>: behavior guardians with watch mode, drift auto-blame, and markdown reports; deterministic pinned replay, byte-compared twice; code and environment signatures classifying every observation.</p>
    <p><b>plumbing</b>: JSONL time series in <code>.crux/history.jsonl</code>, causal chain reports, <code>doctor</code>, <code>diff</code>, <code>predict</code>, <code>index export</code>, shell completions.</p>
    <p>Single 2.04 MiB stripped binary. Only dependency: git on PATH.</p>
    """),
    ("reference", "Command reference", "reference", """
    <h2 id="who">who</h2>
    <pre><code>crux who -c &lt;command&gt; [-f range] [--fast] [-p] [--follow] [--no-merges] [--interactions] [--upstream-deep]</code></pre>
    <table>
      <tr><th>flag</th><th>meaning</th></tr>
      <tr><td>-c, --cmd</td><td>behavior target command. required.</td></tr>
      <tr><td>-f, --from</td><td>revision range. default HEAD~10..HEAD.</td></tr>
      <tr><td>--fast</td><td>search only. no suspects, minimization, interactions or dependency work.</td></tr>
      <tr><td>-p, --parallel</td><td>probe commits across git worktrees. worth it when 1 test run costs seconds.</td></tr>
      <tr><td>--follow</td><td>follow renames when collecting suspects.</td></tr>
      <tr><td>--no-merges</td><td>exclude merge commits from the range.</td></tr>
      <tr><td>--interactions</td><td>hunt for partner commits required by the flip commit.</td></tr>
      <tr><td>--upstream-deep</td><td>attribute dependency changes to upstream history. network required.</td></tr>
    </table>
    <h2 id="diff">diff</h2>
    <p><code>crux diff &lt;range&gt; [-o json]</code>. Commits in the range with changed files. json for tooling.</p>
    <h2 id="log">log</h2>
    <p><code>crux log &lt;range&gt;</code>. Commits newest first with changed files.</p>
    <h2 id="predict">predict</h2>
    <p><code>crux predict -c &lt;command&gt; [-f range]</code>. Tests both ends without searching. Default main..HEAD. Answers whether merging changes the behavior, before the merge.</p>
    <h2 id="replay-cmd">replay</h2>
    <p><code>crux replay "&lt;command&gt;"</code>. Pinned world rerun with a determinism verdict.</p>
    <h2 id="report">report</h2>
    <p><code>crux report "&lt;behavior&gt;"</code>. Causal chain for a stored signature. -o json available.</p>
    <h2 id="index">index</h2>
    <p><code>crux index list</code> & <code>crux index export</code>. Show or export recorded signatures.</p>
    <h2 id="init">init</h2>
    <p><code>crux init</code>. Detects cargo, pytest, go & npm projects, suggests targets.</p>
    <h2 id="doctor">doctor</h2>
    <p><code>crux doctor</code>. 4 checks, 1 line each, exit 1 on required failure.</p>
    <h2 id="completions">completions</h2>
    <p><code>crux completions bash|zsh|fish</code>. Prints a completion script.</p>
    <h2 id="guardian">guardian</h2>
    <p><code>crux guardian add &lt;name&gt; -c &lt;command&gt; [--expect pass|fail]</code>, <code>guardian list</code>, <code>guardian rm &lt;name&gt;</code>.</p>
    <h2 id="watch-cmd">watch</h2>
    <p><code>crux watch [-c &lt;command&gt;]</code>. With -c checks 1 behavior. Without, checks every guardian.</p>
    """),
    ("state", "State files", "reference", """
    <p>All state lives in .crux/ inside the repository being investigated. 4 files, each with 1 job.</p>
    <table>
      <tr><th>file</th><th>purpose</th></tr>
      <tr><td>store.json</td><td>latest observed state per behavior, used for drift comparison</td></tr>
      <tr><td>history.jsonl</td><td>append-only log of every observation with code & environment signatures</td></tr>
      <tr><td>guardians.json</td><td>declared guardians & expectations</td></tr>
      <tr><td>reports/</td><td>markdown drift reports written by watch</td></tr>
    </table>
    <p>history.jsonl is append-only by design. Past observations are the time-series that makes drift detection meaningful, so nothing prunes them. Delete .crux for a clean slate. Commit it if you want drift detection across machines & CI runs.</p>
    """),
    ("platforms", "Platforms", "reference", """
    <p>crux runs on macOS, linux & windows with identical behavior. On windows, commands execute through cmd /C, so write targets as .cmd scripts or single cmd commands. git for windows supplies everything else.</p>
    <p>1 interop note. git bisect on windows executes its run command through the bundled sh, which can't launch a .cmd script. Using both tools on the same repo means giving bisect an .sh script & crux a .cmd script.</p>
    """),
    ("troubleshooting", "Troubleshooting", "reference", """
    <p>no behavior change detected in range. The target passed at every commit or failed at every one. Run it by hand at HEAD & at the range start. If it genuinely flips somewhere, the history isn't monotone, and crux printed ranked candidates instead.</p>
    <p>all commits fail in range. The behavior was already broken before the range began. Move the start further back.</p>
    <p>Parallel mode reports every commit as failing. The target script isn't committed. Worktrees contain tracked files only, and your untracked helper doesn't exist at any probe path.</p>
    <p>repro deterministic: no. Something outside the recorded environment drives the command: wall clock, network, ports. The finding stands but is weaker than a deterministic one.</p>
    <p>Watch keeps printing baseline stored. The -c string changed, even by whitespace, so crux treats it as a different behavior. Remove the stale entry with guardian rm or edit .crux/store.json.</p>
    <p>The search landed on an innocent commit. The target is flaky or reads untracked files. Fix the target before trusting anything crux says about it. The tool is only as honest as the predicate it's given.</p>
    """),
]
