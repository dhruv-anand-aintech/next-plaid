//! D1 and R2 storage helpers

use wasm_bindgen::JsValue;
use worker::*;

fn to_d1_arg(s: &str) -> JsValue {
    JsValue::from_str(s)
}

pub fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", t)
}

pub fn generate_token() -> Result<String> {
    let mut bytes = [0u8; 24];
    getrandom::getrandom(&mut bytes).map_err(|e| worker::Error::RustError(e.to_string()))?;
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD
        .encode(bytes)
        .trim_end_matches('=')
        .replace('/', "_")
        .replace('+', "-"))
}

pub async fn get_user_by_email(d1: &D1Database, email: &str) -> Result<Option<(String, String)>> {
    let stmt = d1.prepare("SELECT id, password_hash FROM users WHERE email = ?1");
    let query = stmt.bind(&[to_d1_arg(email)])?;
    let result = query.first::<(String, String)>(None).await?;
    Ok(result)
}

pub async fn create_user(
    d1: &D1Database,
    id: &str,
    email: &str,
    password_hash: &str,
) -> Result<()> {
    let stmt = d1.prepare("INSERT INTO users (id, email, password_hash) VALUES (?1, ?2, ?3)");
    let query = stmt.bind(&[to_d1_arg(id), to_d1_arg(email), to_d1_arg(password_hash)])?;
    query.run().await?;
    Ok(())
}

pub async fn create_codebase(
    d1: &D1Database,
    id: &str,
    user_id: &str,
    name: &str,
    root_path: &str,
) -> Result<()> {
    let stmt =
        d1.prepare("INSERT INTO codebases (id, user_id, name, root_path) VALUES (?1, ?2, ?3, ?4)");
    let query = stmt.bind(&[
        to_d1_arg(id),
        to_d1_arg(user_id),
        to_d1_arg(name),
        to_d1_arg(root_path),
    ])?;
    query.run().await?;
    Ok(())
}

pub async fn list_codebases(d1: &D1Database, user_id: &str) -> Result<Vec<CodebaseRow>> {
    let stmt = d1.prepare(
        "SELECT id, name, root_path, file_count, code_unit_count, last_indexed, created_at
         FROM codebases WHERE user_id = ?1 ORDER BY created_at DESC",
    );
    let query = stmt.bind(&[to_d1_arg(user_id)])?;
    let result = query.all().await?;
    let rows: Vec<CodebaseRow> = result.results()?;
    Ok(rows)
}

pub async fn get_codebase(d1: &D1Database, id: &str, user_id: &str) -> Result<Option<CodebaseRow>> {
    let stmt = d1.prepare(
        "SELECT id, name, root_path, file_count, code_unit_count, last_indexed, created_at
         FROM codebases WHERE id = ?1 AND user_id = ?2",
    );
    let query = stmt.bind(&[to_d1_arg(id), to_d1_arg(user_id)])?;
    let result = query.first::<CodebaseRow>(None).await?;
    Ok(result)
}

pub async fn delete_codebase(d1: &D1Database, id: &str, user_id: &str) -> Result<bool> {
    let stmt = d1.prepare("DELETE FROM codebases WHERE id = ?1 AND user_id = ?2");
    let query = stmt.bind(&[to_d1_arg(id), to_d1_arg(user_id)])?;
    let result = query.run().await?;
    Ok(result
        .meta()?
        .and_then(|m| m.changes)
        .map(|c| c > 0)
        .unwrap_or(false))
}

pub async fn store_code_units_in_r2(bucket: &Bucket, key: &str, data: &[u8]) -> Result<()> {
    bucket.put(key, data.to_vec()).execute().await?;
    Ok(())
}

pub async fn update_codebase_stats(
    d1: &D1Database,
    id: &str,
    file_count: u32,
    code_unit_count: u32,
) -> Result<()> {
    let stmt = d1.prepare(
        "UPDATE codebases SET file_count = ?1, code_unit_count = ?2, last_indexed = datetime('now'), updated_at = datetime('now') WHERE id = ?3",
    );
    let query = stmt.bind(&[
        JsValue::from_f64(file_count as f64),
        JsValue::from_f64(code_unit_count as f64),
        to_d1_arg(id),
    ])?;
    query.run().await?;
    Ok(())
}

#[derive(Debug, serde::Deserialize)]
pub struct CodebaseRow {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub file_count: i64,
    pub code_unit_count: i64,
    pub last_indexed: Option<String>,
    pub created_at: String,
}
