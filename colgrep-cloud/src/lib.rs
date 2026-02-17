//! ColGREP Cloud - Cloudflare Worker for semantic code search
//!
//! Features:
//! - Password-based auth (register, login)
//! - User-scoped codebase storage (D1 + R2)
//! - Workers AI for embeddings
//! - Vectorize for vector search
//! - Web UI for login, register, dashboard

mod auth;
mod handlers;
mod html;
mod models;
mod storage;

use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        // Web UI - public
        .get("/", handlers::index_page)
        .get("/login", handlers::login_page)
        .get("/register", handlers::register_page)
        .get("/dashboard", handlers::dashboard_page)
        .get("/logout", handlers::logout_page)
        // Auth API
        .post_async("/api/register", handlers::register)
        .post_async("/api/login", handlers::login)
        .post_async("/api/logout", handlers::logout)
        // Codebases API (require auth)
        .get_async("/api/codebases", handlers::list_codebases)
        .post_async("/api/codebases", handlers::create_codebase)
        .get_async("/api/codebases/:id", handlers::get_codebase)
        .delete_async("/api/codebases/:id", handlers::delete_codebase)
        // Index upload & search
        .post_async("/api/codebases/:id/upload", handlers::upload_index)
        .post_async("/api/codebases/:id/search", handlers::search)
        .run(req, env)
        .await
}
