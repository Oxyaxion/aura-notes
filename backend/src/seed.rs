use std::path::Path;

const HOME_MD:        &str = include_str!("defaults/Home.md");
const HELP_MD:        &str = include_str!("defaults/Help.md");
const RISOTTO_MD:     &str = include_str!("defaults/Recipes/Mushroom Risotto.md");
const CARBONARA_MD:   &str = include_str!("defaults/Recipes/Pasta Carbonara.md");
const LINUX_MD:       &str = include_str!("defaults/Dev/Linux Commands.md");
const GIT_MD:         &str = include_str!("defaults/Dev/Git Cheatsheet.md");
const PRAGPROG_MD:    &str = include_str!("defaults/Books/The Pragmatic Programmer.md");
const ATOMIC_MD:      &str = include_str!("defaults/Books/Atomic Habits.md");
const BLOG_MD:        &str = include_str!("defaults/Projects/Blog Redesign.md");
const LISBON_MD:      &str = include_str!("defaults/Travel/Lisbon 2026.md");
const MAY_MD:         &str = include_str!("defaults/Journal/May 2026.md");

pub async fn seed_defaults(notes_dir: &Path) {
    seed_file(notes_dir, "Home.md", HOME_MD).await;
    seed_file(notes_dir, "Help.md", HELP_MD).await;
    seed_file(notes_dir, "Recipes/Mushroom Risotto.md", RISOTTO_MD).await;
    seed_file(notes_dir, "Recipes/Pasta Carbonara.md", CARBONARA_MD).await;
    seed_file(notes_dir, "Dev/Linux Commands.md", LINUX_MD).await;
    seed_file(notes_dir, "Dev/Git Cheatsheet.md", GIT_MD).await;
    seed_file(notes_dir, "Books/The Pragmatic Programmer.md", PRAGPROG_MD).await;
    seed_file(notes_dir, "Books/Atomic Habits.md", ATOMIC_MD).await;
    seed_file(notes_dir, "Projects/Blog Redesign.md", BLOG_MD).await;
    seed_file(notes_dir, "Travel/Lisbon 2026.md", LISBON_MD).await;
    seed_file(notes_dir, "Journal/May 2026.md", MAY_MD).await;
}

async fn seed_file(notes_dir: &Path, filename: &str, content: &str) {
    let path = notes_dir.join(filename);
    if path.exists() {
        return;
    }
    if let Some(parent) = path.parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            tracing::warn!("Failed to create dir for {filename}: {e}");
            return;
        }
    }
    if let Err(e) = tokio::fs::write(&path, content).await {
        tracing::warn!("Failed to seed {filename}: {e}");
    }
}
