use argp::FromArgs;

#[derive(FromArgs)]
/// finds the exact commit behind a behavior change
pub struct Cli {
    #[argp(subcommand)]
    pub command: Command,
}

#[derive(FromArgs)]
#[argp(subcommand)]
pub enum Command {
    Who(WhoArgs),
    Diff(DiffArgs),
    Init(InitArgs),
    Log(LogArgs),
    Replay(ReplayArgs),
    Doctor(DoctorArgs),
    Report(ReportArgs),
    Index(IndexArgs),
    Watch(WatchArgs),
    Predict(PredictArgs),
    Completions(CompletionsArgs),
    Guardian(GuardianArgs),
}

#[derive(FromArgs)]
/// find the commit that changed a behavior
#[argp(subcommand, name = "who")]
pub struct WhoArgs {
    /// behavior target (test command or assertion)
    #[argp(option, short = 'c')]
    pub cmd: String,
    /// revision range (e.g. HEAD~10..HEAD)
    #[argp(option, short = 'f')]
    pub from: Option<String>,
    /// parallel bisection using git worktrees
    #[argp(switch, short = 'p')]
    pub parallel: bool,
    /// follow file renames through rebases
    #[argp(switch)]
    pub follow: bool,
    /// skip merge commits
    #[argp(switch)]
    pub no_merges: bool,
    /// fast mode: binary search only, skip ranked analysis
    #[argp(switch)]
    pub fast: bool,
    /// hunt for interaction faults (commits that only break together)
    #[argp(switch)]
    pub interactions: bool,
    /// attempt network attribution to upstream dependency history
    #[argp(switch)]
    pub upstream_deep: bool,
}

#[derive(FromArgs)]
/// compare two revisions and list changed behaviors
#[argp(subcommand, name = "diff")]
pub struct DiffArgs {
    /// revision range (e.g. v1.0.0..v1.1.0)
    #[argp(positional)]
    pub range: String,
    /// output format: terminal or json
    #[argp(option, short = 'o')]
    pub output: Option<String>,
}

#[derive(FromArgs)]
/// scan repo and suggest behavior targets
#[argp(subcommand, name = "init")]
pub struct InitArgs {}

#[derive(FromArgs)]
/// list commits in a range
#[argp(subcommand, name = "log")]
pub struct LogArgs {
    /// revision range (e.g. HEAD~10..HEAD)
    #[argp(positional)]
    pub range: String,
}

#[derive(FromArgs)]
/// rerun a finding in a pinned world
#[argp(subcommand, name = "replay")]
pub struct ReplayArgs {
    /// fingerprint from a previous run
    #[argp(positional)]
    pub fingerprint: String,
}

#[derive(FromArgs)]
/// health check
#[argp(subcommand, name = "doctor")]
pub struct DoctorArgs {}

#[derive(FromArgs)]
/// render a causal-chain report
#[argp(subcommand, name = "report")]
pub struct ReportArgs {
    /// run id, commit hash, or PR number
    #[argp(positional)]
    pub target: String,
}

#[derive(FromArgs)]
/// manage the local behavior-signature DB
#[argp(subcommand, name = "index")]
pub struct IndexArgs {
    #[argp(subcommand)]
    pub command: IndexCommand,
}

#[derive(FromArgs)]
#[argp(subcommand)]
pub enum IndexCommand {
    List(IndexListArgs),
    Export(IndexExportArgs),
}

#[derive(FromArgs)]
/// list stored signatures
#[argp(subcommand, name = "list")]
pub struct IndexListArgs {}

#[derive(FromArgs)]
/// export signatures to JSON
#[argp(subcommand, name = "export")]
pub struct IndexExportArgs {}

#[derive(FromArgs)]
/// watch a behavior and report drift
#[argp(subcommand, name = "watch")]
pub struct WatchArgs {
    /// behavior target (omit to watch all declared guardians)
    #[argp(option, short = 'c')]
    pub cmd: Option<String>,
}

#[derive(FromArgs)]
/// predict if a PR will break a behavior
#[argp(subcommand, name = "predict")]
pub struct PredictArgs {
    /// behavior target (test command or assertion)
    #[argp(option, short = 'c')]
    pub cmd: String,
    /// revision range to predict for (e.g. main..HEAD)
    #[argp(option, short = 'f')]
    pub from: Option<String>,
}

#[derive(FromArgs)]
/// generate shell completions
#[argp(subcommand, name = "completions")]
pub struct CompletionsArgs {
    /// shell type: bash, zsh, or fish
    #[argp(positional)]
    pub shell: String,
}

// this code was written by an ai - begin guardian command surface
#[derive(FromArgs)]
/// declare and manage behavior guardians
#[argp(subcommand, name = "guardian")]
pub struct GuardianArgs {
    #[argp(subcommand)]
    pub command: GuardianCommand,
}

#[derive(FromArgs)]
#[argp(subcommand)]
pub enum GuardianCommand {
    Add(GuardianAddArgs),
    List(GuardianListArgs),
    Rm(GuardianRmArgs),
}

#[derive(FromArgs)]
/// declare a protected behavior
#[argp(subcommand, name = "add")]
pub struct GuardianAddArgs {
    /// guardian name
    #[argp(positional)]
    pub name: String,
    /// behavior target command
    #[argp(option, short = 'c')]
    pub cmd: String,
    /// expected state: pass or fail
    #[argp(option)]
    pub expect: Option<String>,
}

#[derive(FromArgs)]
/// list declared guardians
#[argp(subcommand, name = "list")]
pub struct GuardianListArgs {}

#[derive(FromArgs)]
/// remove a guardian
#[argp(subcommand, name = "rm")]
pub struct GuardianRmArgs {
    /// guardian name
    #[argp(positional)]
    pub name: String,
}
