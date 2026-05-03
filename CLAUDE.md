# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project: Aura Notes

A local-first, self-hosted Zettelkasten note-taking app targeting FreeBSD. Design goal: Typora/Obsidian-level UX with minimal RAM/CPU usage.

## Tech Stack

- **Backend:** Rust + Axum (`/backend`)
- **Frontend:** SvelteKit + Tailwind CSS (`/frontend`)
- **Editor:** TipTap (headless WYSIWYG, bidirectional Markdown)
- **Storage:** Flat `.md` files on disk (no database)
- **Indexing:** In-memory `HashMap<name, NoteRow>` (built at startup, updated on every write) + in-memory backlink index
- **External tools:** Pandoc (export), Excalidraw (diagrams)

## Development Commands

### Backend (Rust/Axum)
```bash
cd backend
cargo build              # compile
cargo run                # start server (default: http://localhost:3000)
cargo clippy             # lint
cargo fmt                # format
```

There are currently no automated tests in the backend.

### Frontend (SvelteKit)
```bash
cd frontend
npm install              # install deps
npm run dev              # dev server (default: http://localhost:5173)
npm run build            # production build
npm run preview          # serve production build locally
npm run check            # type-check with svelte-check
npm run check:watch      # type-check in watch mode
```

### Single binary (production)
```bash
cd frontend && npm run build   # build static assets → frontend/build/
cd backend && cargo build      # embeds frontend/build via rust-embed
./backend/target/debug/aura-notes  # serves everything on http://localhost:3000
```

The release binary embeds all frontend assets at compile time (via `rust-embed`). In dev mode, assets are read from `../frontend/build` at runtime, so rebuild the frontend whenever you change it.

Set `VITE_API_BASE=''` (empty) for production builds — requests go to the same origin. Use `.env.development` with `VITE_API_BASE=http://localhost:3000` for frontend-only dev mode.

### Running both separately (dev)
```bash
# Terminal 1
cd backend && cargo run
# Terminal 2
cd frontend && npm run dev
```

## Architecture

### Storage layout

Storage defaults to `../storage` relative to the `backend/` working directory. It is configurable via `--storage <path>` CLI flag or the `storage` key in `aura_notes.toml` (CLI takes precedence). The `aura_notes.toml` location is unaffected by `--storage`. Port `3000` is still hardcoded in `src/main.rs`.

```
storage/
  notes/      # .md files (one per note, filename = note name, sub-dirs allowed)
  assets/     # uploaded images
  drawings/   # Excalidraw JSON files (.excalidraw)
```

### Backend API endpoints

All endpoints except `POST /auth/login` require `Authorization: Bearer <token>` (session token or API key).

- `GET /notes` — list all notes (recursive)
- `GET /notes/{*name}` — read note (returns `{name, content, frontmatter}`)
- `PUT /notes/{*name}` — create or overwrite note (auto-stamps `lastModified` if frontmatter present)
- `PATCH /notes/{*name}` — rename note (`{new_name}`)
- `DELETE /notes/{*name}` — delete note
- `GET /backlinks/{*name}` — notes linking to this note via `[[WikiLinks]]`
- `POST /assets` — upload image (multipart), returns `{url}`
- `GET /api/assets` — list asset filenames
- `GET /assets/{*filename}` — serve asset file
- `DELETE /assets/{*filename}` — delete asset
- `GET /api/drawings` — list drawing names
- `GET /api/drawings/{*name}` — get drawing JSON (Excalidraw format)
- `PUT /api/drawings/{*name}` — save drawing JSON
- `DELETE /api/drawings/{*name}` — delete drawing + SVG preview
- `GET /api/drawing-preview/{*name}` — get SVG preview of drawing
- `PUT /api/drawing-preview/{*name}` — save SVG preview
- `GET /api/search?q=` — full-text search, returns snippets
- `GET /api/query?q=` — structured DSL query on frontmatter metadata
- `GET /api/tags` — list all distinct tags across notes
- `GET /api/aliases` — list all distinct aliases across notes
- `GET /api/field-values?field=` — distinct values for a given frontmatter field
- `GET /api/openapi.json` — OpenAPI 3.0 spec (derives base URL from request headers)
- `POST /auth/login` — **public** — `{password}` → `{token}` (Argon2 verification)
- `POST /auth/logout` — invalidate session token
- `GET /api/key` — return configured API key (masked display in Settings)

At startup the backend fully indexes all `.md` files into a `HashMap<String, NoteRow>` (blocking until complete). On every PUT/PATCH/DELETE the in-memory index is updated immediately and the backlink index is rebuilt via a `walkdir` scan — not incremental.

Search and query use linear scans over the in-memory index. `api/search` matches on title+body; `api/query` runs a DSL of boolean AND/OR/NOT predicates. `NOT field:x` correctly includes notes where the field is absent (Rust `Option` semantics, no NULL edge cases).

### Query DSL (`api/query?q=`)

Parsed entirely in `src/db.rs::parse_query`. Tokens are whitespace-separated.

**Filters** — combined with implicit AND, `OR`, `NOT`:
- Bare word → `Pred::Text` — substring match on title **and** name (both checked; unlike `title:` which is title-only)
- `#tag` / `tag:val` → prefix match on any tag
- `title:val` → substring on resolved title (frontmatter → H1 → last path segment)
- `path:val` → substring on full file path (`Boulot/Scality/Ring/Note`)
- `name:val` → substring on last path segment only (`Note`)
- `status:val`, `type:val` → exact match
- `area:val`, `author:val`, `url:val`, `project:val` → substring
- `date:prefix`, `due:prefix`, `lastModified:prefix` → starts-with (ISO format)
- `rating:n` → exact integer match
- `alias:val` → exact match on aliases list
- `pinned:true/false` → boolean
- `locked:true/false` → boolean (note is read-only when `true`)
- `priority:val` → exact match (`high`, `medium`, `low`); `order by priority` uses semantic order (high → medium → low)

**Limiters** (applied before `order by`):
- `recent:n` → keep n most recently modified (by filesystem mtime)
- `oldest:n` → keep n least recently modified (by filesystem mtime)

**Sorting** (applied after limiters):
- `order by <field> [asc|desc]` — fields: `name`, `title`, `date`, `modified`, `due`, `status`, `rating`, `area`, `author`, `priority`, `project`, `lastModified`. Default `asc`. Notes missing the field sort last.
- Default sort (no limiter, no order by): alphabetical by name.

**Display** (frontend-only, stripped before API call):
- `print <fields>` — controls which columns render in the query block. Always placed last in the query string.
- Valid print fields: `name` (last path segment only), `path` (full path), `title`, `tags`, `date`, `status`, `area`, `author`, `due`, `rating`, `url`
- `name` vs `path`: `print name` shows only the filename (`Note`); `print path` shows the full path (`Dev/Axum/Note`)

**NOT semantics:** `NOT` predicates are **global exclusions** — applied after OR-group evaluation. `A OR B AND NOT C` means `(A OR B) AND NOT C`, not `A OR (B AND NOT C)`. This ensures `NOT` always excludes from all results, regardless of where it appears in the query.

**`date` vs `modified` vs `lastModified`**: `date:` filters on the user-written frontmatter date. `recent:`/`oldest:`/`order by modified` use filesystem mtime. `lastModified:` filters on the auto-maintained frontmatter field updated on every save — three independent concepts.

### Automatic `lastModified` stamping

On every `PUT /notes/{name}`, `src/frontmatter.rs::stamp_last_modified()` is called before writing to disk. If the note has a YAML frontmatter block, it updates (or inserts) `lastModified: YYYY-MM-DD` with today's date. Notes without frontmatter are untouched.

### Index pages (`type: index`)

Notes with `type: index` in their frontmatter are treated as dashboard pages:
- **Backend:** `NoteMeta.is_index` is set to `true` in `GET /notes`
- **Sidebar:** shown in a dedicated section at the top (above the tree), with a grid icon (⊞)
- **Editor:** H1 is centered + larger; query blocks render in a responsive CSS grid (`repeat(auto-fill, minmax(min(100%, 380px), 1fr))`) — 1 column on mobile, 2 on tablet, 3+ on wide screens. Non-query-block elements span full width via `grid-column: 1 / -1`.
- **Command palette:** grid icon instead of the document icon

### Note locking (`locked: true`)

Setting `locked: true` in a note's frontmatter makes it read-only:
- **Frontend:** padlock icon in the title bar (discrete when unlocked, orange when locked). Clicking toggles `locked: true/false` in the frontmatter and saves. TipTap switches to `editable(false)` via `$effect` reacting to the `isLocked` prop.
- **Backend:** `NoteRow.locked` is indexed from frontmatter; queryable via `locked:true/false` in the DSL (`Pred::Locked`).
- **Enforcement:** frontend-only — the backend does not reject PUT/DELETE on locked notes. Locking is a UX guard, not a security boundary.

### Backend source files
- `src/main.rs` — Axum router, app state, startup indexing, `--hash-password` flag
- `src/config.rs` — `aura_notes.toml` loading and validation
- `src/auth.rs` — auth middleware (`from_fn_with_state`), `login`/`logout` handlers, `hash_password`
- `src/session.rs` — in-memory session store (`RwLock<HashMap<token, expiry>>`, TTL 30 days)
- `src/key.rs` — `GET /api/key` handler, `random_hex_key()` utility
- `src/notes.rs` — CRUD handlers, asset upload/serve/delete, rename; calls `stamp_last_modified` on PUT
- `src/backlinks.rs` — WikiLink extraction, backlink index, handler
- `src/db.rs` — in-memory note index (`RwLock<HashMap>`) with DSL query engine
- `src/seed.rs` — seeds `Home.md` and `Help.md` into `storage/notes/` on first launch (files embedded via `include_str!` from `src/defaults/`)
- `src/frontmatter.rs` — YAML frontmatter parsing + `stamp_last_modified()`
- `src/query.rs` — HTTP handlers for search/tags/aliases/field-values; calls into `db.rs` for actual querying
- `src/openapi.rs` — `GET /api/openapi.json` handler, full OpenAPI 3.0 spec
- `src/drawings.rs` — Excalidraw drawing CRUD + SVG preview handlers
- `src/frontend.rs` — embedded static asset serving via `rust-embed`; falls back to `index.html` for SPA routing

### Frontend structure
- `src/routes/+page.svelte` — main layout; derives `isIndex` from frontmatter and passes to Editor
- `src/lib/api.ts` — typed fetch wrappers; session token from localStorage injected as Bearer; global 401 handler dispatches `auth:expired`
- `src/lib/LoginPage.svelte` — password login form; calls `POST /auth/login`, stores token
- `src/lib/Editor.svelte` — TipTap instance, NodeViews, table toolbar; accepts `isIndex` prop for dashboard layout
- `src/lib/Sidebar.svelte` — note tree, mobile drawer; index pages shown in dedicated top section
- `src/lib/CommandPalette.svelte` — Ctrl+K palette (search, commands, export, themes); grid icon for index pages
- `src/lib/Backlinks.svelte` — backlinks panel
- `src/lib/TableOfContents.svelte` — collapsible ToC bar (disable via `toc: false` frontmatter)
- `src/lib/FrontmatterEditor.svelte` — inline frontmatter (tags, status, date)
- `src/lib/slashCommands.ts` — `/` slash-command menu; `/?` shortcut inserts a dynamic query block
- `src/lib/wikiLink.ts` + `wikiLinkSuggestion.ts` — `[[WikiLink]]` input rule + autocomplete
- `src/lib/queryBlock.ts` — dynamic query block NodeView (rendered from DSL)
- `src/lib/drawingBlock.ts` — TipTap NodeView for embedded Excalidraw drawings (lazy-loads React + Excalidraw)
- `src/lib/taskExtensions.ts` — checkbox task list extension
- `src/lib/theme.ts` — theme switching (Default / GitHub Dark), persisted in localStorage
- `src/lib/settings.ts` — `AppSettings` type (font, size, lineHeight, custom CSS), localStorage persistence under `aura-settings`
- `src/lib/Settings.svelte` — settings panel UI (font, size, line height, custom CSS, Security section, Sign out)
- `src/lib/MetaPage.svelte` — media library modal: lists and deletes uploaded assets and Excalidraw drawings

### Frontend notes
- **Svelte 5 runes mode** is forced in `svelte.config.js` — use `$state`, `$derived`, `$effect` everywhere, not the legacy store API.
- **SPA mode:** `src/routes/+layout.ts` sets `export const ssr = false`; the app uses `@sveltejs/adapter-static` with SPA fallback — all routing is client-side.
- **Tailwind 4** is configured entirely via the `@tailwindcss/vite` plugin — there is no `tailwind.config.js`. All theme tokens and CSS variables are in `src/app.css`.
- **`tiptap-markdown`** is the library used for bidirectional Markdown ↔ TipTap/ProseMirror document serialization.
- Theme switching sets `data-theme="<id>"` on `<html>`; `default` removes the attribute entirely. IDs: `rose-pine`, `dracula`, `solarized`, `catppuccin`, `github-dark`. Persisted under localStorage key `maunotes-theme` (see `src/lib/theme.ts`).
- Both the `/` slash-command menu and the `[[WikiLink]]` autocomplete use the same `@tiptap/suggestion` plugin — each must receive a **unique `pluginKey`** or they will conflict.

### Key TipTap extensions
- `StarterKit` (codeBlock disabled — replaced by CodeBlockLowlight)
- `CodeBlockLowlight` with custom NodeView: clickable language label (top-left), copy button (top-right on hover), syntax highlighting via lowlight/highlight.js
- `Suggestion` powering the `/` slash-command menu
- Custom WikiLink input rule + suggestion popup
- Custom `queryBlock` node for dynamic note queries
- Custom `DrawingBlock` node for embedded Excalidraw drawings (lazy-loads React + Excalidraw at render time)
- Task list (checkbox) extension
- Table extension with floating toolbar (add/delete rows and columns)
- `ExitBlockquote` extension — `Ctrl+Enter` exits a blockquote by inserting a paragraph after it

### Scripts
- `scripts/an` — CLI client (bash + jq); set `AN_KEY` env var. Commands: `ls`, `get <name>`, `put <name>`, `rm <name>`, `query <q>`, `search <q>`, `key`
- `scripts/migrate-frontmatter` — Python script to rewrite legacy frontmatter field names in `.md` files (`displayName`→`title`, `lastModified`→`date`, `scheduled`→`due`, `customer`→`project`, `draft:true`→`status:draft`). Supports `--dry-run` and `--dir`.

## Authentication

All auth configuration lives in `aura_notes.toml` (at the repo root by default, or via `--config <path>` / `AURA_NOTES_CONFIG` env var).

```toml
# Hash with: ./server --hash-password "yourpassword"
password = "$argon2id$v=19$..."

# Optional — for CLI/REST/OpenAPI programmatic access
# api_key = ""   # openssl rand -hex 32
```

**Web UI:** password login → session token stored in localStorage (TTL 30 days). Token sent as `Authorization: Bearer` on every request. Any 401 clears the session and shows the login page.

**CLI (`scripts/an`):** set `AN_KEY` (or `AURA_NOTES_KEY`) and optionally `AN_URL` (default `http://localhost:3000`). Available commands: `ls`, `get <name>`, `put <name>`, `rm <name>`, `query <q>`, `search <q>`, `key`.

**Future:** architecture is ready for OIDC (Authelia, etc.) — the auth middleware is strategy-agnostic; adding OIDC = new `/auth/callback` route that creates sessions the same way.

## UI/UX Constraints

- Default font: Inter, `1.1rem`, line-height `1.7`, no visible editor borders — all three are user-configurable via Settings panel (persisted in localStorage)
- Light: `#f8f6f1` bg / `#2c2825` text — Dark: `#1c1c1c` bg / `#e1dcd4` text (warm palette)
- Six themes: Default (no attr), Rosé Pine, Dracula, Solarized, Catppuccin, GitHub Dark — set via `data-theme` on `<html>`
- Focus mode: sidebar fades when actively typing
- All saves and searches must be async (never block the UI)
- Ctrl+K opens the global command palette
- Mobile: sidebar is a CSS drawer, command palette becomes a bottom sheet
- `type: index` notes render in dashboard mode: centered H1, query blocks in 2-column grid
