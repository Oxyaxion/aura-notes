use serde::Deserialize;

const TEMPLATE: &str = r#"# Aura Notes — configuration

# Password for the web UI login.
# Hash with: ./aura-notes --hash-password "yourpassword"
password = ""

# Storage directory — optional. Defaults to ../storage (relative to the backend/).
# Can be overridden at runtime with: ./aura-notes --storage /mnt/notes
# storage = "/mnt/notes"

# Port — optional. Defaults to 3000.
# Can be overridden at runtime with: ./aura-notes --port 8080
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
    storage_path.parent().unwrap_or(storage_path).join("aura_notes.toml")
}

pub fn load(storage_path: &std::path::Path) -> Config {
    let path = resolve_path(storage_path);

    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => {
            std::fs::write(&path, TEMPLATE).ok();
            eprintln!("error: aura_notes.toml not found — a template has been created at {}", path.display());
            eprintln!("       Set password (./aura-notes --hash-password \"yourpassword\") and restart.");
            std::process::exit(1);
        }
    };

    let cfg: Config = match toml::from_str(&raw) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: failed to parse aura_notes.toml: {e}");
            std::process::exit(1);
        }
    };

    if cfg.password.trim().is_empty() {
        eprintln!("error: password is not set in aura_notes.toml");
        eprintln!("       Hash one with: ./aura-notes --hash-password \"yourpassword\"");
        std::process::exit(1);
    }

    if !cfg.password.starts_with("$argon2") {
        eprintln!("error: password in aura_notes.toml must be an Argon2 hash");
        eprintln!("       Generate with: ./aura-notes --hash-password \"yourpassword\"");
        std::process::exit(1);
    }

    cfg
}
