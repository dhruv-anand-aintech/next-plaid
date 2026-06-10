//! HTTP request handlers

use crate::auth;
use crate::embeddings;
use crate::html;
use crate::models::*;
use crate::storage;
use worker::*;

fn get_user_from_request(req: &Request) -> Option<String> {
    req.headers()
        .get("Authorization")
        .ok()
        .flatten()
        .and_then(|v| v.strip_prefix("Bearer ").map(|s| s.to_string()))
}

async fn get_user_id_from_token(ctx: &RouteContext<()>, token: &str) -> Option<String> {
    let kv = ctx.kv("SESSIONS").ok()?;
    kv.get(token).text().await.ok().flatten()
}

fn json_error(msg: &str, status: u16) -> Result<Response> {
    Ok(Response::from_json(&serde_json::json!({"error": msg}))?.with_status(status))
}

pub fn index_page(_: Request, _: RouteContext<()>) -> Result<Response> {
    Response::ok(html::INDEX_HTML)
}

pub fn login_page(_: Request, _: RouteContext<()>) -> Result<Response> {
    Response::ok(html::LOGIN_HTML)
}

pub fn register_page(_: Request, _: RouteContext<()>) -> Result<Response> {
    Response::ok(html::REGISTER_HTML)
}

pub fn dashboard_page(_: Request, _: RouteContext<()>) -> Result<Response> {
    Response::ok(html::DASHBOARD_HTML)
}

pub fn logout_page(_: Request, _: RouteContext<()>) -> Result<Response> {
    Response::ok(
        r#"<!DOCTYPE html><html><head><script>localStorage.removeItem('token');location.href='/';</script></head><body>Logging out...</body></html>"#,
    )
}

pub async fn register(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let body: RegisterRequest = req
        .json()
        .await
        .map_err(|e| Error::RustError(format!("Invalid JSON: {}", e)))?;
    if body.email.is_empty() || body.password.len() < 8 {
        return json_error("Email required, password min 8 chars", 400);
    }

    let d1 = ctx.env.d1("DB")?;
    if storage::get_user_by_email(&d1, &body.email).await?.is_some() {
        return json_error("Email already registered", 409);
    }

    let mut salt_arr = [0u8; 16];
    getrandom::getrandom(&mut salt_arr).map_err(|e| Error::RustError(e.to_string()))?;
    let password_hash = auth::hash_password(&body.password, &salt_arr);
    let user_id = storage::generate_id();

    storage::create_user(&d1, &user_id, &body.email, &password_hash).await?;

    let token = storage::generate_token()?;
    let kv = ctx.kv("SESSIONS")?;
    kv.put(&token, &user_id)?.execute().await?;

    let auth = AuthResponse {
        token: token.clone(),
        user_id: user_id.clone(),
        email: body.email,
    };
    Response::from_json(&auth)
}

pub async fn login(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let body: LoginRequest = req
        .json()
        .await
        .map_err(|e| Error::RustError(format!("Invalid JSON: {}", e)))?;
    if body.email.is_empty() || body.password.is_empty() {
        return json_error("Email and password required", 400);
    }

    let d1 = ctx.env.d1("DB")?;
    let Some((user_id, password_hash)) = storage::get_user_by_email(&d1, &body.email).await? else {
        return json_error("Invalid email or password", 401);
    };

    if !auth::verify_password(&body.password, &password_hash).map_err(Error::RustError)? {
        return json_error("Invalid email or password", 401);
    }

    let token = storage::generate_token()?;
    let kv = ctx.kv("SESSIONS")?;
    kv.put(&token, &user_id)?.execute().await?;

    let auth = AuthResponse {
        token: token.clone(),
        user_id: user_id.clone(),
        email: body.email,
    };
    Response::from_json(&auth)
}

pub async fn logout(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(token) = get_user_from_request(&req) {
        let kv = ctx.kv("SESSIONS")?;
        kv.delete(&token).await?;
    }
    Response::ok("{}")
}

pub async fn list_codebases(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let token = get_user_from_request(&req)
        .ok_or_else(|| Error::RustError("No Authorization header".into()))?;
    let user_id = get_user_id_from_token(&ctx, &token)
        .await
        .ok_or_else(|| Error::RustError("Invalid token".into()))?;

    let d1 = ctx.env.d1("DB")?;
    let rows = storage::list_codebases(&d1, &user_id).await?;
    let codebases: Vec<CodebaseResponse> = rows
        .into_iter()
        .map(|r| CodebaseResponse {
            id: r.id,
            name: r.name,
            root_path: r.root_path,
            file_count: r.file_count as u32,
            code_unit_count: r.code_unit_count as u32,
            last_indexed: r.last_indexed,
            created_at: r.created_at,
        })
        .collect();
    Response::from_json(&codebases)
}

pub async fn create_codebase(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let token = get_user_from_request(&req)
        .ok_or_else(|| Error::RustError("No Authorization header".into()))?;
    let user_id = get_user_id_from_token(&ctx, &token)
        .await
        .ok_or_else(|| Error::RustError("Invalid token".into()))?;

    let body: CreateCodebaseRequest = req
        .json()
        .await
        .map_err(|e| Error::RustError(format!("Invalid JSON: {}", e)))?;
    if body.name.is_empty() || body.root_path.is_empty() {
        return Response::error("Name and root_path required", 400);
    }

    let d1 = ctx.env.d1("DB")?;
    let id = storage::generate_id();
    storage::create_codebase(&d1, &id, &user_id, &body.name, &body.root_path).await?;

    let cb = CodebaseResponse {
        id: id.clone(),
        name: body.name,
        root_path: body.root_path,
        file_count: 0,
        code_unit_count: 0,
        last_indexed: None,
        created_at: js_sys::Date::new_0()
            .to_iso_string()
            .as_string()
            .unwrap_or_default(),
    };
    Response::from_json(&cb)
}

pub async fn get_codebase(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let token = get_user_from_request(&req)
        .ok_or_else(|| Error::RustError("No Authorization header".into()))?;
    let user_id = get_user_id_from_token(&ctx, &token)
        .await
        .ok_or_else(|| Error::RustError("Invalid token".into()))?;
    let id = ctx
        .param("id")
        .ok_or_else(|| Error::RustError("No id".into()))?
        .to_string();

    let d1 = ctx.env.d1("DB")?;
    let Some(row) = storage::get_codebase(&d1, &id, &user_id).await? else {
        return Response::error("Not found", 404);
    };

    let cb = CodebaseResponse {
        id: row.id,
        name: row.name,
        root_path: row.root_path,
        file_count: row.file_count as u32,
        code_unit_count: row.code_unit_count as u32,
        last_indexed: row.last_indexed,
        created_at: row.created_at,
    };
    Response::from_json(&cb)
}

pub async fn delete_codebase(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let token = get_user_from_request(&req)
        .ok_or_else(|| Error::RustError("No Authorization header".into()))?;
    let user_id = get_user_id_from_token(&ctx, &token)
        .await
        .ok_or_else(|| Error::RustError("Invalid token".into()))?;
    let id = ctx
        .param("id")
        .ok_or_else(|| Error::RustError("No id".into()))?
        .to_string();

    let d1 = ctx.env.d1("DB")?;
    if !storage::delete_codebase(&d1, &id, &user_id).await? {
        return Response::error("Not found", 404);
    }
    Response::ok("{}")
}

pub async fn upload_index(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let token = get_user_from_request(&req)
        .ok_or_else(|| Error::RustError("No Authorization header".into()))?;
    let user_id = get_user_id_from_token(&ctx, &token)
        .await
        .ok_or_else(|| Error::RustError("Invalid token".into()))?;
    let id = ctx
        .param("id")
        .ok_or_else(|| Error::RustError("No id".into()))?
        .to_string();

    let d1 = ctx.env.d1("DB")?;
    let Some(_) = storage::get_codebase(&d1, &id, &user_id).await? else {
        return Response::error("Not found", 404);
    };

    let body: UploadIndexRequest = req
        .json()
        .await
        .map_err(|e| Error::RustError(format!("Invalid JSON: {}", e)))?;

    let mut blob = IndexBlob {
        code_units: body.code_units,
        embeddings: None,
    };

    // If HF_TOKEN is set, fetch embeddings via Hugging Face Inference API
    if let Ok(secret) = ctx.secret("HF_TOKEN") {
        let hf_token = secret.to_string();
        let texts: Vec<String> = blob.code_units.iter().map(|u| u.code.clone()).collect();
        if let Ok(embs) = embeddings::fetch_embeddings(&hf_token, &texts, None).await {
            if embs.len() == blob.code_units.len() {
                blob.embeddings = Some(embs);
            }
        }
    }

    let bucket = ctx.env.bucket("CODE_STORAGE")?;
    let key = format!("{}/{}", user_id, id);
    let data = serde_json::to_vec(&blob).map_err(|e| Error::RustError(e.to_string()))?;
    storage::store_code_units_in_r2(&bucket, &key, &data).await?;

    let file_count = blob
        .code_units
        .iter()
        .map(|u| u.file_path.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();
    let code_unit_count = blob.code_units.len() as u32;
    storage::update_codebase_stats(&d1, &id, file_count as u32, code_unit_count).await?;

    Response::from_json(&serde_json::json!({
        "ok": true,
        "file_count": file_count,
        "code_unit_count": code_unit_count
    }))
}

pub async fn search(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let token = get_user_from_request(&req)
        .ok_or_else(|| Error::RustError("No Authorization header".into()))?;
    let user_id = get_user_id_from_token(&ctx, &token)
        .await
        .ok_or_else(|| Error::RustError("Invalid token".into()))?;
    let id = ctx
        .param("id")
        .ok_or_else(|| Error::RustError("No id".into()))?
        .to_string();

    let search_req: SearchRequest = req
        .json()
        .await
        .map_err(|e| Error::RustError(format!("Invalid JSON: {}", e)))?;

    let d1 = ctx.env.d1("DB")?;
    let Some(_) = storage::get_codebase(&d1, &id, &user_id).await? else {
        return Response::error("Not found", 404);
    };

    let bucket = ctx.env.bucket("CODE_STORAGE")?;
    let key = format!("{}/{}", user_id, id);
    let obj = bucket
        .get(&key)
        .execute()
        .await
        .map_err(|e| Error::RustError(e.to_string()))?
        .ok_or_else(|| Error::RustError("No index - upload first".into()))?;
    let obj_body = obj
        .body()
        .ok_or_else(|| Error::RustError("No body".into()))?;
    let body_bytes = obj_body
        .bytes()
        .await
        .map_err(|e| Error::RustError(e.to_string()))?;

    // Parse index blob (supports legacy array format or new {code_units, embeddings})
    let blob: IndexBlob = match serde_json::from_slice::<serde_json::Value>(&body_bytes) {
        Ok(serde_json::Value::Array(arr)) => IndexBlob {
            code_units: serde_json::from_value(serde_json::Value::Array(arr))
                .map_err(|e| Error::RustError(e.to_string()))?,
            embeddings: None,
        },
        Ok(v) => serde_json::from_value(v).map_err(|e| Error::RustError(e.to_string()))?,
        Err(e) => return Err(Error::RustError(e.to_string())),
    };

    let code_units = &blob.code_units;
    let q = search_req.query.to_lowercase();
    let max_results = search_req.max_results.unwrap_or(15).min(50) as usize;

    let mut matches: Vec<(f32, &CodeUnitUpload)> = if let (Some(embeddings), Ok(hf_secret)) =
        (blob.embeddings.as_ref(), ctx.secret("HF_TOKEN"))
    {
        // Semantic search: embed query via HF, compute cosine similarity
        let hf_token = hf_secret.to_string();
        let query_embs = embeddings::fetch_embeddings(&hf_token, &[search_req.query.clone()], None)
            .await
            .ok()
            .and_then(|v| v.into_iter().next());
        if let (Some(query_emb), embs) = (query_embs, embeddings) {
            code_units
                .iter()
                .zip(embs.iter())
                .map(|(u, emb)| (embeddings::cosine_similarity(&query_emb, emb), u))
                .filter(|(score, _)| *score > 0.0)
                .collect()
        } else {
            vec![]
        }
    } else {
        // Fallback: text search
        code_units
            .iter()
            .filter_map(|u| {
                if u.code.to_lowercase().contains(&q) {
                    Some((1.0, u))
                } else if u.file_path.to_lowercase().contains(&q) {
                    Some((0.8, u))
                } else {
                    None
                }
            })
            .collect()
    };

    matches.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let results: Vec<SearchResultResponse> = matches
        .into_iter()
        .take(max_results)
        .map(|(score, u)| SearchResultResponse {
            file_path: u.file_path.clone(),
            line_number: u.line_number,
            snippet: u.code.clone(),
            score,
        })
        .collect();
    Response::from_json(&results)
}
