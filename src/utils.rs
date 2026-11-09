use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Url;
use std::error::Error;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// Download a file to the specified directory and return its absolute path.
pub(crate) async fn download(
    url: Url,
    filename: &str,
    download_dir: &Path,
    stdout: Option<&mut dyn Write>,
) -> Result<PathBuf, Box<dyn Error>> {
    std::fs::create_dir_all(download_dir)?;
    let file_path = std::path::absolute(download_dir.join(filename))?;
    let mut response = reqwest::get(url).await?.error_for_status()?;
    let mut file = std::fs::File::create(&file_path)?;
    let progress = match response.content_length() {
        Some(total) => {
            let progress = ProgressBar::new(total);
            progress.set_style(ProgressStyle::with_template(
                "{wide_bar} {percent}% {bytes}/{total_bytes} {bytes_per_sec} ETA {eta}",
            )?);
            progress
        }
        None => {
            let progress = ProgressBar::new_spinner();
            progress.set_style(ProgressStyle::with_template(
                "{spinner} {bytes} {bytes_per_sec}",
            )?);
            progress.enable_steady_tick(Duration::from_millis(100));
            progress
        }
    };

    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk)?;
        progress.inc(chunk.len() as u64);
    }
    progress.finish_and_clear();

    if let Some(stdout) = stdout {
        writeln!(stdout, "Downloaded {} to {}", filename, file_path.display())?;
    }

    Ok(file_path)
}

pub(crate) fn nvim_edit_scp(host: &str, file_path: &Path) -> Result<(), Box<dyn Error>> {
    // nvim scp://router-gl-be3600//etc/openclash/custom/openclash_custom_rules.list
    let scp_path = format!("scp://{}//{}", host, file_path.display());
    let status = Command::new("nvim").arg(&scp_path).status()?;
    if !status.success() {
        return Err(format!("nvim scp edit failed: {status}").into());
    }
    Ok(())
}

pub(crate) fn scp_upload(file_path: &Path, dst: &str) -> Result<(), Box<dyn Error>> {
    let status = Command::new("scp")
        .arg("--")
        .arg(file_path)
        .arg(dst)
        .status()?;
    if !status.success() {
        return Err(format!("scp upload failed: {status}").into());
    }
    Ok(())
}

pub(crate) fn ssh_cat(host: &str, file_path: &str) -> Result<String, Box<dyn Error>> {
    let command = format!("cat {file_path}");
    ssh_exec(host, &[&command])
}

pub(crate) fn ssh_exec(host: &str, commands: &[&str]) -> Result<String, Box<dyn Error>> {
    let output = Command::new("ssh")
        .arg("--")
        .arg(host)
        .args(commands)
        .output()?;
    Ok(String::from_utf8(output.stdout)?)
}

pub(crate) fn confirm(prompt: Option<&str>, stdout: Option<&mut dyn Write>) -> bool {
    let out = if let Some(stdout) = stdout {
        stdout
    } else {
        &mut io::stdout().lock()
    };
    let prompt = prompt.unwrap_or("Are you sure you want to continue?");
    write!(out, "{prompt} (y/N) ").unwrap();
    out.flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

pub(crate) fn format_error(mut err: &dyn Error) -> String {
    let mut message = err.to_string();
    while let Some(source) = err.source() {
        message.push_str(&format!(": {source}"));
        err = source;
    }
    message
}
