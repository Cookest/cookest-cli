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

pub fn compose_down(instance_dir: &Path, volumes: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = compose_command(instance_dir);
    cmd.arg("down");
    if volumes {
        cmd.arg("-v");
    }
    let status = cmd.status()?;
    if !status.success() {
        return Err("docker compose down failed".into());
    }
    Ok(())
}

pub fn compose_pull(instance_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = compose_command(instance_dir);
    cmd.arg("pull");
    let status = cmd.status()?;
    if !status.success() {
        return Err("docker compose pull failed".into());
    }
    Ok(())
}

pub fn compose_logs(
    instance_dir: &Path,
    service: Option<&str>,
    follow: bool,
    tail: Option<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = compose_command(instance_dir);
    cmd.arg("logs");
    if follow {
        cmd.arg("-f");
    }
    if let Some(n) = tail {
        cmd.arg("--tail").arg(n.to_string());
    }
    if let Some(svc) = service {
        cmd.arg(svc);
    }
    let status = cmd.status()?;
    if !status.success() {
        return Err("docker compose logs failed".into());
    }
    Ok(())
}

pub fn compose_ps(instance_dir: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut cmd = compose_command(instance_dir);
    cmd.args(["ps", "--format", "json"]);
    let output = cmd.output()?;
    if !output.status.success() {
        return Err("docker compose ps failed".into());
    }