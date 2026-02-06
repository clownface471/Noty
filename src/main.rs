use axum::{routing::get, Router};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::env; // Tambah ini buat baca .env

mod github;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load file .env
    dotenvy::dotenv().ok(); // Pastikan library dotenvy jalan

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    println!("⏳ Menghubungkan ke database...");

    let db_url = env::var("DATABASE_URL").unwrap_or("sqlite://noty.db?mode=rwc".to_string());
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_url)
        .await
        .expect("❌ Gagal connect ke SQLite.");

    // --- MIGRATION AREA ---
    // Create Users Table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            ai_provider TEXT DEFAULT 'gemini',
            api_key TEXT,
            github_token TEXT,
            github_repo TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );"
    ).execute(&pool).await?;

    // Create Logs Table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS github_logs (
            id INTEGER PRIMARY KEY,
            repo_name TEXT NOT NULL,
            commit_hash TEXT NOT NULL UNIQUE,
            message TEXT NOT NULL,
            author TEXT NOT NULL,
            timestamp DATETIME NOT NULL,
            is_processed BOOLEAN DEFAULT FALSE
        );"
    ).execute(&pool).await?;
    
    println!("✅ Database Siap.");

    // --- SECURE SEED (Sekarang aman!) ---
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(&pool).await?;
    if count == 0 {
        // Baca dari .env (Bukan hardcode lagi)
        let token = env::var("MY_GITHUB_TOKEN").expect("⚠️ GITHUB_TOKEN belum diisi di .env");
        let repo = env::var("MY_GITHUB_REPO").expect("⚠️ GITHUB_REPO belum diisi di .env");

        sqlx::query(
            "INSERT INTO users (username, github_token, github_repo) VALUES (?, ?, ?)"
        )
        .bind("Mori")
        .bind(token) // Pakai variabel
        .bind(repo)  // Pakai variabel
        .execute(&pool)
        .await?;
        println!("💉 User data injected securely from .env!");
    }
    // ------------------------------------

    // Start GitHub Poller
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        github::start_polling(pool_clone).await;
    });

    let app = Router::new()
        .route("/", get(|| async { "Noty Backend is Running! 🚀" }))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 Server berjalan di http://localhost:3000");
    axum::serve(listener, app).await?;

    Ok(())
}