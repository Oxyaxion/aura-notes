use axum::{Json, extract::Request};
use serde_json::{Value, json};

pub async fn get_spec(req: Request) -> Json<Value> {
    let host = req.headers()
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:3000")
        .to_string();

    let scheme = req.headers()
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or(if host.starts_with("localhost") || host.starts_with("127.") { "http" } else { "https" });

    let base = format!("{scheme}://{host}");

    Json(json!({
        "openapi": "3.0.3",
        "info": {
            "title": "Clef Note",
            "description": "Self-hosted Zettelkasten note-taking API. All endpoints require `Authorization: Bearer <api_key>`.",
            "version": "1.0.0"
        },
        "servers": [{ "url": base }],
        "security": [{ "bearer": [] }],
        "components": {
            "securitySchemes": {
                "bearer": {
                    "type": "http",
                    "scheme": "bearer",
                    "description": "API key configured in clef-note.toml (api_key field)"
                }
            },
            "schemas": {
                "NoteMeta": {
                    "type": "object",
                    "properties": {
                        "name":        { "type": "string", "example": "Dev/Rust - lifetimes" },
                        "pinned":      { "type": "boolean" },
                        "is_template": { "type": "boolean" }
                    },
                    "required": ["name"]
                },
                "NoteContent": {
                    "type": "object",
                    "properties": {
                        "name":        { "type": "string" },
                        "content":     { "type": "string", "description": "Markdown body without YAML frontmatter" },
                        "frontmatter": { "type": "object", "additionalProperties": true, "description": "Parsed YAML frontmatter fields" }
                    },
                    "required": ["name", "content", "frontmatter"]
                },
                "SearchResult": {
                    "type": "object",
                    "properties": {
                        "name":    { "type": "string" },
                        "title":   { "type": "string" },
                        "snippet": { "type": "string", "description": "Contextual excerpt around the match" }
                    },
                    "required": ["name", "snippet"]
                },
                "QueryResult": {
                    "type": "object",
                    "properties": {
                        "name":      { "type": "string" },
                        "title":     { "type": "string" },
                        "tags":      { "type": "array", "items": { "type": "string" } },
                        "status":    { "type": "string" },
                        "date":      { "type": "string" },
                        "area":      { "type": "string" },
                        "author":    { "type": "string" },
                        "due":       { "type": "string" },
                        "rating":    { "type": "integer" },
                        "pinned":    { "type": "boolean" },
                        "note_type": { "type": "string" },
                        "priority": { "type": "string", "enum": ["high", "medium", "low"] },
                        "project":  { "type": "string" }
                    },
                    "required": ["name", "tags"]
                },
                "TagCount": {
                    "type": "object",
                    "properties": {
                        "tag":   { "type": "string" },
                        "count": { "type": "integer" }
                    },
                    "required": ["tag", "count"]
                }
            }
        },
        "paths": {
            "/notes": {
                "get": {
                    "summary": "List all notes",
                    "operationId": "listNotes",
                    "responses": {
                        "200": {
                            "description": "Array of note metadata",
                            "content": { "application/json": { "schema": {
                                "type": "array", "items": { "$ref": "#/components/schemas/NoteMeta" }
                            }}}
                        }
                    }
                }
            },
            "/notes/{name}": {
                "parameters": [{
                    "name": "name", "in": "path", "required": true,
                    "schema": { "type": "string" },
                    "description": "Note name or path, e.g. `Dev/Rust - lifetimes`. URL-encode each segment."
                }],
                "get": {
                    "summary": "Read a note",
                    "operationId": "getNote",
                    "responses": {
                        "200": { "description": "Note content and frontmatter", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/NoteContent" } } } },
                        "404": { "description": "Note not found" }
                    }
                },
                "put": {
                    "summary": "Create or overwrite a note",
                    "description": "Pass the full Markdown content. To include frontmatter, start with `---\\ntitle: ...\\ntags:\\n  - foo\\n---\\n\\n`.",
                    "operationId": "putNote",
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "properties": { "content": { "type": "string" } },
                            "required": ["content"]
                        }}}
                    },
                    "responses": {
                        "204": { "description": "Saved" }
                    }
                },
                "patch": {
                    "summary": "Rename a note",
                    "operationId": "renameNote",
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "properties": { "new_name": { "type": "string" } },
                            "required": ["new_name"]
                        }}}
                    },
                    "responses": {
                        "204": { "description": "Renamed" },
                        "409": { "description": "A note with that name already exists" }
                    }
                },
                "delete": {
                    "summary": "Delete a note",
                    "operationId": "deleteNote",
                    "responses": {
                        "204": { "description": "Deleted" },
                        "404": { "description": "Note not found" }
                    }
                }
            },
            "/backlinks/{name}": {
                "get": {
                    "summary": "Get notes that link to this note via [[WikiLinks]]",
                    "operationId": "getBacklinks",
                    "parameters": [{
                        "name": "name", "in": "path", "required": true, "schema": { "type": "string" }
                    }],
                    "responses": {
                        "200": {
                            "description": "Backlink list",
                            "content": { "application/json": { "schema": {
                                "type": "object",
                                "properties": {
                                    "note":      { "type": "string" },
                                    "backlinks": { "type": "array", "items": { "type": "string" } }
                                }
                            }}}
                        }
                    }
                }
            },
            "/api/search": {
                "get": {
                    "summary": "Full-text search across all notes",
                    "operationId": "searchNotes",
                    "parameters": [{
                        "name": "q", "in": "query", "required": true,
                        "schema": { "type": "string" }, "description": "Free-text query"
                    }],
                    "responses": {
                        "200": {
                            "description": "Up to 20 results with contextual snippets",
                            "content": { "application/json": { "schema": {
                                "type": "array", "items": { "$ref": "#/components/schemas/SearchResult" }
                            }}}
                        }
                    }
                }
            },
            "/api/query": {
                "get": {
                    "summary": "Structured DSL query on note metadata",
                    "description": "Filter notes by frontmatter fields.\n\n**Filters** (combine with AND / OR / NOT):\n- `#tag` or `tag:val` — tag prefix match\n- `title:val` — substring on title\n- `status:val` — exact match (e.g. `active`, `draft`)\n- `area:val`, `author:val` — substring\n- `date:prefix` — starts-with, e.g. `date:2025-04`\n- `due:prefix` — same for due date\n- `rating:n` — exact integer\n- `pinned:true`\n\n**Limiters**: `recent:n`, `oldest:n`\n\n**Sorting**: `order by <field> [asc|desc]` — fields: `name`, `title`, `date`, `modified`, `rating`, `status`, `area`\n\n**Examples**: `#rust status:active`, `recent:10`, `area:pro order by date desc`, `NOT tag:archive`",
                    "operationId": "queryNotes",
                    "parameters": [{
                        "name": "q", "in": "query", "required": true,
                        "schema": { "type": "string" },
                        "example": "#rust status:active recent:10"
                    }],
                    "responses": {
                        "200": {
                            "description": "Matching notes",
                            "content": { "application/json": { "schema": {
                                "type": "array", "items": { "$ref": "#/components/schemas/QueryResult" }
                            }}}
                        }
                    }
                }
            },
            "/api/tags": {
                "get": {
                    "summary": "List all distinct tags with their note count",
                    "operationId": "listTags",
                    "responses": {
                        "200": {
                            "description": "Tag list sorted by count",
                            "content": { "application/json": { "schema": {
                                "type": "array", "items": { "$ref": "#/components/schemas/TagCount" }
                            }}}
                        }
                    }
                }
            },
            "/api/aliases": {
                "get": {
                    "summary": "List all note aliases",
                    "description": "Returns a map of alias → note name.",
                    "operationId": "listAliases",
                    "responses": {
                        "200": {
                            "description": "Alias map",
                            "content": { "application/json": { "schema": {
                                "type": "object", "additionalProperties": { "type": "string" }
                            }}}
                        }
                    }
                }
            },
            "/api/field-values": {
                "get": {
                    "summary": "List distinct values for a given frontmatter field",
                    "operationId": "fieldValues",
                    "parameters": [{
                        "name": "field", "in": "query", "required": true,
                        "schema": { "type": "string" },
                        "description": "Frontmatter field name, e.g. `status`, `area`, `author`"
                    }],
                    "responses": {
                        "200": {
                            "description": "Distinct values",
                            "content": { "application/json": { "schema": {
                                "type": "array", "items": { "type": "string" }
                            }}}
                        }
                    }
                }
            }
        }
    }))
}
