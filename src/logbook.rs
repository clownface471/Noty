use axum::{
    extract::{Path, State, Query},
    response::Json,
    Json as JsonBody,
    http::StatusCode,
};
use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// --- MODEL DATA (Sesuai Database) ---
#[derive(Serialize, sqlx::FromRow)]
pub struct LogEntry {
    pub id: i64,
    pub content: String,
    pub entry_date: String,     // Format YYYY-MM-DD
    pub entry_time: String,     // Format HH:MM:SS
    pub created_at: String,     // Timestamp sistem
    pub tags: Option<String>,   // JSON String: "['ide', 'kerja']"
    pub category: String,
    pub mood: Option<String>,
    pub source: String,
}

// --- INPUT DARI USER (Frontend) ---
#[derive(Deserialize)]
pub struct CreateLogRequest {
    pub content: String,
    pub entry_date: String, // User bisa set tanggal mundur (backdate)
    pub entry_time: String,
    pub tags: Vec<String>,  // Frontend kirim array, kita ubah jadi string nanti
    pub category: Option<String>,
    pub mood: Option<String>,
    pub source: Option<String>, // 'Manual' defaultnya
}

#[derive(Deserialize)]
pub struct LogFilter {
    pub date: Option<String>, // Filter per tanggal
    pub tag: Option<String>,  // Filter per tag
}

// --- HANDLERS (Fungsi API) ---

// 1. GET ALL LOGS (Bisa filter by date)
pub async fn get_logs(
    State(pool): State<SqlitePool>,
    Query(params): Query<LogFilter>,
) -> Result<Json<Vec<LogEntry>>, StatusCode> {
    
    // Query dasar: Ambil yang TIDAK dihapus (Soft Delete)
    let mut sql = "SELECT id, content, entry_date, entry_time, CAST(created_at AS TEXT) as created_at, tags, category, mood, source FROM log_entries WHERE is_deleted = FALSE".to_string();
    
    // Filter tanggal (jika ada)
    if let Some(d) = params.date {
        sql.push_str(&format!(" AND entry_date = '{}'", d));
    }
    
    // Urutkan dari yang terbaru (waktu entry, bukan waktu input)
    sql.push_str(" ORDER BY entry_date DESC, entry_time DESC");

    let logs = sqlx::query_as::<_, LogEntry>(&sql)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            eprintln!("❌ Error fetch logs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(logs))
}

// 2. CREATE NEW LOG
pub async fn create_log(
    State(pool): State<SqlitePool>,
    JsonBody(payload): JsonBody<CreateLogRequest>,
) -> Result<Json<String>, StatusCode> {
    
    // Convert array tags ke JSON String
    let tags_json = serde_json::to_string(&payload.tags).unwrap_or("[]".to_string());
    let category = payload.category.unwrap_or("General".to_string());
    let source = payload.source.unwrap_or("Manual".to_string());

    sqlx::query(
        "INSERT INTO log_entries (content, entry_date, entry_time, tags, category, mood, source) 
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(payload.content)
    .bind(payload.entry_date)
    .bind(payload.entry_time)
    .bind(tags_json)
    .bind(category)
    .bind(payload.mood)
    .bind(source)
    .execute(&pool)
    .await
    .map_err(|e| {
        eprintln!("❌ Error create log: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json("Log berhasil dicatat".to_string()))
}

// 3. DELETE LOG (Soft Delete)
pub async fn delete_log(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<String>, StatusCode> {
    sqlx::query("UPDATE log_entries SET is_deleted = TRUE WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json("Log dihapus (disimpan di trash)".to_string()))
}