use clap::Args;
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;

const BACKEND_REPO: &str = "https://github.com/cookest/cookest-backend";
const ADMIN_REPO: &str = "https://github.com/cookest/cookest-admin-panel";

#[derive(Args)]
pub struct BuildArgs {
    /// Use a local clone of the source repos instead of cloning from GitHub.
    /// Expects subdirectories: cookest-backend/ and cookest-admin-panel/
    #[arg(long, value_name = "DIR")]
    pub local: Option<PathBuf>,

    /// Image tag to apply to built images (default: local)
    #[arg(long, default_value = "local")]
    pub tag: String,
}

pub fn run(args: BuildArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "🍳 Building Cookest images from source".green().bold());
    println!("{}", "━".repeat(40).dimmed());

    let work_dir = match &args.local {
        Some(p) => {
            let p = std::fs::canonicalize(p)?;
            println!("  Using local source at {}", p.display().to_string().cyan());
            p
        }
        None => {
            let tmp = std::env::temp_dir().join("cookest-build");
            std::fs::create_dir_all(&tmp)?;
            clone_repos(&tmp)?;
            tmp
        }
    };

    println!();
    build_image(
        &work_dir.join("cookest-backend"),
        "crates/app-api/Dockerfile",
        &format!("cookest/app-api:{}", args.tag),
        "app-api",
    )?;

    build_image(
        &work_dir.join("cookest-backend"),
        "crates/food-api/Dockerfile",
        &format!("cookest/food-api:{}", args.tag),
        "food-api",
    )?;

    build_image(
        &work_dir.join("cookest-admin-panel"),
        "Dockerfile",
        &format!("cookest/admin:{}", args.tag),
        "admin",
    )?;

    println!();
    println!("{}", "✓ All images built successfully!".green().bold());
    println!();
    println!("{}", "Images tagged:".bold());
    println!("  {}", format!("cookest/app-api:{}", args.tag).cyan());
    println!("  {}", format!("cookest/food-api:{}", args.tag).cyan());
    println!("  {}", format!("cookest/admin:{}", args.tag).cyan());

    if args.tag == "local" {
        println!();
        println!(
            "Run {} then {} to start with your local images.",
            "cookest init --from-source".cyan(),
            "cookest up".cyan()
        );
        println!(
            "Or set {} in an existing {} to switch an instance.",
            "[images]\nsource = \"local\"".cyan(),
            "cookest.toml".cyan()
        );
    }

    Ok(())
}

fn clone_repos(dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for (repo, dir) in [
        (BACKEND_REPO, "cookest-backend"),
        (ADMIN_REPO, "cookest-admin-panel"),
    ] {
        let target = dest.join(dir);
        if target.exists() {
            println!("  {} {} (already cloned — pulling latest)", "↻".cyan(), dir);
            let status = Command::new("git")
                .args(["-C", target.to_str().unwrap(), "pull", "--ff-only"])
                .status()?;
            if !status.success() {
                return Err(format!("git pull failed for {dir}").into());
            }
        } else {
            println!("  {} {} from {}", "↓".cyan(), dir, repo);
            let status = Command::new("git")
                .args(["clone", "--depth=1", repo, target.to_str().unwrap()])
                .status()?;
            if !status.success() {
                return Err(format!("git clone failed for {repo}").into());
            }
        }
        println!("  {} {}", "✓".green(), dir);
    }
    Ok(())
}

fn build_image(
    context: &Path,
    dockerfile: &str,
    tag: &str,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  Building {}...", label.bold());
    let status = Command::new("docker")
        .args([
            "build",
            "-t",
            tag,
            "-f",
            dockerfile,
            context.to_str().unwrap_or("."),
        ])
        .current_dir(context)
        .status()?;

    if !status.success() {
        return Err(format!("docker build failed for {label}").into());
    }
    println!("  {} {} → {}", "✓".green(), label, tag.cyan());
    Ok(())
}
