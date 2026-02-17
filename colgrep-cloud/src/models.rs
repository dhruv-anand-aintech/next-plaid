//! Request/response models

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCodebaseRequest {
    pub name: String,
    pub root_path: String,
}

#[derive(Debug, Serialize)]
pub struct CodebaseResponse {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub file_count: u32,
    pub code_unit_count: u32,
    pub last_indexed: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CodeUnitUpload {
    pub file_path: String,
    pub line_number: u32,
    pub code: String,
    pub unit_type: String,
    pub language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UploadIndexRequest {
    pub code_units: Vec<CodeUnitUpload>,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub max_results: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SearchResultResponse {
    pub file_path: String,
    pub line_number: u32,
    pub snippet: String,
    pub score: f32,
}
