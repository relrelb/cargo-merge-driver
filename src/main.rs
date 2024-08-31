use clap::{Parser, Subcommand};
use std::fs;
use std::fs::File;
use std::io::{Result, Write};
use std::path::PathBuf;
use std::process::{exit, Command};

#[derive(Subcommand)]
enum Subcmd {
    Install(Install),
    Uninstall(Install),
    Merge(Merge),
}

#[derive(Parser)]
struct Install {
    #[arg(short, long)]
    global: bool,

    #[arg(long, default_value = "cargo-merge-driver")]
    name: String,
}

#[derive(Parser)]
struct Merge {
    ancestor: PathBuf,

    current: PathBuf,

    other: PathBuf,

    placeholder: PathBuf,
}

#[derive(Parser)]
struct Opts {
    #[command(subcommand)]
    subcommand: Subcmd,
}

fn git(args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .output()
        .expect("Failed to execute git");
    String::from_utf8(output.stdout).unwrap().trim_end().into()
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
    println!("Updating {}...", path.to_str().unwrap());
    let text = fs::read_to_string(&path).unwrap_or("".into());
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

fn merge(opts: Merge) -> Result<()> {
    let output = git(&[
        "merge-file",
        "-p",
        opts.current.to_str().unwrap(),
        opts.ancestor.to_str().unwrap(),
        opts.other.to_str().unwrap(),
    ]);
    fs::write(&opts.placeholder, output)?;
    Command::new("cargo")
        .arg("generate-lockfile")
        .status()
        .expect("Failed to execute cargo");
    fs::copy(&opts.current, &opts.placeholder)?;
    Ok(())
}

fn main() {
    let opts = Opts::parse();
    let result = match opts.subcommand {
        Subcmd::Install(i) => install(i),
        Subcmd::Uninstall(i) => uninstall(i),
        Subcmd::Merge(m) => merge(m),
    };
    exit(match result {
        Ok(()) => 0,
        Err(_) => -1,
    });
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use crate::Opts;

    #[test]
    fn valid_command() {
        Opts::command().debug_assert();
    }
}
