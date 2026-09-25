//! Stamps the commit a binary was built from into `--version`, so a PDF can be
//! traced back to the exact renderer. A build outside a git checkout, from a
//! published crate, reports the version alone.

use std::process::Command;

fn main() {
    for path in [".git/HEAD", ".git/index", "src", "templates", "assets"] {
        println!("cargo:rerun-if-changed={path}");
    }
    let version = env!("CARGO_PKG_VERSION");
    let full = match git(&["rev-parse", "--short", "HEAD"]) {
        Some(hash) => {
            let dirty = git(&["status", "--porcelain", "--untracked-files=no"])
                .is_some_and(|status| !status.is_empty());
            format!("{version} ({hash}{})", if dirty { "-dirty" } else { "" })
        }
        None => version.to_owned(),
    };
    println!("cargo:rustc-env=CRATECV_VERSION={full}");
}

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}
