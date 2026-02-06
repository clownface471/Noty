use axum::{
    extract::State,
    response::Json,
    Json as JsonBody,
};
use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct PolishRequest {
    pub draft_content: String,
}

#[derive(Serialize)]
pub struct PolishResponse {
    pub polished_content: String,
}

// --- STRUKTUR DATA GEMINI (Fix: Tambahkan Deserialize) ---

#[derive(Serialize, Deserialize, Debug)] // <--- FIX: Tambah Deserialize
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize, Deserialize, Debug)] // <--- FIX: Tambah Deserialize
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize, Debug)] // <--- FIX: Tambah Deserialize
struct GeminiPart {
    text: String,
}

#[derive(Deserialize, Debug)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Deserialize, Debug)]
struct GeminiCandidate {
    content: GeminiContent,
}

// --- HANDLER ---

pub async fn polish_content(
    State(pool): State<SqlitePool>,
    JsonBody(payload): JsonBody<PolishRequest>,
) -> Json<PolishResponse> {
    
    // 1. Ambil API Key & Active Persona dari DB
    let settings = sqlx::query!("SELECT ai_api_key, ai_model_name FROM app_settings LIMIT 1")
        .fetch_optional(&pool).await.unwrap_or(None);

    let active_persona = sqlx::query!("SELECT name, system_prompt FROM ai_personas WHERE is_active = TRUE LIMIT 1")
        .fetch_optional(&pool).await.unwrap_or(None);

    // Default values
// Default values
    let api_key = settings.as_ref().and_then(|s| s.ai_api_key.clone()).unwrap_or_default();
    
// --- FIX SAFETY NET ---
    // Ambil model dari DB. Kalau kosong atau None, paksa pakai 'gemini-3-flash-preview'
    let model = settings.as_ref()
        .and_then(|s| s.ai_model_name.clone())
        .filter(|s| !s.is_empty()) // Filter: kalau string kosong, jadikan None
        .unwrap_or("gemini-3-flash-preview".to_string()); // Fallback aman
    
    // Default Persona: Scribe
    let (persona_name, persona_prompt) = match active_persona {
        Some(p) => (p.name, p.system_prompt),
        None => ("Scribe".to_string(), "You are a professional editor. Rewrite the text clearly.".to_string())
    };

    if api_key.is_empty() {
        return Json(PolishResponse { polished_content: "⚠️ API Key belum diset di Settings.".to_string() });
    }

// 2. Susun Prompt (Versi Bebas Format)
    let system_instruction = format!(
        r#"
        ROLE: You are Noty, utilizing the '{}' persona.
        CORE INSTRUCTION: {}
        
        TASK:
        Rewrite the user's raw draft below into a clean, structured logbook entry. 
        
        FORMATTING GUIDELINES:
        - You MAY use **Bold** for emphasis on key results or names.
        - You MAY use *Italic* for thoughts or nuances.
        - You MAY use Bullet Points (-) for lists.
        - You MAY use `Code Block` if technical terms appear.
        - Keep headers minimal (use ### if splitting sections, avoid # or ##).
        - Ensure the output is visually pleasing when rendered in Markdown.
        
        Draft to Rewrite:
        "{}"
        "#,
        persona_name,
        persona_prompt,
        payload.draft_content
    );
    // 3. Tembak Gemini
    let client = reqwest::Client::new();
    let gemini_payload = GeminiRequest {
        contents: vec![GeminiContent { role: "user".to_string(), parts: vec![GeminiPart { text: system_instruction }] }],
    };

    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", model, api_key);
    
    let res = client.post(&url).json(&gemini_payload).send().await;

    let result_text = match res {
        Ok(response) => {
            if response.status().is_success() {
                let data: GeminiResponse = response.json().await.unwrap_or(GeminiResponse { candidates: None });
                data.candidates.and_then(|c| c.into_iter().next())
                    .map(|c| c.content.parts[0].text.clone())
                    .unwrap_or(payload.draft_content) // Kalau gagal parse, balikin original
            } else {
                format!("Error AI: {}", response.status())
            }
        },
        Err(_) => "Error Connection".to_string(),
    };

    Json(PolishResponse { polished_content: result_text })
}