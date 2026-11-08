use octocrab::{Octocrab, models::repos::Asset};
use std::error::Error;
use std::time::Duration;

const GITHUB_OWNER: &str = "wkccd";
const GITHUB_REPO: &str = "CloudRunFilesBuilder";

pub struct GithubService {
    client: Octocrab,
}

impl GithubService {
    pub fn new() -> octocrab::Result<Self> {
        let mut builder = Octocrab::builder();
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            builder = builder.personal_token(token);
        }
        Ok(Self {
            client: builder.build()?,
        })
    }

    pub async fn check_openclash_release(&self) -> Result<Vec<Asset>, Box<dyn Error>> {
        let latest_release = tokio::time::timeout(
            Duration::from_secs(30),
            self.client
                .repos(GITHUB_OWNER, GITHUB_REPO)
                .releases()
                .get_latest(),
        )
        .await??;

        Ok(latest_release
            .assets
            .into_iter()
            .filter(|asset| asset.name.to_lowercase().contains("openclash"))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::thread;

    fn mock_github(status: &str, body: String) -> (GithubService, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut reader = BufReader::new(&stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert_eq!(
                line,
                "GET /repos/wkccd/CloudRunFilesBuilder/releases/latest HTTP/1.1\r\n"
            );
            loop {
                line.clear();
                assert!(reader.read_line(&mut line).unwrap() > 0);
                if line == "\r\n" {
                    break;
                }
            }
            stream.write_all(response.as_bytes()).unwrap();
        });
        let client = Octocrab::builder()
            .base_uri(format!("http://{address}"))
            .unwrap()
            .build()
            .unwrap();
        (GithubService { client }, server)
    }

    fn release_body(names: &[&str]) -> String {
        let assets: Vec<_> = names
            .iter()
            .enumerate()
            .map(|(id, name)| {
                json!({
                    "id": id, "node_id": "asset", "name": name,
                    "url": "https://api.github.com/assets/1",
                    "browser_download_url": format!("https://example.com/{name}"),
                    "state": "uploaded", "content_type": "application/octet-stream",
                    "size": 1, "download_count": 0,
                    "created_at": "2026-11-02T00:00:00Z",
                    "updated_at": "2026-11-02T00:00:00Z"
                })
            })
            .collect();
        json!({
            "id": 1, "node_id": "release", "tag_name": "v1",
            "target_commitish": "main", "draft": false, "prerelease": false,
            "url": "https://api.github.com/releases/1",
            "html_url": "https://github.com/wkccd/CloudRunFilesBuilder/releases/tag/v1",
            "assets_url": "https://api.github.com/releases/1/assets",
            "upload_url": "https://uploads.github.com/releases/1/assets",
            "assets": assets
        })
        .to_string()
    }

    #[tokio::test]
    async fn filters_assets_case_insensitively_and_keeps_order() {
        let (service, server) = mock_github(
            "200 OK",
            release_body(&["openclash-arm64.run", "other.run", "25-OpenClash-x86.run"]),
        );
        let assets = service.check_openclash_release().await.unwrap();
        server.join().unwrap();
        let names: Vec<_> = assets.iter().map(|asset| asset.name.as_str()).collect();
        assert_eq!(names, ["openclash-arm64.run", "25-OpenClash-x86.run"]);
        assert_eq!(
            assets[0].browser_download_url.as_str(),
            "https://example.com/openclash-arm64.run"
        );
    }

    #[tokio::test]
    async fn returns_no_assets_for_empty_or_nonmatching_releases() {
        for names in [vec![], vec!["other.run"]] {
            let (service, server) = mock_github("200 OK", release_body(&names));
            let assets = service.check_openclash_release().await.unwrap();
            server.join().unwrap();
            assert!(assets.is_empty());
        }
    }

    #[tokio::test]
    async fn propagates_github_and_decode_errors() {
        for (status, body, expected_message) in [
            ("404 Not Found", r#"{"message":"Not Found"}"#, "Not Found"),
            (
                "403 Forbidden",
                r#"{"message":"API rate limit exceeded"}"#,
                "API rate limit exceeded",
            ),
            ("200 OK", "{", "EOF"),
        ] {
            let (service, server) = mock_github(status, body.to_owned());
            let result = service.check_openclash_release().await;
            server.join().unwrap();
            let error = result.unwrap_err();
            let message = crate::utils::format_error(error.as_ref());
            assert!(message.contains(expected_message), "{status}: {message}");
        }
    }
}
