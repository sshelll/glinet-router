use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Generate zsh completion script
    GenZshCompletion {
        /// Output file path for the generated zsh completion script
        #[arg(long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Check OpenClash assets from the latest CloudRunFilesBuilder release
    CheckUpdate {
        /// Directory for downloaded files [default: ~/Downloads]
        #[arg(long, value_name = "DIR")]
        download_dir: Option<PathBuf>,
        /// Destination passed to scp
        #[arg(
            long,
            value_name = "TARGET",
            default_value = "router-gl-be3600:upload/",
            value_parser = clap::builder::NonEmptyStringValueParser::new()
          )]
        upload_target: String,
    },

    /// Manage custom rules(override)
    CustomRules {
        /// SSH host or alias
        #[arg(
            long,
            value_name = "HOST",
            default_value = "router-gl-be3600",
            value_parser = clap::builder::NonEmptyStringValueParser::new()
          )]
        host: String,
        /// Custom rules filename
        #[arg(
            long,
            value_name = "FILE",
            default_value = "openclash_custom_rules.list",
            value_parser = clap::builder::NonEmptyStringValueParser::new()
          )]
        file: String,

        #[command(subcommand)]
        command: CustomRulesCommands,
    },

    /// Manage openclash
    #[command(name = "openclash")]
    OpenClash {
        #[arg(
              long,
              value_name = "HOST",
              default_value = "router-gl-be3600",
              value_parser = clap::builder::NonEmptyStringValueParser::new()
          )]
        host: String,

        #[command(subcommand)]
        command: OpenClashCommands,
    },

    /// Manage config(subscriptions)
    Config {
        #[arg(
              long,
              value_name = "HOST",
              default_value = "router-gl-be3600",
              value_parser = clap::builder::NonEmptyStringValueParser::new()
          )]
        host: String,

        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
pub(crate) enum CustomRulesCommands {
    /// Check custom rules
    Check,

    /// Edit custom rules
    Edit,
}

#[derive(Subcommand)]
pub(crate) enum OpenClashCommands {
    /// check status
    Status,

    /// restart openclash
    Restart,

    /// stop openclash
    Stop,

    /// start openclash
    Start,

    /// reload configs
    Reload,
}

#[derive(Subcommand)]
pub(crate) enum ConfigCommands {
    /// list all configs(subscriptions)
    List,

    /// show current config(subscription)
    Current,

    /// switch existing config(subscription)
    Switch,
}
