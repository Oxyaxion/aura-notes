use serde::Deserialize;

const TEMPLATE: &str = r#"# Clef Note — configuration

# Password for the web UI login.
# Hash with: ./clef-note --hash-password "yourpassword"
password = ""

# Storage directory — optional. Defaults to ../storage (relative to the backend/).
# Can be overridden at runtime with: ./clef-note --storage /mnt/notes
# storage = "/mnt/notes"

# Port — optional. Defaults to 3000.
# Can be overridden at runtime with: ./clef-note --port 8080
# port = 3000

# API key — optional. For programmatic access (CLI, REST, OpenAI tools…).
# Generate with: openssl rand -hex 32
# api_key = ""
"#;

#[derive(Deserialize)]
pub struct Config {
    pub password: String,
    pub storage: Option<String>,
    pub port: Option<u16>,
    pub api_key: Option<String>,
}

pub fn resolve_path(storage_path: &std::path::Path) -> std::path::PathBuf {
    let args: Vec<String> = std::env::args().collect();
    for (i, arg) in args.iter().enumerate() {
        if let Some(val) = arg.strip_prefix("--config=") {
            return std::path::PathBuf::from(val);
        }
        if arg == "--config" && let Some(p) = args.get(i + 1) {
            return std::path::PathBuf::from(p);
        }
    }
    if let Ok(p) = std::env::var("AURA_NOTES_CONFIG") && !p.is_empty() {
        return std::path::PathBuf::from(p);
    }
    storage_path.parent().unwrap_or(storage_path).join("clef-note.toml")
}

pub fn load(storage_path: &std::path::Path) -> Config {
    let path = resolve_path(storage_path);

    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => {
            std::fs::write(&path, TEMPLATE).ok();
            eprintln!("error: clef-note.toml not found — a template has been created at {}", path.display());
            eprintln!("       Set password (./clef-note --hash-password \"yourpassword\") and restart.");
            std::process::exit(1);
        }
    };

    let cfg: Config = match toml::from_str(&raw) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: failed to parse clef-note.toml: {e}");
            std::process::exit(1);
        }
    };

    if cfg.password.trim().is_empty() {
        eprintln!("error: password is not set in clef-note.toml");
        eprintln!("       Hash one with: ./clef-note --hash-password \"yourpassword\"");
        std::process::exit(1);
    }

    if !cfg.password.starts_with("$argon2") {
        eprintln!("error: password in clef-note.toml must be an Argon2 hash");
        eprintln!("       Generate with: ./clef-note --hash-password \"yourpassword\"");
        std::process::exit(1);
    }

    cfg
}
