mod commands;
mod github;
mod handler;
mod utils;

use clap::{CommandFactory, Parser};
use commands::Commands;
use std::error::Error;

#[derive(Parser)]
#[command(name = "glinet-router", about = "Manage GL.iNet router resources")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(err) = run(Cli::parse()).await {
        eprintln!("Error: {}", utils::format_error(err.as_ref()));
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    match cli.command {
        Commands::GenZshCompletion { output } => {
            let output_path = output.unwrap_or_else(|| {
                dirs::home_dir()
                    .expect("Failed to get home directory")
                    .join(".oh-my-zsh/custom/plugins/glinet-router/glinet-router.plugin.zsh")
            });
            handler::gen_zsh_completion(Cli::command(), &output_path)?;
        }
        Commands::CheckUpdate {
            download_dir,
            upload_target,
        } => {
            let download_dir = download_dir.unwrap_or_else(|| {
                dirs::home_dir()
                    .expect("Failed to get home directory")
                    .join("Downloads")
            });
            handler::check_update(&download_dir, &upload_target).await?;
        }

        Commands::CheckCustomRules { host, file } => {
            handler::check_custom_rules(&host, &file)?;
        }

        Commands::OpenClash { host, command } => {
            handler::openclash(&host, command)?;
        }

        Commands::Config { host, command } => {
            handler::config(&host, command)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn parses_default_paths() {
        let cli = Cli::try_parse_from(["glinet-router", "check-update"]).unwrap();
        let Commands::CheckUpdate {
            download_dir,
            upload_target,
        } = cli.command
        else {
            return;
        };
        assert_eq!(download_dir, None);
        assert_eq!(upload_target, "router-gl-be3600:upload/");
    }

    #[test]
    fn parses_custom_paths_independently() {
        use std::path::PathBuf;
        for (options, expected_dir, expected_target) in [
            (
                vec!["--download-dir", "./download files"],
                Some("./download files"),
                "router-gl-be3600:upload/",
            ),
            (
                vec!["--upload-target", "other-router:/tmp/upload/"],
                None,
                "other-router:/tmp/upload/",
            ),
            (
                vec![
                    "--download-dir",
                    "/tmp/downloads",
                    "--upload-target",
                    "other-router:/tmp/upload/",
                ],
                Some("/tmp/downloads"),
                "other-router:/tmp/upload/",
            ),
        ] {
            let cli =
                Cli::try_parse_from(["glinet-router", "check-update"].into_iter().chain(options))
                    .unwrap();
            let Commands::CheckUpdate {
                download_dir,
                upload_target,
            } = cli.command
            else {
                continue;
            };
            assert_eq!(download_dir, expected_dir.map(PathBuf::from));
            assert_eq!(upload_target, expected_target);
        }
    }

    #[test]
    fn rejects_unknown_commands_and_extra_arguments() {
        for args in [
            vec!["glinet-router", "unknown"],
            vec!["glinet-router", "check-update", "extra"],
            vec!["glinet-router", "check-update", "--download-dir"],
            vec!["glinet-router", "check-update", "--upload-target"],
            vec!["glinet-router", "check-update", "--upload-target", ""],
        ] {
            assert!(Cli::try_parse_from(args).is_err());
        }
    }

    #[test]
    fn help_lists_the_command() {
        for args in [
            vec!["glinet-router", "--help"],
            vec!["glinet-router", "check-update", "--help"],
        ] {
            let err = Cli::try_parse_from(args).err().unwrap();
            assert_eq!(err.kind(), ErrorKind::DisplayHelp);
            assert!(err.to_string().contains("check-update"));
        }
        let help = Cli::try_parse_from(["glinet-router", "check-update", "--help"])
            .err()
            .unwrap()
            .to_string();
        assert!(help.contains("--download-dir"));
        assert!(help.contains("--upload-target"));
    }
}
