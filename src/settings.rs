use axum::{
    extract::State,
    response::Json,
    Json as JsonBody,
};
use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct AppSettings {
    pub username: String,
    pub theme: String,
    pub ai_provider: String,
    pub ai_model_name: String,
    pub is_api_key_set: bool, // Kita cuma kasih tau "udah diset" atau "belum", jangan kirim key aslinya
    pub use_local_ai: bool,
}

#[derive(Deserialize)]
pub struct UpdateSettingsRequest {
    pub username: String,
    pub ai_api_key: String, // Bisa kosong kalau gak mau update
    pub ai_model_name: String,
}

// GET SETTINGS
pub async fn get_settings(
    State(pool): State<SqlitePool>,
) -> Json<AppSettings> {
    let row = sqlx::query!(
        "SELECT username, theme, ai_provider, ai_model_name, ai_api_key, use_local_ai FROM app_settings LIMIT 1"
    )
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    if let Some(r) = row {
        Json(AppSettings {
            username: r.username.unwrap_or("User".to_string()),
            theme: r.theme.unwrap_or("dark".to_string()),
            ai_provider: r.ai_provider.unwrap_or("gemini".to_string()),
            ai_model_name: r.ai_model_name.unwrap_or("gemini-1.5-flash".to_string()),
            is_api_key_set: r.ai_api_key.is_some() && !r.ai_api_key.unwrap().is_empty(),
            use_local_ai: r.use_local_ai.unwrap_or(false),
        })
    } else {
        // Default fallback updated to 2026 standard
        Json(AppSettings {
            username: "User".to_string(),
            theme: "dark".to_string(),
            ai_provider: "gemini".to_string(),
            ai_model_name: "gemini-3-flash-preview".to_string(), // <--- Default Baru
            is_api_key_set: false,
            use_local_ai: false,
        })
    }
}

// UPDATE SETTINGS
pub async fn update_settings(
    State(pool): State<SqlitePool>,
    JsonBody(payload): JsonBody<UpdateSettingsRequest>,
) -> Json<String> {
    
    // 1. Update Username & Model
    let _ = sqlx::query!(
        "UPDATE app_settings SET username = ?, ai_model_name = ? WHERE id = (SELECT id FROM app_settings LIMIT 1)",
        payload.username,
        payload.ai_model_name
    )
    .execute(&pool)
    .await;

    // 2. Update API Key (Hanya jika diisi user)
    if !payload.ai_api_key.trim().is_empty() {
        let _ = sqlx::query!(
            "UPDATE app_settings SET ai_api_key = ? WHERE id = (SELECT id FROM app_settings LIMIT 1)",
            payload.ai_api_key
        )
        .execute(&pool)
        .await;
    }

    Json("Settings updated".to_string())
}