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
    Ok(String::from_utf8(output.stdout)?)
}

pub fn backup_database(
    instance_dir: &Path,
    container_name: &str,
    db_name: &str,
    db_password: &str,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("docker")
        .args(["exec", "-e"])
        .arg(format!("PGPASSWORD={db_password}"))
        .arg(container_name)
        .args(["pg_dump", "-U", "postgres", "-Fc", db_name])
        .stdout(std::fs::File::create(output_path)?)
        .status()?;
    if !status.success() {
        return Err(format!("backup of {db_name} failed").into());
    }
    Ok(())
}

pub fn restore_database(
    instance_dir: &Path,
    container_name: &str,
    db_name: &str,
    db_password: &str,
    input_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("docker")
        .args(["exec", "-i", "-e"])
        .arg(format!("PGPASSWORD={db_password}"))
        .arg(container_name)
        .args(["pg_restore", "-U", "postgres", "-d", db_name, "--clean", "--if-exists"])
        .stdin(std::fs::File::open(input_path)?)
        .status()?;
    if !status.success() {
        return Err(format!("restore of {db_name} failed").into());
    }
    Ok(())
}

/// Check if Docker and Docker Compose are available.
pub fn check_prerequisites() -> Result<(), Box<dyn std::error::Error>> {
    if which::which("docker").is_err() {
        return Err("Docker is not installed. Install it from https://docker.com".into());
    }

    let output = Command::new("docker")
        .args(["compose", "version"])
        .output();
    match output {
        Ok(o) if o.status.success() => {}
        _ => {
            return Err(
                "Docker Compose v2 is required. Update Docker or install the compose plugin."
                    .into(),
            );
        }
    }

    Ok(())
}
