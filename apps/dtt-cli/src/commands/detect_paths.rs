use anyhow::Result;
use dtt_application::{DetectEnvironmentRequest, detect_environment};
use dtt_i18n::{CliMessage as M, Translator};
pub fn run(t: &Translator) -> Result<()> {
    let env = detect_environment(&DetectEnvironmentRequest::default());
    for (label, path) in [
        (M::GameRoot, env.game_root),
        (M::Documents, env.documents_dir),
        (M::Launcher, env.launcher_db),
    ] {
        println!(
            "{}: {}",
            t.cli(label)?,
            path.map(|p| p.display().to_string())
                .unwrap_or(t.cli(M::NotDetected)?)
        );
    }
    if !env.steam_libraries.is_empty() {
        println!("{}:", t.cli(M::Libraries)?);
    }
    for path in env.steam_libraries {
        println!("  {}", path.display());
    }
    Ok(())
}
