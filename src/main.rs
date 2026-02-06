use axum::{
    routing::{get, post},
    Router
};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::env;
use tower_http::cors::CorsLayer;

// Import modul yang sudah kita buat
mod github;
mod chat;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load .env (Wajib di awal)
    dotenvy::dotenv().ok();

    // 2. Setup Log Terminal
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    println!("⏳ Noty System Starting...");

    // 3. Setup Database SQLite
    let db_url = env::var("DATABASE_URL").unwrap_or("sqlite://noty.db?mode=rwc".to_string());
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_url)
        .await
        .expect("❌ Gagal connect ke noty.db");

    // --- AUTO MIGRATION (Bikin Tabel Otomatis) ---
    // Tabel Users
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

    // Tabel GitHub Logs
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
    // ---------------------------------------------

    // 4. Inject User Data dari .env (Hanya jika tabel user kosong)
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(&pool).await?;
    if count == 0 {
        let token = env::var("MY_GITHUB_TOKEN").expect("⚠️ MY_GITHUB_TOKEN belum diisi di .env");
        let repo = env::var("MY_GITHUB_REPO").expect("⚠️ MY_GITHUB_REPO belum diisi di .env");

        sqlx::query(
            "INSERT INTO users (username, github_token, github_repo) VALUES (?, ?, ?)"
        )
        .bind("Admin")
        .bind(token)
        .bind(repo)
        .execute(&pool)
        .await?;
        println!("💉 User Credentials diinject aman dari .env");
    }

    // 5. Jalankan GitHub Poller di Background
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        github::start_polling(pool_clone).await;
    });

    // 6. Setup Router API
    let app = Router::new()
        .route("/", get(|| async { "Noty Backend is Running (Gemini 2.0 Enabled)! 🚀" }))
        .route("/api/chat", post(chat::handle_chat)) // Endpoint Chat
        .layer(CorsLayer::permissive())
        .with_state(pool);

    // 7. Start Server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 Server Noty siap di http://localhost:3000");
    axum::serve(listener, app).await?;

    Ok(())
}