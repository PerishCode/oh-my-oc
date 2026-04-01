use std::fs;
use std::path::PathBuf;
use std::process::Command;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_PATCH_RELEASE_BASE_URL: &str =
    "https://github.com/PerishCode/resources/releases/download";

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(first) = args.next() else {
        print_help(0);
    };

    match first.as_str() {
        "--help" => print_help(0),
        "--version" => {
            if args.next().is_some() {
                fail("error: unexpected extra arguments");
            }
            println!("oh-my-oc {}", CURRENT_VERSION);
        }
        "patch" => {
            let mut path = None;
            let mut version = None;
            let mut force = false;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--path" => path = Some(next_value("--path", &mut args)),
                    "--version" => version = Some(next_value("--version", &mut args)),
                    "--force" => force = true,
                    "--help" => print_help(0),
                    _ => fail(&format!("error: unknown argument: {arg}")),
                }
            }

            let path = path
                .or_else(|| std::env::var("OH_MY_OC_PATCH_PATH").ok())
                .map(PathBuf::from)
                .unwrap_or_else(default_patch_path);
            let version = version
                .or_else(|| std::env::var("OH_MY_OC_PATCH_VERSION").ok())
                .unwrap_or_else(|| CURRENT_VERSION.to_string());

            if let Err(error) = patch(&path, &version, force) {
                fail(&format!("error: {error}"));
            }
        }
        other => fail(&format!("error: unknown argument or command: {other}")),
    }
}

fn next_value(name: &str, args: &mut impl Iterator<Item = String>) -> String {
    args.next()
        .unwrap_or_else(|| fail(&format!("error: missing value for {name}")))
}

fn fail(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(1);
}

fn print_help(code: i32) -> ! {
    println!("oh-my-oc {}", CURRENT_VERSION);
    println!();
    println!("Usage:");
    println!("  oh-my-oc patch [--path <value>] [--version <value>] [--force]");
    println!("  oh-my-oc --help");
    println!("  oh-my-oc --version");
    std::process::exit(code);
}

fn default_patch_path() -> PathBuf {
    let home = std::env::var_os("HOME").unwrap_or_else(|| fail("error: HOME is not set"));
    PathBuf::from(home).join(".config/opencode")
}

fn patch(target: &std::path::Path, version: &str, force: bool) -> Result<(), String> {
    fs::create_dir_all(target)
        .map_err(|e| format!("failed to create {}: {}", target.display(), e))?;

    let tarball = format!("oh-my-oc-{version}.tar.gz");
    let tarball_url =
        format!("{DEFAULT_PATCH_RELEASE_BASE_URL}/{version}/oh-my-oc-{version}.tar.gz");
    let tmpdir = temp_dir()?;
    let archive = tmpdir.join(&tarball);

    fetch_file(&tarball_url, &archive)?;
    extract_tarball(&archive, &tmpdir)?;

    let source_root = tmpdir.join("oh-my-oc").join("opencode");
    if !source_root.is_dir() {
        return Err(format!(
            "missing oh-my-oc/opencode/ directory in {}",
            tarball
        ));
    }

    let files = [
        "opencode.json",
        "agent/commander.md",
        "agent/explorer.md",
        "agent/coder.md",
        "agent/advisor.md",
    ];

    let mut prepared = Vec::with_capacity(files.len());

    for relative in files {
        let path = target.join(relative);
        if path.exists() && !force {
            return Err(format!("{} already exists", path.display()));
        }
        let source = source_root.join(relative);
        let contents = fs::read_to_string(&source)
            .map_err(|e| format!("failed to read {}: {}", source.display(), e))?;
        prepared.push((path, contents));
    }

    let staging = tmpdir.join("staging");
    fs::create_dir_all(&staging)
        .map_err(|e| format!("failed to create {}: {}", staging.display(), e))?;

    for (path, contents) in &prepared {
        let staged_path = staging.join(path.strip_prefix(target).unwrap_or(path));
        if let Some(parent) = staged_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create {}: {}", parent.display(), e))?;
        }
        fs::write(&staged_path, contents)
            .map_err(|e| format!("failed to write {}: {}", staged_path.display(), e))?;
    }

    for (path, _) in prepared {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create {}: {}", parent.display(), e))?;
        }
        let staged_path = staging.join(path.strip_prefix(target).unwrap_or(&path));
        fs::rename(&staged_path, &path).map_err(|e| {
            format!(
                "failed to move {} to {}: {}",
                staged_path.display(),
                path.display(),
                e
            )
        })?;
    }

    Ok(())
}

fn fetch_file(url: &str, output: &std::path::Path) -> Result<(), String> {
    let output = Command::new("curl")
        .args(["-fsSL", url, "-o"])
        .arg(output)
        .output()
        .map_err(|e| format!("failed to run curl for {url}: {e}"))?;
    if !output.status.success() {
        return Err(format!("failed to fetch {url}"));
    }
    Ok(())
}

fn extract_tarball(tarball: &std::path::Path, dir: &std::path::Path) -> Result<(), String> {
    let status = Command::new("tar")
        .args(["-xzf"])
        .arg(tarball)
        .args(["-C"])
        .arg(dir)
        .status()
        .map_err(|e| format!("failed to run tar for {}: {e}", tarball.display()))?;
    if !status.success() {
        return Err(format!("failed to extract {}", tarball.display()));
    }
    Ok(())
}

fn temp_dir() -> Result<PathBuf, String> {
    let mut base = std::env::temp_dir();
    base.push(format!(
        "oh-my-oc-{}-{}",
        std::process::id(),
        timestamp_nanos()
    ));
    fs::create_dir_all(&base).map_err(|e| format!("failed to create {}: {}", base.display(), e))?;
    Ok(base)
}

fn timestamp_nanos() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}
