use std::path::Path;
use std::process::Command;

use crate::config::CookestConfig;

pub fn compose_command(instance_dir: &Path) -> Command {
    let mut cmd = Command::new("docker");
    cmd.args(["compose", "-f"])
        .arg(instance_dir.join("docker-compose.yml"))
        .arg("--project-directory")
        .arg(instance_dir);
    cmd
}

pub fn compose_up(instance_dir: &Path, detach: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = compose_command(instance_dir);
    cmd.arg("up");
    if detach {
        cmd.arg("-d");
    }
    cmd.arg("--build");
    let status = cmd.status()?;
    if !status.success() {
        return Err("docker compose up failed".into());
    }
    Ok(())
}
