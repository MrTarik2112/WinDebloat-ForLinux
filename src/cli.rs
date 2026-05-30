use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "windebloat", about = "Advanced Linux debloating & cleaning tool", version)]
pub struct Cli {
    /// Verbose output (-v for verbose, -vv for debug)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Quiet mode (only errors)
    #[arg(short, long)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Launch the interactive TUI (default)
    Tui {
        /// Skip confirmation dialogs
        #[arg(short, long)]
        yes: bool,
    },

    /// Scan for cleanable items (dry-run)
    Scan {
        /// Category to scan (packages, system, apps, privacy, services, duplicates, containers, disk, all)
        #[arg(short, long, default_value = "all")]
        category: String,

        /// Output format: json, table, simple
        #[arg(short, long, default_value = "simple")]
        format: String,

        /// Minimum size filter (e.g., 10MB)
        #[arg(short, long)]
        min_size: Option<String>,

        /// Show only safe items
        #[arg(short, long)]
        safe_only: bool,

        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },

    /// Clean selected items
    Clean {
        /// Category to clean (packages, system, apps, privacy, services, duplicates, containers, disk, all)
        #[arg(short, long, default_value = "all")]
        category: String,

        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,

        /// Dry run (show what would be done)
        #[arg(short, long)]
        dry_run: bool,

        /// Output format: json, table, simple
        #[arg(short, long, default_value = "simple")]
        format: String,
    },

    /// Batch mode - read paths from stdin and generate clean list
    Batch {
        /// Read paths from stdin (one per line)
        #[arg(short, long)]
        stdin: bool,

        /// Output clean commands (do not execute)
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Export scan results to file
    Export {
        /// Output file path
        #[arg(short, long)]
        output: String,

        /// Category to export
        #[arg(short, long, default_value = "all")]
        category: String,
    },

    /// Import scan results from file
    Import {
        /// Input file path
        #[arg(short, long)]
        input: String,
    },

    /// List and manage backups
    Undo {
        /// Specific backup ID to restore
        #[arg(short, long)]
        id: Option<String>,

        /// List all backups
        #[arg(short, long)]
        list: bool,

        /// Verify backup integrity
        #[arg(short, long)]
        verify: bool,
    },

    /// Show system status
    Status,

    /// Show action logs
    Logs {
        /// Number of recent log entries to show
        #[arg(short, long, default_value = "20")]
        count: usize,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },

    /// Full automated cleaning with 30-phase deep analysis
    AutoClean {
        /// Categories to clean (comma-separated: packages,system,apps,privacy,services,containers,disk,games,ides,messaging,mail,databases,web,dotfiles,ssh,fonts,desktop,network,orphans,duplicates)
        #[arg(short, long, default_value = "all")]
        categories: String,

        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,

        /// Dry run (show what would be done without actually cleaning)
        #[arg(short, long)]
        dry_run: bool,

        /// Safe-only mode (skip risky items)
        #[arg(short = 'S', long, default_value_t = true)]
        safe_only: bool,

        /// Output report format (text, json, markdown)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Save report to file
        #[arg(short, long)]
        output: Option<String>,

        /// Clean profile (safe, balanced, aggressive)
        #[arg(short, long, default_value = "balanced")]
        profile: String,

        /// Run disk benchmark before and after
        #[arg(short, long)]
        benchmark: bool,

        /// Show growth predictions
        #[arg(short, long)]
        predict: bool,

        /// Minimum item size to include (e.g., 10MB)
        #[arg(long)]
        min_size: Option<String>,

        /// Maximum item size to include (e.g., 1GB)
        #[arg(long)]
        max_size: Option<String>,

        /// Install cron scheduler for periodic auto-clean
        #[arg(long)]
        install_scheduler: bool,

        /// Remove installed cron scheduler
        #[arg(long)]
        remove_scheduler: bool,

        /// Verbose phase details
        #[arg(short, long)]
        verbose: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Edit configuration in default editor
    Edit,
    /// Reset configuration to defaults
    Reset,
}
