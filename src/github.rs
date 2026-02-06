use sqlx::SqlitePool;
use std::time::Duration;
use serde::Deserialize;

// --- Struktur Data Respon GitHub ---
#[derive(Debug, Deserialize)]
struct GithubCommitResponse {
    sha: String,
    commit: CommitDetail,
    author: Option<AuthorDetail>, 
}

#[derive(Debug, Deserialize)]
struct CommitDetail {
    message: String,
    author: CommitAuthor,
}

#[derive(Debug, Deserialize)]
struct CommitAuthor {
    name: String, // Nama Author di Git Config
    date: String,
}

#[derive(Debug, Deserialize)]
struct AuthorDetail {
    login: String, // Username GitHub
}

// --- Struktur User dari Database ---
#[derive(sqlx::FromRow)]
struct UserConfig {
    #[allow(dead_code)] // Supaya compiler gak rewel kalau ID gak dipake
    id: i64,
    github_token: String,
    github_repo: String,
}

// --- Logic Utama Polling ---
pub async fn start_polling(pool: SqlitePool) {
    println!("👀 GitHub Watcher (Polling Mode) Dimulai...");

    loop {
        // 1. Cari user yang punya token & repo valid
        let users: Vec<UserConfig> = sqlx::query_as(
            "SELECT id, github_token, github_repo FROM users WHERE github_token IS NOT NULL AND github_repo IS NOT NULL"
        )
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        // 2. Cek update untuk setiap user
        for user in users {
            if let Err(e) = check_repo_updates(&pool, &user).await {
                // Ignore error 'Conflict' (Repo kosong) biar log gak penuh sampah
                if !e.to_string().contains("409") {
                    eprintln!("❌ Error checking repo {}: {}", user.github_repo, e);
                }
            }
        }

        // 3. Tidur 60 detik (Biar gak kena rate limit GitHub)
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

async fn check_repo_updates(pool: &SqlitePool, user: &UserConfig) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}/commits?per_page=5", user.github_repo);

    let response = client.get(&url)
        .header("User-Agent", "Noty-App")
        .header("Authorization", format!("Bearer {}", user.github_token))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("GitHub API Error: {}", response.status()).into());
    }

    let commits: Vec<GithubCommitResponse> = response.json().await?;

    for commit in commits {
        // Cek duplikasi di database
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM github_logs WHERE commit_hash = ?)")
            .bind(&commit.sha)
            .fetch_one(pool)
            .await?;

        if !exists {
            // Prioritas nama: Username GitHub -> Nama di Git Config
            let author_name = commit.author.map(|a| a.login).unwrap_or(commit.commit.author.name);
            
            sqlx::query(
                "INSERT INTO github_logs (repo_name, commit_hash, message, author, timestamp) VALUES (?, ?, ?, ?, ?)"
            )
            .bind(&user.github_repo)
            .bind(&commit.sha)
            .bind(&commit.commit.message)
            .bind(&author_name)
            .bind(&commit.commit.author.date)
            .execute(pool)
            .await?;

            println!("✨ Commit Baru Masuk: [{}] {}", user.github_repo, commit.commit.message);
        }
    }

    Ok(())
}