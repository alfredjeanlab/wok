// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::colors;
use crate::help;
use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

/// Parse a string that must not be empty or whitespace-only.
fn non_empty_string(s: &str) -> Result<String, String> {
    if s.trim().is_empty() {
        Err("cannot be empty".to_string())
    } else {
        Ok(s.to_string())
    }
}

/// Output format for commands supporting structured output.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    #[value(alias = "ids")] // Accept "ids" for backwards compatibility
    Id,
}

#[derive(Parser)]
#[command(name = "wok")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(disable_version_flag = true)]
#[command(
    about = "A collaborative, offline-first, AI-friendly issue tracker with dependency tracking"
)]
#[command(
    long_about = "A collaborative, offline-first, AI-friendly issue tracker.\n\n\
    Track issues and dependencies using git-based or realtime sync for fleet collaboration."
)]
#[command(help_template = help::template())]
#[command(before_help = help::commands())]
#[command(after_help = help::quickstart())]
#[command(styles = help::styles())]
// Allow the unit type field pattern which is required for clap's ArgAction::Version
#[allow(clippy::manual_non_exhaustive)]
pub struct Cli {
    /// Print version
    #[arg(short = 'v', short_alias = 'V', long = "version", action = clap::ArgAction::Version)]
    version: (),

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    // ─────────────────────────────────────────────────────────────────────────
    // Issues
    // ─────────────────────────────────────────────────────────────────────────
    /// Create a new issue
    #[command(after_help = colors::examples("\
Examples:
  wok new \"Fix login bug\"                Create task with title only
  wok new bug \"Fix crash\"                Create bug with explicit type
  wok new chore \"Update deps\"            Create chore for maintenance
  wok new feature \"User auth\" -l auth    Create feature with label
  wok new idea \"Better caching\"          Create idea for future consideration
  wok new task \"Multi\" -l a,b,c          Create task with multiple labels
  wok new \"Task\" -a alice                Create task assigned to alice
  wok new bug \"Fix bug\" --blocks prj-1   Create bug that blocks prj-1
  wok new \"Task\" --tracked-by prj-feat   Create task tracked by a feature
  wok new task \"My task\" -o id           Create task, output only ID"))]
    New {
        /// Issue type (feature, task, bug, chore, idea) or title if type is omitted
        #[arg(value_parser = non_empty_string)]
        type_or_title: String,

        /// Title (if type was provided as first arg)
        #[arg(value_parser = non_empty_string)]
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

        /// Issues this new issue blocks (comma-separated or repeated)
        #[arg(long, value_name = "IDS")]
        blocks: Vec<String>,

        /// Issues that block this new issue (comma-separated or repeated)
        #[arg(long, value_name = "IDS")]
        blocked_by: Vec<String>,

        /// Issues this new issue tracks/contains (comma-separated or repeated)
        #[arg(long, value_name = "IDS")]
        tracks: Vec<String>,

        /// Issues that track this new issue (comma-separated or repeated)
        #[arg(long, value_name = "IDS")]
        tracked_by: Vec<String>,

        /// Output format (text, json, id)
        #[arg(long = "output", short = 'o', value_enum, default_value = "text")]
        output: OutputFormat,
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
        after_help = colors::examples("\
Examples:
  wok done prj-1                     Complete in-progress issue
  wok done prj-1 prj-2               Complete multiple issues
  wok done prj-1 -r \"Already done\"   Skip to done from todo")
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
        after_help = colors::examples("\
Examples:
  wok edit prj-1 description \"Updated description\"    Update description
  wok edit prj-1 title \"New title\"                    Update title
  wok edit prj-1 type bug                               Change type to bug
  wok edit prj-1 assignee alice                         Assign to alice
  wok edit prj-1 assignee none                          Clear assignment")
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
    #[command(after_help = colors::examples("\
Examples:
  wok list                        List open issues (todo + in_progress)
  wok list --all                  List all issues (any status)
  wok list -s done                List completed issues
  wok list -s todo --blocked      List blocked todo issues
  wok list -t bug                 List bugs only
  wok list -l urgent              List issues with 'urgent' label
  wok list -l a -l b              List issues with label 'a' AND label 'b'
  wok list -l a,b -l c            List issues with (label 'a' OR 'b') AND label 'c'
  wok list -a alice               List issues assigned to alice
  wok list --unassigned           List unassigned issues
  wok list -q \"age < 3d\"          List issues created in last 3 days
  wok list -q \"updated > 1w\"      List issues not updated in 7+ days
  wok list --limit 10             Show only first 10 results
  wok list -o json                Output in JSON format
  wok list -o id                  Output only IDs (space-separated)

Filter Expressions (-q/--filter):
  Syntax: FIELD [OPERATOR VALUE]
  Fields: age, activity, completed, skipped, closed
  Status shortcuts: 'closed', 'skipped', 'completed' (no operator needed)
  Operators: < <= > >= = != (or: lt lte gt gte eq ne)
  Values: durations (30d, 1w, 24h, 5m, 10s), dates (2024-01-01), or 'now'
  Duration units: ms, s, m, h, d, w, M (30d), y (365d)"))]
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

        /// Output format (text, json, id)
        #[arg(long = "output", short = 'o', value_enum, default_value = "text")]
        output: OutputFormat,
    },

    /// Show ready issues (unblocked todo items)
    #[command(after_help = colors::examples("\
Examples:
  wok ready                       Show unblocked todo issues (unassigned only by default)
  wok ready -t bug                Show ready bugs
  wok ready -l urgent             Show ready urgent issues
  wok ready -a alice              Show ready issues assigned to alice
  wok ready --unassigned          Show only unassigned ready issues
  wok ready --all-assignees       Show all ready issues regardless of assignment"))]
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
        #[arg(long = "output", short = 'o', value_enum, default_value = "text")]
        output: OutputFormat,
    },

    /// Search issues by text
    #[command(
        arg_required_else_help = true,
        after_help = colors::examples("\
Examples:
  wok search \"login\"                Search for 'login' in all fields
  wok search \"auth\" -s todo         Search todo issues only
  wok search \"bug\" -t bug           Search bugs only
  wok search \"task\" -a alice        Search issues assigned to alice
  wok search \"auth\" -q \"age < 30d\"  Search with time filter
  wok search \"auth\" -n 5            Limit to 5 results

Filter Expressions (-q/--filter):
  Syntax: FIELD [OPERATOR VALUE]
  Fields: age, activity, completed, skipped, closed
  Status shortcuts: 'closed', 'skipped', 'completed' (no operator needed)
  Operators: < <= > >= = != (or: lt lte gt gte eq ne)
  Values: durations (30d, 1w, 24h, 5m, 10s), dates (2024-01-01), or 'now'
  Duration units: ms, s, m, h, d, w, M (30d), y (365d)")
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
        #[arg(long = "output", short = 'o', value_enum, default_value = "text")]
        output: OutputFormat,
    },

    /// Show full details of an issue
    #[command(arg_required_else_help = true)]
    Show {
        /// Issue ID
        id: String,

        /// Output format (text, json)
        #[arg(long = "output", short = 'o', default_value = "text")]
        output: String,
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
        after_help = colors::examples("\
Examples:
  wok link prj-a3f2 https://github.com/org/repo/issues/123
  wok link prj-a3f2 https://gitlab.com/org/project/issues/456
  wok link prj-a3f2 jira://PE-5555
  wok link prj-a3f2 https://company.atlassian.net/browse/PE-5555 --reason import
  wok link prj-a3f2 https://company.atlassian.net/wiki/spaces/DOC/pages/123")
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

    /// Remove an external link from an issue
    #[command(
        arg_required_else_help = true,
        after_help = colors::examples("\
Examples:
  wok unlink prj-a3f2 https://github.com/org/repo/issues/123
  wok unlink prj-a3f2 jira://PE-5555")
    )]
    Unlink {
        /// Issue ID
        id: String,
        /// External URL to remove (must match exactly)
        url: String,
    },

    /// Add dependency between issues
    #[command(
        arg_required_else_help = true,
        after_help = colors::examples("\
Examples:
  wok dep prj-1 blocks prj-2              prj-1 blocks prj-2
  wok dep prj-1 blocked-by prj-2 prj-3    prj-1 is blocked by prj-2 and prj-3
  wok dep prj-feat tracks prj-task        Feature tracks a task
  wok dep prj-task tracked-by prj-feat    Task is tracked by feature")
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
        after_help = colors::examples("\
Examples:
  wok label prj-1 urgent              Add label to one issue
  wok label prj-1 prj-2 prj-3 urgent  Add label to multiple issues")
    )]
    Label {
        /// Issue ID(s) followed by the label to add
        #[arg(required = true, num_args = 2..)]
        args: Vec<String>,
    },

    /// Remove a label from issue(s)
    #[command(
        arg_required_else_help = true,
        after_help = colors::examples("\
Examples:
  wok unlabel prj-1 urgent              Remove label from one issue
  wok unlabel prj-1 prj-2 prj-3 urgent  Remove label from multiple issues")
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
    #[command(after_help = colors::examples("\
Examples:
  wok init                          Initialize with auto-detected prefix
  wok init --prefix myproj          Initialize with custom prefix
  wok init --remote .               Enable git sync (same repo orphan branch)
  wok init --remote ~/tracker       Enable git sync (separate repo)
  wok init --remote ws://host:7890  Enable WebSocket sync"))]
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

        /// Remote URL for sync (git:., path, ssh URL, or ws://host:port).
        /// If not specified, initializes in local-only mode.
        #[arg(long, value_name = "URL")]
        remote: Option<String>,

        /// Initialize without remote [default behavior, kept for compatibility]
        #[arg(long, hide = true)]
        local: bool,
    },

    /// Export all issues to JSONL
    #[command(arg_required_else_help = true)]
    Export {
        /// Output file path
        filepath: String,
    },

    /// Import issues from JSONL file
    #[command(after_help = colors::examples("\
Examples:
  wok import issues.jsonl           Import from file
  wok import -                      Import from stdin
  wok import --format bd beads.jsonl  Import beads format
  wok import --dry-run issues.jsonl   Preview without applying"))]
    Import {
        /// Input file (use '-' for stdin)
        #[arg(value_name = "FILE")]
        file: Option<String>,

        /// Input file (alternative to positional)
        #[arg(long)]
        input: Option<String>,

        /// Input format: wk (default) or bd (beads)
        #[arg(long = "format", short = 'f', default_value = "wk")]
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
        after_help = colors::examples("\
Examples:
  wok completion bash > ~/.local/share/bash-completion/completions/wk
  wok completion zsh > ~/.zfunc/_wk
  wok completion fish > ~/.config/fish/completions/wk.fish")
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

    /// Output JSON Schema for commands with JSON output
    ///
    /// Use these schemas to validate JSON output or generate type definitions.
    #[command(
        subcommand,
        after_help = colors::examples("\
Examples:
  wok schema list    Output schema for 'wok list -o json'
  wok schema show    Output schema for 'wok show <id> -o json'

Available schemas: list, show, ready, search")
    )]
    Schema(SchemaCommand),
}

/// Configuration management commands.
#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Rename the issue ID prefix (updates config and all existing issues)
    #[command(
        arg_required_else_help = true,
        after_help = colors::examples("\
Examples:
  wok config rename old new    Rename prefix from 'old' to 'new'")
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
        after_help = colors::examples("\
Examples:
  wok config remote .               Use git orphan branch in current repo
  wok config remote git:.           Same as above (explicit)
  wok config remote ws://host:7890  Use WebSocket server")
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

/// Schema output commands.
#[derive(Subcommand)]
pub enum SchemaCommand {
    /// Output JSON Schema for 'wok list' JSON output
    List,
    /// Output JSON Schema for 'wok show' JSON output
    Show,
    /// Output JSON Schema for 'wok ready' JSON output
    Ready,
    /// Output JSON Schema for 'wok search' JSON output
    Search,
}

/// Hooks management commands.
#[derive(Subcommand)]
pub enum HooksCommand {
    /// Install Claude Code hooks
    #[command(after_help = colors::examples("\
Examples:
  wok hooks install              Install to local scope (default)
  wok hooks install -y           Non-interactive, local scope
  wok hooks install project      Install to project scope
  wok hooks install -i           Interactive picker"))]
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
