use axum::{
    extract::{State, Path},
    response::Json,
    http::StatusCode,
};
use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};

#[derive(Serialize, sqlx::FromRow)]
pub struct Persona {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub is_custom: bool,
}

// 1. GET ALL PERSONAS
pub async fn get_personas(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Persona>>, StatusCode> {
    let personas = sqlx::query_as::<_, Persona>(
        "SELECT id, name, description, is_active, is_custom FROM ai_personas ORDER BY id ASC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(personas))
}

// 2. ACTIVATE PERSONA (Switch Personality)
pub async fn activate_persona(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<String>, StatusCode> {
    
    // Matikan semua dulu
    sqlx::query("UPDATE ai_personas SET is_active = FALSE").execute(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Hidupkan yang dipilih
    sqlx::query("UPDATE ai_personas SET is_active = TRUE WHERE id = ?").bind(id).execute(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(format!("Persona ID {} diaktifkan", id)))
}