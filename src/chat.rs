use axum::{
    extract::State,
    response::Json,
    Json as JsonBody,
};
use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};
use std::env;

// Request dari User
#[derive(Deserialize)]
pub struct ChatRequest {
    pub user_message: String,
}

// Response ke User
#[derive(Serialize)]
pub struct ChatResponse {
    pub reply: String,
}

// --- Struktur Body Gemini API ---
// KITA TAMBAHKAN 'Deserialize' dan 'Debug' DI SINI SUPAYA BISA DIBACA BALIK
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

// Struktur untuk menangkap balasan
#[derive(Deserialize, Debug)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Deserialize, Debug)]
struct GeminiCandidate {
    content: GeminiContent,
}

pub async fn handle_chat(
    State(pool): State<SqlitePool>,
    JsonBody(payload): JsonBody<ChatRequest>,
) -> Json<ChatResponse> {
    
    // 1. Cek "Inbox" Commit GitHub
    let pending_commits: Vec<(String, String)> = sqlx::query_as(
        "SELECT commit_hash, message FROM github_logs WHERE is_processed = FALSE ORDER BY timestamp DESC LIMIT 5"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    // 2. Susun Konteks
    let mut commit_context = String::new();
    if !pending_commits.is_empty() {
        commit_context.push_str("⚠️ PENDING GITHUB ACTIVITY (Ask the user about these!):\n");
        for (hash, msg) in &pending_commits {
            let short_hash = if hash.len() > 7 { &hash[0..7] } else { hash };
            commit_context.push_str(&format!("- Commit {}: {}\n", short_hash, msg));
        }
    }

    // 3. System Prompt
let system_instruction = format!(
        r#"
        You are Noty, a strict Code Review Assistant.
        
        [CRITICAL SYSTEM EVENT]
        Incoming GitHub Commits Status:
        {commit_context}
        
        [INSTRUCTION]
        CHECK the 'Incoming GitHub Commits Status' above explicitly:
        
        CASE A: If it contains commits (start with '⚠️'):
        1. You MUST IGNORE the user's current topic (login, etc).
        2. You MUST INTERRUPT the user. Say: "Hold on, I see new commits..."
        3. You MUST Ask specific clarifying questions about those commits first.
        4. ONLY after the user explains the commits, you may proceed to their topic.
        
        CASE B: If it is empty:
        - Proceed to help the user normally.

        [ADAPTIVE LANGUAGE]
        Reply in the language the user uses.
        
        USER INPUT:
        "{}"
        "#,
        payload.user_message
    );

    // 4. Kirim ke Gemini API
    let api_key = env::var("GEMINI_API_KEY").unwrap_or_default();
    // Default fallback ke 3.0 jika .env error
    let model = env::var("GEMINI_MODEL").unwrap_or("gemini-3-flash-preview".to_string()); 
    
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
        model, api_key
    );

    let res = client.post(&url)
        .json(&gemini_payload)
        .send()
        .await;

    let reply_text = match res {
        Ok(response) => {
            if response.status().is_success() {
                // Sekarang ini pasti berhasil karena sudah ada 'Deserialize'
                let data: GeminiResponse = response.json().await.unwrap_or(GeminiResponse { candidates: None });
                
                let ai_reply = data.candidates
                    .and_then(|c| c.into_iter().next())
                    .map(|c| c.content.parts[0].text.clone())
                    .unwrap_or("... (AI diam membisu, mungkin response kosong)".to_string());

                // Update status processed jika ada commit
                if !pending_commits.is_empty() {
                    let _ = sqlx::query("UPDATE github_logs SET is_processed = TRUE WHERE is_processed = FALSE")
                        .execute(&pool)
                        .await;
                }

                ai_reply
            } else {
                format!("⚠️ Error API Gemini ({}) - Cek API Key di .env", response.status())
            }
        },
        Err(e) => format!("⚠️ Koneksi Error: {}", e),
    };

    Json(ChatResponse { reply: reply_text })
}