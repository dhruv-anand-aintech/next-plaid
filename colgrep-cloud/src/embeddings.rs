//! Hugging Face Inference API for text embeddings

use worker::*;

const HF_API_BASE: &str = "https://api-inference.huggingface.co";
const DEFAULT_MODEL: &str = "sentence-transformers/all-MiniLM-L6-v2";
const BATCH_SIZE: usize = 16;

/// Fetch embeddings from Hugging Face Inference API.
/// Texts are batched to avoid timeouts and rate limits.
pub async fn fetch_embeddings(
    token: &str,
    texts: &[String],
    model: Option<&str>,
) -> Result<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }

    let model_id = model.unwrap_or(DEFAULT_MODEL);
    let url = format!("{}/models/{}", HF_API_BASE, model_id);

    let mut all_embeddings = Vec::with_capacity(texts.len());

    for chunk in texts.chunks(BATCH_SIZE) {
        let body_json = serde_json::json!({ "inputs": chunk });
        let body_str = body_json.to_string();

        let mut init = RequestInit::new();
        init.with_method(Method::Post);
        init.with_body(Some(wasm_bindgen::JsValue::from_str(&body_str)));

        let headers = Headers::new();
        headers
            .set("Authorization", &format!("Bearer {}", token))
            .map_err(|e| Error::RustError(e.to_string()))?;
        headers
            .set("Content-Type", "application/json")
            .map_err(|e| Error::RustError(e.to_string()))?;
        init.with_headers(headers);

        let req = Request::new_with_init(&url, &init)?;
        let mut resp = Fetch::Request(req).send().await?;

        if resp.status_code() < 200 || resp.status_code() >= 300 {
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::RustError(format!(
                "HF API error {}: {}",
                resp.status_code(),
                text
            )));
        }

        let body = resp.json::<serde_json::Value>().await.map_err(|e| {
            Error::RustError(format!("Parse HF response: {}", e))
        })?;

        let chunk_embeddings: Vec<Vec<f32>> = match body {
            serde_json::Value::Array(arr) => arr
                .into_iter()
                .map(|v| {
                    serde_json::from_value(v)
                        .map_err(|e| Error::RustError(format!("Embedding format: {}", e)))
                })
                .collect::<Result<Vec<_>>>()?,
            serde_json::Value::Object(obj) => {
                if let Some(emb) = obj.get("embeddings") {
                    serde_json::from_value(emb.clone())
                        .map_err(|e| Error::RustError(format!("Embeddings field: {}", e)))?
                } else if let Some(arr) = obj.get("output") {
                    serde_json::from_value(arr.clone())
                        .map_err(|e| Error::RustError(format!("Output field: {}", e)))?
                } else {
                    return Err(Error::RustError("Unknown HF response format".into()));
                }
            }
            _ => return Err(Error::RustError("HF response not array or object".into())),
        };

        for emb in chunk_embeddings {
            all_embeddings.push(emb);
        }
    }

    Ok(all_embeddings)
}

/// Cosine similarity between two unit-norm vectors.
#[inline]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    dot // Assume API returns normalized vectors
}
