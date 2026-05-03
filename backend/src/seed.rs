use std::path::Path;

const HOME_MD: &str = include_str!("defaults/Home.md");
const HELP_MD: &str = include_str!("defaults/Help.md");

pub async fn seed_defaults(notes_dir: &Path) {
    seed_file(notes_dir, "Home.md", HOME_MD).await;
    seed_file(notes_dir, "Help.md", HELP_MD).await;
}

async fn seed_file(notes_dir: &Path, filename: &str, content: &str) {
    let path = notes_dir.join(filename);
    if !path.exists() {
        if let Err(e) = tokio::fs::write(&path, content).await {
            tracing::warn!("Failed to seed {filename}: {e}");
        }
    }
}
