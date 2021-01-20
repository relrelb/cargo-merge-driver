use clap::Clap;
use std::fs;
use std::fs::File;
use std::io::{Result, Write};
use std::path::PathBuf;
use std::process::{exit, Command};

#[derive(Clap)]
enum Subcommand {
    Install(Install),
    Uninstall(Install),
    Merge,
}

#[derive(Clap)]
struct Install {
    #[clap(short, long)]
    global: bool,

    #[clap(long, default_value = "cargo-merge-driver")]
    name: String,
}

#[derive(Clap)]
struct Opts {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

fn git(args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .output()
        .expect("Failed to execute git");
    String::from_utf8(output.stdout).unwrap()
}

fn install_gitattributes(opts: Install, install: bool) -> Result<()> {
    let path = if opts.global {
        let path = git(&["config", "--global", "core.attributesFile"]);
        if path.is_empty() {
            dirs::home_dir().unwrap().join(".gitattributes")
        } else {
            PathBuf::from(path)
        }
    } else {
        let git_dir = git(&["rev-parse", "--git-dir"]);
        PathBuf::from(git_dir).join("info").join("attributes")
    };
    let text = fs::read_to_string(&path)?;
    let mut file = File::create(path)?;
    for line in text.lines() {
        // TODO: Handle spaces.
        if !line.starts_with("Cargo.lock merge=") {
            writeln!(file, "{}", line)?;
        }
    }
    if install {
        writeln!(file, "Cargo.lock merge={}", opts.name)?;
    }
    Ok(())
}

fn install(opts: Install) -> Result<()> {
    let global = if opts.global { "--global" } else { "--local" };
    git(&[
        "config",
        global,
        &format!("merge.{}.name", opts.name),
        "Automatically merge Cargo.lock files",
    ]);
    git(&[
        "config",
        global,
        &format!("merge.{}.driver", opts.name),
        "cargo-merge-driver merge %O %A %B %P",
    ]);
    git(&[
        "config",
        global,
        "--unset",
        &format!("merge.{}.recursive", opts.name),
    ]);
    install_gitattributes(opts, true)?;
    Ok(())
}

fn uninstall(opts: Install) -> Result<()> {
    let global = if opts.global { "--global" } else { "--local" };
    git(&[
        "config",
        global,
        "--remove-section",
        &format!("merge.{}", opts.name),
    ]);
    install_gitattributes(opts, false)?;
    Ok(())
}

fn merge(_opts: Opts) -> Result<()> {
    println!("merge");
    Ok(())
}

fn main() {
    let opts = Opts::parse();
    let result = match opts.subcommand {
        Subcommand::Install(i) => install(i),
        Subcommand::Uninstall(i) => uninstall(i),
        Subcommand::Merge => merge(opts),
    };
    exit(match result {
        Ok(()) => 0,
        Err(_) => -1,
    });
}
