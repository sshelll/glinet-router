use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Generate zsh completion script
    #[command(name = "gen-zsh-completion")]
    GenZshCompletion {
        /// Output file path for the generated zsh completion script
        #[arg(long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Check OpenClash assets from the latest CloudRunFilesBuilder release
    #[command(name = "check-update")]
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

    /// Check custom rules(override)
    #[command(name = "check-custom-rules")]
    CheckCustomRules {
        /// Directory for downloaded files [default: router-gl-be3600]
        #[arg(
            long,
            value_name = "HOST",
            default_value = "router-gl-be3600",
            value_parser = clap::builder::NonEmptyStringValueParser::new()
          )]
        host: String,
        /// Destination passed to scp
        #[arg(
            long,
            value_name = "FILE",
            default_value = "openclash_custom_rules.list",
            value_parser = clap::builder::NonEmptyStringValueParser::new()
          )]
        file: String,
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
    #[command(name = "config")]
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
pub(crate) enum OpenClashCommands {
    /// check status
    #[command(name = "status")]
    Status,

    /// restart openclash
    #[command(name = "restart")]
    Restart,

    /// stop openclash
    #[command(name = "stop")]
    Stop,

    /// start openclash
    #[command(name = "start")]
    Start,

    /// reload configs
    #[command(name = "reload")]
    Reload,
}

#[derive(Subcommand)]
pub(crate) enum ConfigCommands {
    /// list all configs(subscriptions)
    #[command(name = "list")]
    List,

    /// show current config(subscription)
    #[command(name = "current")]
    Current,

    /// switch existing config(subscription)
    #[command(name = "switch")]
    Switch,
}
