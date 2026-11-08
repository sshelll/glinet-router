use colored::Colorize;

use std::error::Error;
use std::io::{self, Write};
use std::path::Path;

use crate::github::GithubService;
use crate::{commands, utils};
use clap_complete::{Shell, generate};

pub(crate) fn gen_zsh_completion(
    mut cmd: clap::Command,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    use std::fs;
    let mut output = io::stdout().lock();
    writeln!(
        output,
        "Generating Zsh completion script at {}",
        output_path.display()
    )?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create plugin directory");
        writeln!(
            output,
            "{}",
            format!("Created plugin directory {}", parent.display()).yellow()
        )?;
    }

    let mut file = fs::File::create(output_path).expect("Failed to create Zsh completion file");
    generate(Shell::Zsh, &mut cmd, "glinet-router", &mut file);
    writeln!(
        output,
        "{}",
        format!(
            "Zsh completion script generated at {}",
            output_path.display()
        )
        .yellow(),
    )?;
    Ok(())
}

pub(crate) async fn check_update(
    download_dir: &Path,
    upload_dst: &str,
) -> Result<(), Box<dyn Error>> {
    // 1. fetch release assets
    let assets = GithubService::new()?.check_openclash_release().await?;
    let mut output = io::stdout().lock();
    if assets.is_empty() {
        writeln!(
            output,
            "{}",
            "No assets containing openclash found in the latest release.".yellow()
        )?;
    }

    // 2. display menu and select asset
    let mut menu = termenu::Menu::new()?;
    let item_list = assets
        .into_iter()
        .map(|x| {
            let name = x.name.clone();
            termenu::Item::new(name.as_str(), x)
        })
        .collect::<Vec<_>>();
    let title = format!(
        "Select an asset and download to {}:",
        download_dir.display()
    );
    let selection = menu.set_title(&title).add_list(item_list).select()?;

    // 3. download or cancel
    match selection {
        None => {
            writeln!(output, "{}", "No asset selected.".yellow())?;
            Ok(())
        }

        Some(selection) => {
            let download_url = selection.browser_download_url.clone();
            let download_file_name = selection.name.clone();
            // we need to drop the menu here to recover the terminal ui
            drop(menu);
            if !utils::confirm(Some("Download the file?"), Some(&mut output)) {
                writeln!(output, "{}", "Download cancelled.".yellow())?;
                return Ok(());
            }
            let file_path = utils::download(
                download_url,
                &download_file_name,
                download_dir,
                Some(&mut output),
            )
            .await?;
            output.flush()?;

            // scp to router
            let prompt = format!("Upload the file to {upload_dst}?");
            if !utils::confirm(Some(&prompt), Some(&mut output)) {
                writeln!(output, "{}", "Upload cancelled.".yellow())?;
                return Ok(());
            }
            utils::scp_upload(&file_path, upload_dst)?;
            writeln!(
                output,
                "{}",
                format!("Uploaded {} to {upload_dst}", file_path.display()).yellow()
            )?;
            Ok(())
        }
    }
}

pub(crate) fn check_custom_rules(host: &str, filename: &str) -> Result<(), Box<dyn Error>> {
    let file_path = format!("/etc/openclash/custom/{filename}");
    let content = utils::ssh_cat(host, &file_path)?;

    let mut output = io::stdout().lock();

    let filtered = content
        .lines()
        .filter(|ln| !ln.is_empty() && !ln.starts_with("##"))
        .collect::<Vec<_>>()
        .join("\n");

    writeln!(
        output,
        "Custom rules from `{host}:{file_path}`:\n{}",
        filtered.yellow()
    )?;
    Ok(())
}

pub(crate) fn openclash(
    host: &str,
    subcommand: commands::OpenClashCommands,
) -> Result<(), Box<dyn Error>> {
    use commands::OpenClashCommands;

    let mut stdout = io::stdout().lock();

    let cmd = match subcommand {
        OpenClashCommands::Status => "status",
        OpenClashCommands::Restart => "restart",
        OpenClashCommands::Stop => "stop",
        OpenClashCommands::Start => "start",
        OpenClashCommands::Reload => "reload",
    };

    let output = utils::ssh_exec(host, &["/etc/init.d/openclash", cmd])?;
    writeln!(stdout, "OpenClash {cmd} on {host}:\n{}", output.yellow())?;
    Ok(())
}

pub(crate) fn config(
    host: &str,
    subcommand: commands::ConfigCommands,
) -> Result<(), Box<dyn Error>> {
    use commands::ConfigCommands;

    let mut stdout = io::stdout().lock();

    let simply_exec = |ssh_cmd: &[&str], stdout: &mut dyn Write| -> Result<(), Box<dyn Error>> {
        let desc = ssh_cmd.join(" ");
        let output = utils::ssh_exec(host, ssh_cmd)?;
        writeln!(
            stdout,
            "Result of '{desc}' on {host}:\n{}",
            output.trim_start().yellow()
        )?;
        Ok(())
    };

    match subcommand {
        ConfigCommands::List => {
            let ssh_cmd = vec!["ls", "-1", "/etc/openclash/config"];
            simply_exec(&ssh_cmd, &mut stdout)
        }
        ConfigCommands::Current => {
            let ssh_cmd = vec!["uci", "show", "openclash", "|", "grep", "config_path"];
            simply_exec(&ssh_cmd, &mut stdout)
        }
        ConfigCommands::Switch => {
            let ssh_cmd = vec!["ls", "-1", "/etc/openclash/config"];
            let output = utils::ssh_exec(host, &ssh_cmd)?;

            let mut menu = termenu::Menu::new()?;
            let item_list = output
                .lines()
                .map(|x| termenu::Item::new(x, x))
                .collect::<Vec<_>>();
            let selection = menu
                .set_title("Select an config:")
                .add_list(item_list)
                .select()?;
            match selection {
                None => {
                    writeln!(stdout, "{}", "No asset selected.".yellow())?;
                    Ok(())
                }
                Some(selection) => {
                    let target = selection.to_string();
                    drop(menu);

                    let prompt = &format!("Switch to {}?", target);
                    if !utils::confirm(Some(prompt), Some(&mut stdout)) {
                        writeln!(stdout, "{}", "Switch cancelled.".yellow())?;
                        return Ok(());
                    }

                    let set_cmd =
                        &format!("openclash.config.config_path='/etc/openclash/config/{target}'");
                    let ssh_cmd = vec!["uci", "set", set_cmd, "&&", "uci", "commit", "openclash"];
                    simply_exec(&ssh_cmd, &mut stdout)?;

                    if !utils::confirm(Some("Restart openclash?"), Some(&mut stdout)) {
                        writeln!(stdout, "{}",
                            "Restart cancelled. You can manually run `glinet-router openclash restart` later.".yellow())?;
                        return Ok(());
                    }

                    openclash(host, commands::OpenClashCommands::Restart)
                }
            }
        }
    }
}
