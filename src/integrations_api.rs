use axum::{
    extract::State,
    response::Json,
    Json as JsonBody,
};
use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct GithubConfigResponse {
    pub repo_name: String,
    pub is_active: bool,
    pub is_token_set: bool, // Kita sembunyikan token aslinya
}

#[derive(Deserialize)]
pub struct GithubConfigReq {
    pub repo_name: String,
    pub token: String, // Bisa kosong kalau gak mau update
    pub is_active: bool,
}

// GET Config
pub async fn get_github_config(
    State(pool): State<SqlitePool>,
) -> Json<GithubConfigResponse> {
    let row = sqlx::query!(
        "SELECT config_json, is_active, api_key FROM integrations WHERE service_name = 'github' LIMIT 1"
    )
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    if let Some(r) = row {
        Json(GithubConfigResponse {
            repo_name: r.config_json.unwrap_or_default(),
            is_active: r.is_active.unwrap_or(false),
            is_token_set: r.api_key.is_some() && !r.api_key.unwrap().is_empty(),
        })
    } else {
        Json(GithubConfigResponse {
            repo_name: "".to_string(),
            is_active: false,
            is_token_set: false,
        })
    }
}

// UPDATE Config
pub async fn update_github_config(
    State(pool): State<SqlitePool>,
    JsonBody(payload): JsonBody<GithubConfigReq>,
) -> Json<String> {
    
    // Cek apakah data sudah ada
    let exists = sqlx::query!("SELECT count(*) as count FROM integrations WHERE service_name = 'github'")
        .fetch_one(&pool).await.unwrap();

    if exists.count == 0 {
        // Insert Baru
        sqlx::query!(
            "INSERT INTO integrations (service_name, config_json, api_key, is_active) VALUES ('github', ?, ?, ?)",
            payload.repo_name, payload.token, payload.is_active
        ).execute(&pool).await.unwrap();
    } else {
        // Update yang ada
        // Kalau token kosong, jangan ditimpa (biarkan yang lama)
        if payload.token.trim().is_empty() {
            sqlx::query!(
                "UPDATE integrations SET config_json = ?, is_active = ? WHERE service_name = 'github'",
                payload.repo_name, payload.is_active
            ).execute(&pool).await.unwrap();
        } else {
            sqlx::query!(
                "UPDATE integrations SET config_json = ?, api_key = ?, is_active = ? WHERE service_name = 'github'",
                payload.repo_name, payload.token, payload.is_active
            ).execute(&pool).await.unwrap();
        }
    }

    Json("GitHub Config Updated".to_string())
}