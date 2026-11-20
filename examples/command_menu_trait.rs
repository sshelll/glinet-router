//! Run with `cargo run --example command_menu_trait`; handlers only produce previews.

use std::{io, path::PathBuf};

trait MenuSource: Sized + Send + Sync {
    fn items() -> Vec<termenu::Item<Self>>;

    fn build_menu() -> io::Result<termenu::Menu<Self>> {
        let mut menu = termenu::Menu::new()?;
        menu.set_title("Select a command (preview only):")
            .add_list(Self::items());
        Ok(menu)
    }

    fn select() -> io::Result<Option<Self>>
    where
        Self: Clone,
    {
        let mut menu = Self::build_menu()?;
        // Own the selection so dropping the menu restores the terminal before dispatch.
        menu.select().map(|selection| selection.cloned())
    }
}

#[derive(Clone, Debug)]
enum DemoCommands {
    Status,
    CheckUpdate {
        download_dir: Option<PathBuf>,
        upload_target: String,
    },
}

impl MenuSource for DemoCommands {
    fn items() -> Vec<termenu::Item<Self>> {
        vec![
            termenu::Item::new("Show status", Self::Status),
            termenu::Item::new(
                "Check update",
                Self::CheckUpdate {
                    download_dir: None,
                    upload_target: "router-gl-be3600:upload/".into(),
                },
            ),
        ]
    }
}

impl DemoCommands {
    fn preview(self) -> String {
        match self {
            Self::Status => Self::preview_status(),
            Self::CheckUpdate {
                download_dir,
                upload_target,
            } => Self::preview_update(download_dir, upload_target),
        }
    }

    fn preview_status() -> String {
        "Preview: query OpenClash status".into()
    }

    fn preview_update(download_dir: Option<PathBuf>, upload_target: String) -> String {
        let directory = download_dir
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "~/Downloads".into());
        format!("Preview: download to {directory}, then upload to {upload_target}")
    }
}

fn main() -> io::Result<()> {
    if let Some(command) = DemoCommands::select()? {
        println!("{}", command.preview());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_items_dispatch_with_their_declared_values() {
        let actual: Vec<_> = DemoCommands::items()
            .into_iter()
            .map(|item| (item.alias, item.value.preview()))
            .collect();
        assert_eq!(
            actual,
            vec![
                (
                    "Show status".into(),
                    "Preview: query OpenClash status".into()
                ),
                (
                    "Check update".into(),
                    "Preview: download to ~/Downloads, then upload to router-gl-be3600:upload/"
                        .into()
                ),
            ]
        );
    }

    #[test]
    fn dispatch_uses_variant_fields_instead_of_menu_defaults() {
        let command = DemoCommands::CheckUpdate {
            download_dir: Some(PathBuf::from("/tmp/demo downloads")),
            upload_target: "test-router:/tmp/upload/".into(),
        };
        assert_eq!(
            command.preview(),
            "Preview: download to /tmp/demo downloads, then upload to test-router:/tmp/upload/"
        );
    }
}
