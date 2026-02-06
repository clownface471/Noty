use axum::{
    routing::{get, post},
    Router
};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::time::Duration;
use tower_http::cors::CorsLayer;
use std::env;

mod logbook; // <--- BARU
mod personas;
mod ai_features; // <--- BARU
mod github_sync;
mod integrations_api; // <--- Baru
// Kita akan buat modul baru nanti untuk handling logbook
// mod logbook; 
mod settings;
// mod ai;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load Environment Variables
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    println!("📚 Noty Logbook System Initializing...");

    // 2. Setup Database Connection
    let db_url = env::var("DATABASE_URL").unwrap_or("sqlite://noty.db?mode=rwc".to_string());
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("❌ Gagal membuat file database noty.db");

    // ==========================================
    // 3. SKEMA DATABASE BARU (The Foundation)
    // ==========================================

    // A. Tabel Settings (Pengaturan Global & Privasi)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS app_settings (
            id INTEGER PRIMARY KEY,
            username TEXT DEFAULT 'User',
            theme TEXT DEFAULT 'dark',
            
            -- Privacy & AI Settings
            use_local_ai BOOLEAN DEFAULT FALSE, -- False = Cloud, True = Local (Ollama)
            ai_provider TEXT DEFAULT 'gemini',  -- gemini, openai, ollama
            ai_api_key TEXT,
            ai_model_name TEXT DEFAULT 'gemini-1.5-flash',
            
            -- Izin Data Training (Sesuai request privasi)
            allow_data_training BOOLEAN DEFAULT FALSE, 
            
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );"
    ).execute(&pool).await?;

    // B. Tabel Log Entries (Jantung Aplikasi - Hierarkis via Timestamp)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS log_entries (
            id INTEGER PRIMARY KEY,
            content TEXT NOT NULL,
            
            -- Metadata Waktu (Untuk Filter Tahun/Bulan/Hari)
            entry_date DATE NOT NULL,      -- YYYY-MM-DD
            entry_time TIME NOT NULL,      -- HH:MM:SS
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            
            -- Metadata Organisasi
            tags TEXT,                     -- Disimpan sebagai JSON string: ['coding', 'ideas']
            category TEXT DEFAULT 'General',
            
            -- Metadata Konteks
            mood TEXT,                     -- Emosi saat mencatat (opsional)
            source TEXT DEFAULT 'Manual',  -- Manual, GitHub-Auto, Telegram-Bot
            
            -- Soft Delete (Biar aman kalau user salah hapus)
            is_deleted BOOLEAN DEFAULT FALSE
        );"
    ).execute(&pool).await?;

    // C. Tabel AI Personas (3 Mode Dasar + Custom)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_personas (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,          -- e.g., 'Scribe', 'Partner', 'Helper'
            description TEXT,
            system_prompt TEXT NOT NULL, -- Instruksi otak AI
            is_active BOOLEAN DEFAULT FALSE,
            is_custom BOOLEAN DEFAULT FALSE -- False = Bawaan sistem, True = Buatan User
        );"
    ).execute(&pool).await?;

    // D. Tabel Integrations (GitHub, Notion, dll)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS integrations (
            id INTEGER PRIMARY KEY,
            service_name TEXT NOT NULL,  -- 'github', 'notion', 'discord'
            api_key TEXT,
            config_json TEXT,            -- Simpan config tambahan (repo_name, channel_id) di sini
            
            -- Mode Operasi Integrasi
            is_active BOOLEAN DEFAULT FALSE,
            mode TEXT DEFAULT 'notify_only', -- 'notify_only', 'ai_analysis', 'full_sync'
            
            last_synced_at DATETIME
        );"
    ).execute(&pool).await?;

    println!("✅ Struktur Database Logbook Berhasil Dibangun.");

    // ==========================================
    // 4. SEEDING DATA (Isi Data Awal)
    // ==========================================

    // Seed Settings Default
    let settings_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM app_settings").fetch_one(&pool).await?;
    if settings_count == 0 {
        sqlx::query("INSERT INTO app_settings (username) VALUES ('User')").execute(&pool).await?;
    }

    // Seed 3 Persona Dasar (Sesuai Request)
    let persona_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ai_personas").fetch_one(&pool).await?;
    if persona_count == 0 {
        // 1. The Scribe (Pencatat)
        sqlx::query(
            "INSERT INTO ai_personas (name, description, system_prompt, is_active) VALUES (?, ?, ?, ?)"
        )
        .bind("The Scribe")
        .bind("Pencatat logbook yang rapi dan formal.")
        .bind("Kamu adalah Scribe. Tugasmu adalah merapikan input pengguna menjadi entri logbook yang terstruktur, jelas, dan kronologis. Jangan banyak bicara, fokus pada akurasi pencatatan.")
        .bind(true) // Default active
        .execute(&pool).await?;

        // 2. Discussion Partner (Teman Diskusi)
        sqlx::query(
            "INSERT INTO ai_personas (name, description, system_prompt, is_active) VALUES (?, ?, ?, ?)"
        )
        .bind("Discussion Partner")
        .bind("Teman brainstorming dan curhat.")
        .bind("Kamu adalah Partner diskusi. Dengarkan ide, keluh kesah, atau rencana pengguna. Berikan feedback konstruktif, pertanyaan pancingan, dan dukungan emosional. Jadilah teman yang cerdas.")
        .bind(false)
        .execute(&pool).await?;

        // 3. The Helper (Asisten)
        sqlx::query(
            "INSERT INTO ai_personas (name, description, system_prompt, is_active) VALUES (?, ?, ?, ?)"
        )
        .bind("The Helper")
        .bind("Asisten manajemen waktu dan pengingat.")
        .bind("Kamu adalah Helper. Tugasmu mengingatkan jadwal, mencari data di log lama, dan memastikan tidak ada tugas yang terlewat. Kamu sangat teliti terhadap waktu dan deadline.")
        .bind(false)
        .execute(&pool).await?;
        
        println!("🌱 Persona AI Dasar (Scribe, Partner, Helper) telah ditanam.");
    }

    // --- BACKGROUND TASK: GITHUB SYNC ---
let pool_for_sync = pool.clone();
tokio::spawn(async move {
    github_sync::start_github_polling(pool_for_sync).await;
});

    // ==========================================
    // 5. SERVER SETUP
    // ==========================================
    
let app = Router::new()
        .route("/", get(|| async { "Noty Logbook System v2.0 Online 🟢" }))
        
        // --- API LOGBOOK ---
        .route("/api/logs", get(logbook::get_logs).post(logbook::create_log))
        // GANTI :id JADI {id}
        .route("/api/logs/{id}", axum::routing::delete(logbook::delete_log)) 
        
        // --- API PERSONAS ---
        .route("/api/personas", get(personas::get_personas))
        // GANTI :id JADI {id}
        .route("/api/personas/{id}/activate", axum::routing::post(personas::activate_persona))

        .route("/api/ai/polish", post(ai_features::polish_content))

        .route("/api/settings", get(settings::get_settings).post(settings::update_settings))
        
        .layer(CorsLayer::permissive())
        .with_state(pool);


    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 Server Logbook berjalan di http://localhost:3000");
    axum::serve(listener, app).await?;

    Ok(())
}