use axum::{
    extract::State,
    response::Json,
    Json as JsonBody,
};
use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize)]
pub struct ChatRequest {
    pub user_message: String,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub reply: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize, Debug)]
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

// Struktur kecil buat ambil API Key dari DB
#[derive(sqlx::FromRow)]
struct UserKeys {
    username: String,
    profession: Option<String>, // <--- Tambah
    api_key: Option<String>,
}

pub async fn handle_chat(
    State(pool): State<SqlitePool>,
    JsonBody(payload): JsonBody<ChatRequest>,
) -> Json<ChatResponse> {
    
    // 1. Ambil Data User
    let user_data: Option<UserKeys> = sqlx::query_as(
        "SELECT username, profession, api_key FROM users LIMIT 1"
    )
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    let (username, profession, api_key) = match user_data {
        Some(u) => (
            u.username, 
            u.profession.unwrap_or("General Assistant".to_string()), 
            u.api_key.unwrap_or_default()
        ),
        None => ("User".to_string(), "General Assistant".to_string(), "".to_string()),
    };

    if api_key.is_empty() {
        return Json(ChatResponse { reply: "⚠️ API Key belum diatur.".to_string() });
    }

    // 2. Cek Inbox GitHub
    let pending_commits: Vec<(String, String)> = sqlx::query_as(
        "SELECT commit_hash, message FROM github_logs WHERE is_processed = FALSE ORDER BY timestamp DESC LIMIT 5"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    // 3. Konteks GitHub
    let mut commit_context = String::new();
    if !pending_commits.is_empty() {
        commit_context.push_str("⚠️ INCOMING WORK UPDATES (GitHub):\n");
        for (hash, msg) in &pending_commits {
            let short_hash = if hash.len() > 7 { &hash[0..7] } else { hash };
            commit_context.push_str(&format!("- ID {}: {}\n", short_hash, msg));
        }
    }

    // 4. SYSTEM PROMPT DINAMIS
    let system_instruction = format!(
        r#"
        You are Noty, an intelligent assistant acting as a **{}** for **{}**.
        
        [CONTEXT: Incoming Updates]
        {commit_context}
        
        [CORE BEHAVIOR]
        1. ADAPT TO YOUR ROLE: 
           - If user is 'Novelist', treat commits as 'Manuscript Revisions'.
           - If user is 'Accountant', treat commits as 'Financial Records'.
           - If user is 'Developer', treat commits as 'Code Updates'.
           - If user is 'General Assistant', just be helpful.
           
        2. CHECK the context above.
           - If there are incoming updates, politely bring them to the user's attention relevant to your role.
           - Example (Novelist): "I see you've revised the 'Chapter 3' draft. Shall we review the plot consistency?"
           
        3. TONE:
           - Professional, aligned with the specific profession designated above.
           - Reply in the language the user uses.
        
        USER INPUT:
        "{}"
        "#,
        profession, // <--- Peran Noty berubah di sini!
        username,
        payload.user_message
    );

    // 5. Kirim ke Gemini
    let model = env::var("GEMINI_MODEL").unwrap_or("gemini-1.5-flash".to_string());
    
    let client = reqwest::Client::new();
    let gemini_payload = GeminiRequest {
        contents: vec![
            GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart { text: system_instruction }],
            }
        ],
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", 
        model, api_key // <--- Pakai API Key dari DB
    );

    let res = client.post(&url)
        .json(&gemini_payload)
        .send()
        .await;

    let reply_text = match res {
        Ok(response) => {
            if response.status().is_success() {
                let data: GeminiResponse = response.json().await.unwrap_or(GeminiResponse { candidates: None });
                let ai_reply = data.candidates
                    .and_then(|c| c.into_iter().next())
                    .map(|c| c.content.parts[0].text.clone())
                    .unwrap_or("...".to_string());

                if !pending_commits.is_empty() {
                    let _ = sqlx::query("UPDATE github_logs SET is_processed = TRUE WHERE is_processed = FALSE")
                        .execute(&pool)
                        .await;
                }
                ai_reply
            } else {
                format!("⚠️ Error API Gemini ({}) - Cek Settings", response.status())
            }
        },
        Err(e) => format!("⚠️ Koneksi Error: {}", e),
    };

    Json(ChatResponse { reply: reply_text })
}