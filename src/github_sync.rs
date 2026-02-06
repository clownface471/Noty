use sqlx::SqlitePool;
use std::time::Duration;
use serde::Deserialize;
use reqwest::header;

#[derive(Deserialize, Debug)]
struct GitHubCommit {
    sha: String,
    commit: CommitDetails,
}

#[derive(Deserialize, Debug)]
struct CommitDetails {
    message: String,
    author: AuthorDetails,
}

#[derive(Deserialize, Debug)]
struct AuthorDetails {
    date: String,
}

pub async fn start_github_polling(pool: SqlitePool) {
    let client = reqwest::Client::new();
    println!("👀 GitHub Watcher Service Started...");

    loop {
        // 1. Ambil config dari DB
        let integration = sqlx::query!(
            "SELECT config_json, api_key FROM integrations WHERE service_name = 'github' AND is_active = TRUE LIMIT 1"
        )
        .fetch_optional(&pool)
        .await
        .unwrap_or(None);

        if let Some(config) = integration {
            let repo_name = config.config_json.unwrap_or_default();
            let token = config.api_key.unwrap_or_default();

            if !repo_name.is_empty() {
                // 2. Tembak GitHub API
                let url = format!("https://api.github.com/repos/{}/commits?per_page=5", repo_name);
                
                let mut headers = header::HeaderMap::new();
                headers.insert("User-Agent", header::HeaderValue::from_static("Noty-Logbook"));
                if !token.is_empty() {
                    let auth_str = format!("Bearer {}", token);
                    if let Ok(val) = header::HeaderValue::from_str(&auth_str) {
                        headers.insert("Authorization", val);
                    }
                }

                match client.get(&url).headers(headers).send().await {
                    Ok(res) => {
                        if let Ok(commits) = res.json::<Vec<GitHubCommit>>().await {
                            for commit in commits {
                                // 3. Cek Duplikat di Logbook (FIXED VERSION)
                                // Kita pakai query_scalar biar return-nya langsung integer (i32/i64)
                                // Gak perlu struct aneh-aneh.
                                let sha_pattern = format!("%{}%", commit.sha);
                                
                                let count: i32 = sqlx::query_scalar(
                                    "SELECT count(*) FROM log_entries WHERE source = 'GitHub' AND content LIKE ?"
                                )
                                .bind(sha_pattern)
                                .fetch_one(&pool)
                                .await
                                .unwrap_or(0);

                                if count == 0 {
                                    println!("✨ Import Commit: {}", commit.commit.message);
                                    
                                    let content = format!("**GitHub Update:** {}\n\n*Commit ID: {}*", commit.commit.message, commit.sha);
                                    
                                    // Auto-Insert ke Logbook
                                    let _ = sqlx::query!(
                                        "INSERT INTO log_entries (content, entry_date, entry_time, tags, category, source, mood) 
                                         VALUES (?, date('now'), time('now'), ?, 'Development', 'GitHub', 'Neutral')",
                                        content,
                                        "[\"coding\", \"github\"]"
                                    )
                                    .execute(&pool)
                                    .await;
                                }
                            }
                        }
                    },
                    Err(e) => eprintln!("⚠️ Gagal fetch GitHub: {}", e),
                }
            }
        }

        // Tunggu 60 detik (Interval Polling)
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}