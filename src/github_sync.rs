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
    println!("👀 GitHub Watcher Service Started (Debug Mode)...");

    loop {
        println!("🔄 [Sync] Mengecek konfigurasi integrasi di DB...");

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

            println!("✅ [Sync] Config ditemukan: Repo='{}', TokenSet={}", repo_name, !token.is_empty());

            if !repo_name.is_empty() {
                // 2. Tembak GitHub API
                let url = format!("https://api.github.com/repos/{}/commits?per_page=5", repo_name);
                println!("🚀 [Sync] Mengirim request ke: {}", url);
                
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
                        let status = res.status();
                        println!("📡 [Sync] Response Status: {}", status);

                        if status.is_success() {
                            match res.json::<Vec<GitHubCommit>>().await {
                                Ok(commits) => {
                                    println!("📦 [Sync] Berhasil fetch {} commits.", commits.len());
                                    for commit in commits {
                                        let sha_pattern = format!("%{}%", commit.sha);
                                        
                                        // Cek Duplikat
                                        let count: i32 = sqlx::query_scalar(
                                            "SELECT count(*) FROM log_entries WHERE source = 'GitHub' AND content LIKE ?"
                                        )
                                        .bind(&sha_pattern)
                                        .fetch_one(&pool)
                                        .await
                                        .unwrap_or(0);

                                        if count == 0 {
                                            println!("✨ [Sync] Menambahkan Commit Baru ke DB: {}", commit.commit.message);
                                            
                                            let content = format!("**GitHub Update:** {}\n\n*Commit ID: {}*", commit.commit.message, commit.sha);
                                            
                                            // Auto-Insert
                                            let insert_result = sqlx::query!(
                                                "INSERT INTO log_entries (content, entry_date, entry_time, tags, category, source, mood) 
                                                 VALUES (?, date('now'), time('now'), ?, 'Development', 'GitHub', 'Neutral')",
                                                content,
                                                "[\"coding\", \"github\"]"
                                            )
                                            .execute(&pool)
                                            .await;

                                            if let Err(e) = insert_result {
                                                println!("❌ [Sync] Gagal insert DB: {}", e);
                                            }
                                        } else {
                                            // println!("zzz [Sync] Commit {} sudah ada.", &commit.sha[0..7]);
                                        }
                                    }
                                },
                                Err(e) => println!("❌ [Sync] Gagal parsing JSON GitHub: {}", e),
                            }
                        } else {
                            // Print body kalau error (misal 404 Not Found atau 401 Unauthorized)
                            let body = res.text().await.unwrap_or_default();
                            println!("⚠️ [Sync] GitHub Error Body: {}", body);
                        }
                    },
                    Err(e) => println!("❌ [Sync] Gagal koneksi HTTP: {}", e),
                }
            }
        } else {
            println!("zzz [Sync] Tidak ada integrasi GitHub yang AKTIF di database.");
        }

        // PERCEPAT JADI 10 DETIK (Biar gak bosen nunggu)
        println!("⏳ [Sync] Tidur 10 detik...");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}