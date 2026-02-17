# ColGREP Cloud

Cloudflare Worker deployment of the ColGREP MCP server backend. Provides user auth, codebase storage (R2 + D1), and search over indexed code.

## Features

- **Password auth**: Register and login with email/password (PBKDF2-SHA256)
- **Codebases**: Create, list, view, delete codebases keyed by user
- **Index upload**: Upload code units (file_path, line_number, code, unit_type) to R2
- **Search**: Text search over stored code units (semantic search via Vectorize coming later)
- **Web UI**: Login, register, dashboard pages

## Prerequisites

- [wrangler](https://developers.cloudflare.com/workers/wrangler/install-and-update/)
- Rust + wasm32 target: `rustup target add wasm32-unknown-unknown`

## Setup

### 1. Create Cloudflare resources

```bash
# D1 database
wrangler d1 create colgrep-cloud
# Copy the database_id into wrangler.toml [[d1_databases]].database_id

# R2 bucket
wrangler r2 bucket create colgrep-code

# KV namespace for sessions
wrangler kv namespace create SESSIONS
# Copy the id into wrangler.toml [[kv_namespaces]].id
```

### 2. Update wrangler.toml

Replace placeholders in `wrangler.toml`:
- `database_id` from `wrangler d1 create`
- `id` under `[[kv_namespaces]]` from `wrangler kv namespace create`

### 3. Run migrations

```bash
wrangler d1 execute colgrep-cloud --remote --file=./migrations/0001_initial.sql
```

### 4. Deploy

```bash
wrangler deploy
```

## API

All API routes (except pages) require `Authorization: Bearer <token>`.

| Method | Path | Description |
|-------|------|-------------|
| POST | /api/register | Register (body: `{email, password}`) |
| POST | /api/login | Login (body: `{email, password}`) |
| POST | /api/logout | Logout (invalidates token) |
| GET | /api/codebases | List user's codebases |
| POST | /api/codebases | Create codebase (body: `{name, root_path}`) |
| GET | /api/codebases/:id | Get codebase |
| DELETE | /api/codebases/:id | Delete codebase |
| POST | /api/codebases/:id/upload | Upload index (body: `{code_units: [...]}`) |
| POST | /api/codebases/:id/search | Search (body: `{query, max_results?}`) |

## Pages

- `/` – Index
- `/login` – Login form
- `/register` – Register form
- `/dashboard` – Dashboard (requires auth, stores token in localStorage)
- `/logout` – Clears token and redirects
