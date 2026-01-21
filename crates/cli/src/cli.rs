// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

/// Output format for commands supporting structured output.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Ids,
}

// Custom help template that groups commands into sections
const HELP_TEMPLATE: &str = "{about-with-newline}
{usage-heading} {usage}

{before-help}Options:
{options}{after-help}";

const COMMANDS_HELP: &str = "\
Issue Tracking:
  new         Create a new issue
  dep         Add dependency between issues
  undep       Remove dependency between issues
  show        Show issue details
  tree        Show dependency tree
  list        List issues
  ready       Show ready issues (unblocked todos)
  search      Search issues by text
  start       Start work on issue(s)
  done        Mark issue(s) as done
  close       Close issue(s) without completing
  reopen      Return issue(s) to todo
  edit        Edit an issue's properties
  note        Add a note to an issue
  label       Add a label to issue(s)
  unlabel     Remove a label from issue(s)
  link        Add external link to an issue
  log         View event log

Setup & Configuration:
  init        Initialize issue tracker
  hooks       Manage Claude Code hooks
  config      Manage configuration
  remote      Manage remote sync
  export      Export issues to JSONL
  import      Import issues from JSONL
  completion  Generate shell completions
  prime       Generate onboarding template";

const QUICKSTART_HELP: &str = "\
Get started:
  wk init                 Initialize tracker
  wk new task \"My task\"   Create a new task
  wk list                 List all issues
  wk start <id>           Start working on an issue
  wk done <id>            Mark issue as complete";

#[derive(Parser)]
#[command(name = "wk")]
#[command(
    about = "A collaborative, offline-first, AI-friendly issue tracker with dependency tracking"
)]
#[command(
    long_about = "A collaborative, offline-first, AI-friendly issue tracker.\n\n\
    Track issues and dependencies using git-based or realtime sync for fleet collaboration."
)]
#[command(help_template = HELP_TEMPLATE)]
#[command(before_help = COMMANDS_HELP)]
#[command(after_help = QUICKSTART_HELP)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    // ─────────────────────────────────────────────────────────────────────────
    // Issues
    // ─────────────────────────────────────────────────────────────────────────
    /// Create a new issue
    #[command(after_help = "Examples:\n  \
        wk new \"Fix login bug\"              Create task with title only\n  \
        wk new bug \"Fix crash\"              Create bug with explicit type\n  \
        wk new chore \"Update deps\"          Create chore for maintenance\n  \
        wk new feature \"User auth\" -l auth   Create feature with label\n  \
        wk new idea \"Better caching\"         Create idea for future consideration\n  \
        wk new task \"Multi\" -l a,b,c         Create task with multiple labels\n  \
        wk new \"Task\" -a alice               Create task assigned to alice")]
    New {
        /// Issue type (feature, task, bug, chore, idea) or title if type is omitted
        type_or_title: String,

        /// Title (if type was provided as first arg)
        title: Option<String>,

        /// Add label(s) to the issue (comma-separated or repeated)
        #[arg(long, short)]
        label: Vec<String>,

        /// Add initial note to the issue
        #[arg(long)]
        note: Option<String>,

        /// Add external link(s) to the issue
        #[arg(long)]
        link: Vec<String>,

        /// Assign the issue to someone (e.g., "alice", "queue:merge")
        #[arg(long, short)]
        assignee: Option<String>,

        /// Set priority (0-4), adds priority:N label (hidden, undocumented)
        #[arg(long, hide = true, value_parser = clap::value_parser!(u8).range(0..=4))]
        priority: Option<u8>,

        /// Add initial description note (hidden, use --note instead)
        #[arg(long, hide = true)]
        description: Option<String>,
    },

    /// Start work on issue(s) (todo -> in_progress)
    #[command(arg_required_else_help = true)]
    Start {
        /// Issue ID(s)
        #[arg(required = true)]
        ids: Vec<String>,
    },

    /// Mark issue(s) as done (in_progress -> done, or todo -> done with reason)
    #[command(
        arg_required_else_help = true,
        after_help = "Examples:\n  \
        wk done prj-1                     Complete in-progress issue\n  \
        wk done prj-1 prj-2               Complete multiple issues\n  \
        wk done prj-1 -r \"Already done\"   Skip to done from todo"
    )]
    Done {
        /// Issue ID(s)
        #[arg(required = true)]
        ids: Vec<String>,

        /// Reason (required when transitioning from todo)
        #[arg(long, short)]
        reason: Option<String>,
    },

    /// Close issue(s) without completing (requires reason for agent)
    #[command(arg_required_else_help = true)]
    Close {
        /// Issue ID(s)
        #[arg(required = true)]
        ids: Vec<String>,

        /// Reason for closing [required for agent]
        #[arg(long, short, value_name = "REASON")]
        reason: Option<String>,
    },

    /// Return issue(s) to todo (in_progress, done, or closed -> todo)
    #[command(arg_required_else_help = true)]
    Reopen {
        /// Issue ID(s)
        #[arg(required = true)]
        ids: Vec<String>,

        /// Reason for reopening [required for agent, required from done/closed]
        #[arg(long, short, value_name = "REASON")]
        reason: Option<String>,
    },

    /// Edit an issue's description, title, type, or assignee
    #[command(
        arg_required_else_help = true,
        after_help = "Examples:\n  \
            wk edit prj-1 description \"Updated description\"    Update description\n  \
            wk edit prj-1 title \"New title\"                    Update title\n  \
            wk edit prj-1 type bug                               Change type to bug\n  \
            wk edit prj-1 assignee alice                         Assign to alice\n  \
            wk edit prj-1 assignee none                          Clear assignment"
    )]
    Edit {
        /// Issue ID
        id: String,

        /// Attribute to edit (title, description, type, assignee)
        attr: String,

        /// New value for the attribute
        value: String,
    },

    /// List issues
    #[command(after_help = "Examples:\n  \
        wk list                        List open issues (todo + in_progress)\n  \
        wk list --all                  List all issues (any status)\n  \
        wk list -s done                List completed issues\n  \
        wk list -s todo --blocked      List blocked todo issues\n  \
        wk list -t bug                 List bugs only\n  \
        wk list -l urgent              List issues with 'urgent' label\n  \
        wk list -l a -l b              List issues with label 'a' AND label 'b'\n  \
        wk list -l a,b -l c            List issues with (label 'a' OR 'b') AND label 'c'\n  \
        wk list -a alice               List issues assigned to alice\n  \
        wk list --unassigned           List unassigned issues\n  \
        wk list -q \"age < 3d\"          List issues created in last 3 days\n  \
        wk list -q \"updated > 1w\"      List issues not updated in 7+ days\n  \
        wk list --limit 10             Show only first 10 results\n  \
        wk list -f json                Output in JSON format\n  \
        wk list -f ids                 Output only IDs (for piping to other commands)\n\n\
      Filter Expressions (-q/--filter):\n  \
        Syntax: FIELD OPERATOR VALUE\n  \
        Fields: age (or created), updated (or activity), closed (or completed, done)\n  \
        Operators: < <= > >= = != (or word forms: lt lte gt gte eq ne)\n  \
        Values: durations (30d, 1w, 24h, 5m, 10s) or dates (2024-01-01)\n  \
        Duration units: ms, s, m, h, d, w, M (30d), y (365d)\n  \
        Word operators are shell-friendly (no quoting needed)")]
    List {
        /// Filter by status (comma-separated for OR, repeat for AND)
        #[arg(long, short)]
        status: Vec<String>,

        /// Filter by type (comma-separated for OR, repeat for AND)
        #[arg(long, short = 't')]
        r#type: Vec<String>,

        /// Filter by label (comma-separated for OR, repeat for AND)
        #[arg(long, short)]
        label: Vec<String>,

        /// Filter by assignee (comma-separated for OR, repeat for AND)
        #[arg(long, short, value_delimiter = ',')]
        assignee: Vec<String>,

        /// Show only unassigned issues
        #[arg(long, conflicts_with = "assignee")]
        unassigned: bool,

        /// Filter expression (e.g., "age < 3d", "updated > 1w")
        #[arg(long = "filter", short = 'q')]
        filter: Vec<String>,

        /// Maximum number of results
        #[arg(long, short = 'n')]
        limit: Option<usize>,

        /// Show only blocked issues
        #[arg(long)]
        blocked: bool,

        /// Show all issues (ignore default status filter)
        #[arg(long)]
        all: bool,

        /// Output format (text, json, ids)
        #[arg(long, short, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Show ready issues (unblocked todo items)
    #[command(after_help = "Examples:\n  \
        wk ready                       Show unblocked todo issues (unassigned only by default)\n  \
        wk ready -t bug                Show ready bugs\n  \
        wk ready -l urgent             Show ready urgent issues\n  \
        wk ready -a alice              Show ready issues assigned to alice\n  \
        wk ready --unassigned          Show only unassigned ready issues\n  \
        wk ready --all-assignees       Show all ready issues regardless of assignment")]
    Ready {
        /// Filter by type (comma-separated for OR, repeat for AND)
        #[arg(long, short = 't')]
        r#type: Vec<String>,

        /// Filter by label (comma-separated for OR, repeat for AND)
        #[arg(long, short)]
        label: Vec<String>,

        /// Filter by assignee (comma-separated for OR)
        #[arg(long, short, value_delimiter = ',')]
        assignee: Vec<String>,

        /// Show only unassigned issues (overrides default behavior)
        #[arg(long, conflicts_with = "assignee", conflicts_with = "all_assignees")]
        unassigned: bool,

        /// Show all issues regardless of assignment
        #[arg(long, conflicts_with = "assignee", conflicts_with = "unassigned")]
        all_assignees: bool,

        /// Output format (text, json)
        #[arg(long, short, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Search issues by text
    #[command(
        arg_required_else_help = true,
        after_help = "Examples:\n  \
            wk search \"login\"              Search for 'login' in all fields\n  \
            wk search \"auth\" -s todo       Search todo issues only\n  \
            wk search \"bug\" -t bug         Search bugs only\n  \
            wk search \"task\" -a alice      Search issues assigned to alice\n  \
            wk search \"auth\" -q \"age < 30d\" Search with time filter\n  \
            wk search \"auth\" -n 5          Limit to 5 results\n\n\
          Filter Expressions (-q/--filter):\n  \
            Syntax: FIELD OPERATOR VALUE\n  \
            Fields: age (or created), updated (or activity), closed (or completed, done)\n  \
            Operators: < <= > >= = != (or word forms: lt lte gt gte eq ne)\n  \
            Values: durations (30d, 1w, 24h, 5m, 10s) or dates (2024-01-01)\n  \
            Duration units: ms, s, m, h, d, w, M (30d), y (365d)\n  \
            Word operators are shell-friendly (no quoting needed)"
    )]
    Search {
        /// Search query
        query: String,

        /// Filter by status (comma-separated for OR, repeat for AND)
        #[arg(long, short)]
        status: Vec<String>,

        /// Filter by type (comma-separated for OR, repeat for AND)
        #[arg(long, short = 't')]
        r#type: Vec<String>,

        /// Filter by label (comma-separated for OR, repeat for AND)
        #[arg(long, short)]
        label: Vec<String>,

        /// Filter by assignee (comma-separated for OR)
        #[arg(long, short, value_delimiter = ',')]
        assignee: Vec<String>,

        /// Show only unassigned issues
        #[arg(long, conflicts_with = "assignee")]
        unassigned: bool,

        /// Filter expression (e.g., "age < 3d", "updated > 1w")
        #[arg(long = "filter", short = 'q')]
        filter: Vec<String>,

        /// Maximum number of results (default: 25 for text output)
        #[arg(long, short = 'n')]
        limit: Option<usize>,

        /// Output format (text, json)
        #[arg(long, short, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Show full details of an issue
    #[command(arg_required_else_help = true)]
    Show {
        /// Issue ID
        id: String,

        /// Output format (text, json)
        #[arg(long, short, default_value = "text")]
        format: String,
    },

    /// Show dependency tree rooted at an issue
    #[command(arg_required_else_help = true)]
    Tree {
        /// Issue ID
        id: String,
    },

    /// Add an external link to an issue
    #[command(
        arg_required_else_help = true,
        after_help = "Examples:\n  \
            wk link prj-a3f2 https://github.com/org/repo/issues/123\n  \
            wk link prj-a3f2 https://gitlab.com/org/project/issues/456\n  \
            wk link prj-a3f2 jira://PE-5555\n  \
            wk link prj-a3f2 https://company.atlassian.net/browse/PE-5555 --reason import\n  \
            wk link prj-a3f2 https://company.atlassian.net/wiki/spaces/DOC/pages/123"
    )]
    Link {
        /// Issue ID
        id: String,
        /// External URL or shorthand (e.g., jira://PE-5555)
        url: String,
        /// Relationship reason (import, blocks, tracks, tracked-by)
        #[arg(long, short)]
        reason: Option<String>,
    },

    /// Add dependency between issues
    #[command(
        arg_required_else_help = true,
        after_help = "Examples:\n  \
        wk dep prj-1 blocks prj-2              prj-1 blocks prj-2\n  \
        wk dep prj-1 blocked-by prj-2 prj-3    prj-1 is blocked by prj-2 and prj-3\n  \
        wk dep prj-feat tracks prj-task        Feature tracks a task\n  \
        wk dep prj-task tracked-by prj-feat    Task is tracked by feature"
    )]
    Dep {
        /// Source issue ID
        from_id: String,

        /// Relationship: blocks, blocked-by, tracks (contains), tracked-by
        rel: String,

        /// Target issue ID(s)
        #[arg(required = true)]
        to_ids: Vec<String>,
    },

    /// Remove dependency between issues
    #[command(arg_required_else_help = true)]
    Undep {
        /// Source issue ID
        from_id: String,

        /// Relationship: blocks, blocked-by, tracks, tracked-by
        rel: String,

        /// Target issue ID(s)
        #[arg(required = true)]
        to_ids: Vec<String>,
    },

    /// Add a label to issue(s)
    #[command(
        arg_required_else_help = true,
        after_help = "Examples:\n  \
        wk label prj-1 urgent             Add label to one issue\n  \
        wk label prj-1 prj-2 prj-3 urgent Add label to multiple issues"
    )]
    Label {
        /// Issue ID(s) followed by the label to add
        #[arg(required = true, num_args = 2..)]
        args: Vec<String>,
    },

    /// Remove a label from issue(s)
    #[command(
        arg_required_else_help = true,
        after_help = "Examples:\n  \
        wk unlabel prj-1 urgent             Remove label from one issue\n  \
        wk unlabel prj-1 prj-2 prj-3 urgent Remove label from multiple issues"
    )]
    Unlabel {
        /// Issue ID(s) followed by the label to remove
        #[arg(required = true, num_args = 2..)]
        args: Vec<String>,
    },

    /// Add a note to an issue
    #[command(arg_required_else_help = true)]
    Note {
        /// Issue ID
        id: String,

        /// Note content
        content: String,

        /// Replace the most recent note instead of adding a new one
        #[arg(long)]
        replace: bool,
    },

    /// View event log
    Log {
        /// Issue ID (optional, shows all if omitted)
        id: Option<String>,

        /// Limit number of events
        #[arg(short = 'n', long, default_value = "20")]
        limit: usize,
    },

    // ─────────────────────────────────────────────────────────────────────────
    // Setup & Configuration
    // ─────────────────────────────────────────────────────────────────────────
    /// Initialize issue tracker in current directory (or specified path)
    #[command(after_help = "Examples:\n  \
        wk init                         Initialize with auto-detected prefix\n  \
        wk init --prefix myproj         Initialize with custom prefix\n  \
        wk init --remote .              Enable git sync (same repo orphan branch)\n  \
        wk init --remote ~/tracker      Enable git sync (separate repo)\n  \
        wk init --remote ws://host:7890 Enable WebSocket sync")]
    Init {
        /// ID prefix for issues (2+ lowercase alphanumeric, defaults to directory name)
        #[arg(long)]
        prefix: Option<String>,

        /// Path to initialize (defaults to current directory)
        #[arg(long)]
        path: Option<String>,

        /// Use shared database at path (for worktrees, monorepos, or multi-project setup)
        #[arg(long, value_name = "/path/to/shared/.work")]
        workspace: Option<String>,

        /// Remote URL for sync (git:., path, ssh URL, or ws://host:port)
        #[arg(long, value_name = "URL")]
        remote: Option<String>,

        /// Initialize without remote (no default sync)
        #[arg(long)]
        local: bool,
    },

    /// Export all issues to JSONL
    #[command(arg_required_else_help = true)]
    Export {
        /// Output file path
        filepath: String,
    },

    /// Import issues from JSONL file
    #[command(after_help = "Examples:\n  \
        wk import issues.jsonl           Import from file\n  \
        wk import -                      Import from stdin\n  \
        wk import --format bd beads.jsonl  Import beads format\n  \
        wk import --dry-run issues.jsonl   Preview without applying")]
    Import {
        /// Input file (use '-' for stdin)
        #[arg(value_name = "FILE")]
        file: Option<String>,

        /// Input file (alternative to positional)
        #[arg(long)]
        input: Option<String>,

        /// Input format: wk (default) or bd (beads)
        #[arg(long, short, default_value = "wk")]
        format: String,

        /// Preview changes without applying
        #[arg(long)]
        dry_run: bool,

        /// Filter by status (comma-separated for OR, repeat for AND)
        #[arg(long, short)]
        status: Vec<String>,

        /// Filter by type (comma-separated for OR, repeat for AND)
        #[arg(long, short = 't')]
        r#type: Vec<String>,

        /// Filter by label (comma-separated for OR, repeat for AND)
        #[arg(long, short)]
        label: Vec<String>,

        /// Filter by ID prefix
        #[arg(long)]
        prefix: Option<String>,
    },

    /// Generate shell completions
    #[command(
        arg_required_else_help = true,
        after_help = "Examples:\n  \
        wk completion bash > ~/.local/share/bash-completion/completions/wk\n  \
        wk completion zsh > ~/.zfunc/_wk\n  \
        wk completion fish > ~/.config/fish/completions/wk.fish"
    )]
    Completion {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Remote sync management
    #[command(subcommand)]
    Remote(RemoteCommand),

    /// Manage Claude Code hooks integration
    #[command(subcommand)]
    Hooks(HooksCommand),

    /// Manage configuration settings
    #[command(subcommand)]
    Config(ConfigCommand),

    /// Output issue tracker onboarding template
    Prime,
}

/// Configuration management commands.
#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Rename the issue ID prefix (updates config and all existing issues)
    #[command(
        arg_required_else_help = true,
        after_help = "Examples:\n  \
        wk config rename old new    Rename prefix from 'old' to 'new'"
    )]
    Rename {
        /// The old prefix to rename from (2+ lowercase alphanumeric with at least one letter)
        old_prefix: String,

        /// The new prefix to rename to (2+ lowercase alphanumeric with at least one letter)
        new_prefix: String,
    },
    /// Configure remote sync for the issue tracker
    #[command(
        arg_required_else_help = true,
        after_help = "Examples:\n  \
        wk config remote .              Use git orphan branch in current repo\n  \
        wk config remote git:.          Same as above (explicit)\n  \
        wk config remote ws://host:7890 Use WebSocket server"
    )]
    Remote {
        /// Remote URL: "." or "git:." for current repo, "git:<path>" for separate repo, or "ws://..." for WebSocket
        url: String,
    },
}

/// Remote sync management commands.
#[derive(Subcommand)]
pub enum RemoteCommand {
    /// Show remote sync status
    Status,
    /// Sync now with remote server
    Sync {
        /// Force full resync (request complete snapshot)
        #[arg(long)]
        force: bool,

        /// Suppress output when not in remote mode (for git hooks)
        #[arg(long)]
        quiet: bool,
    },
    /// Stop the sync daemon
    Stop,
    /// Run the daemon (internal, called by spawn)
    #[command(hide = true)]
    Run {
        /// Daemon directory (where socket/pid/lock files go)
        #[arg(long)]
        daemon_dir: std::path::PathBuf,

        /// Work directory for loading config (.work)
        #[arg(long)]
        work_dir: std::path::PathBuf,
    },
}

/// Hooks management commands.
#[derive(Subcommand)]
pub enum HooksCommand {
    /// Install Claude Code hooks
    #[command(after_help = "Examples:\n  \
        wk hooks install              Install to local scope (default)\n  \
        wk hooks install -y           Non-interactive, local scope\n  \
        wk hooks install project      Install to project scope\n  \
        wk hooks install -i           Interactive picker")]
    Install {
        /// Target scope (local, project, user)
        scope: Option<String>,

        /// Force interactive mode (TUI picker)
        #[arg(long, short = 'i', conflicts_with = "yes")]
        interactive: bool,

        /// Force non-interactive mode (auto-confirm)
        #[arg(long, short = 'y', conflicts_with = "interactive")]
        yes: bool,
    },

    /// Uninstall Claude Code hooks
    Uninstall {
        /// Target scope (local, project, user)
        scope: Option<String>,
    },

    /// Show hooks installation status
    Status,
}

#[cfg(test)]
#[path = "cli_tests/mod.rs"]
mod tests;
