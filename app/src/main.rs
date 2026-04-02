use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_PATCH_RELEASE_BASE_URL: &str =
    "https://github.com/PerishCode/resources/releases/download";
const DEFAULT_PATCH_LATEST_API_URL: &str =
    "https://api.github.com/repos/PerishCode/resources/releases/latest";

#[cfg(windows)]
const PATCH_ARCHIVE_EXTENSION: &str = "zip";
#[cfg(not(windows))]
const PATCH_ARCHIVE_EXTENSION: &str = "tar.gz";

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
            let version = version.or_else(|| std::env::var("OH_MY_OC_PATCH_VERSION").ok());

            if let Err(error) = patch(&path, version.as_deref(), force) {
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
    #[cfg(windows)]
    {
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .unwrap_or_else(|| fail("error: HOME or USERPROFILE is not set"));
        return PathBuf::from(home).join(".config/opencode");
    }

    #[cfg(not(windows))]
    {
        let home = std::env::var_os("HOME").unwrap_or_else(|| fail("error: HOME is not set"));
        PathBuf::from(home).join(".config/opencode")
    }
}

fn patch(target: &std::path::Path, version: Option<&str>, force: bool) -> Result<(), String> {
    fs::create_dir_all(target)
        .map_err(|e| format!("failed to create {}: {}", target.display(), e))?;

    let resolved_version = match version.filter(|value| !value.is_empty()) {
        Some(version) => version.to_string(),
        None => latest_patch_version()?,
    };
    let archive_name = format!("oh-my-oc-{resolved_version}.{PATCH_ARCHIVE_EXTENSION}");
    let archive_url = format!("{DEFAULT_PATCH_RELEASE_BASE_URL}/{resolved_version}/{archive_name}");
    let tmpdir = temp_dir()?;
    let archive = tmpdir.join(&archive_name);

    fetch_file(&archive_url, &archive)?;
    extract_archive(&archive, &tmpdir)?;

    let source_root = tmpdir.join("oh-my-oc").join("opencode");
    if !source_root.is_dir() {
        return Err(format!(
            "missing oh-my-oc/opencode/ directory in {}",
            archive_name
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
        replace_file(&staged_path, &path)?;
    }

    Ok(())
}

fn replace_file(staged_path: &std::path::Path, path: &std::path::Path) -> Result<(), String> {
    if !path.exists() {
        return fs::rename(staged_path, path).map_err(|e| {
            format!(
                "failed to move {} to {}: {}",
                staged_path.display(),
                path.display(),
                e
            )
        });
    }

    let backup_path = backup_path(path);
    fs::rename(path, &backup_path)
        .map_err(|e| format!("failed to back up {}: {}", path.display(), e))?;

    match fs::rename(staged_path, path) {
        Ok(()) => {
            fs::remove_file(&backup_path)
                .map_err(|e| format!("failed to clean up {}: {}", backup_path.display(), e))?;
            Ok(())
        }
        Err(e) => {
            let restore_error = fs::rename(&backup_path, path).err();
            if let Some(restore_error) = restore_error {
                Err(format!(
                    "failed to move {} to {}: {}; restore from {} also failed: {}",
                    staged_path.display(),
                    path.display(),
                    e,
                    backup_path.display(),
                    restore_error
                ))
            } else {
                Err(format!(
                    "failed to move {} to {}: {}",
                    staged_path.display(),
                    path.display(),
                    e
                ))
            }
        }
    }
}

fn backup_path(path: &std::path::Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(OsString::from)
        .unwrap_or_else(|| OsString::from("backup"));
    name.push(".oh-my-oc-backup");
    path.with_file_name(name)
}

fn fetch_file(url: &str, output: &std::path::Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        return run_powershell(&[
            "-NoProfile",
            "-Command",
            "Invoke-WebRequest -UseBasicParsing -Uri $args[0] -OutFile $args[1]",
            url,
            &output.display().to_string(),
        ])
        .map_err(|e| format!("failed to fetch {url}: {e}"));
    }

    #[cfg(not(windows))]
    {
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
}

fn latest_patch_version() -> Result<String, String> {
    #[cfg(windows)]
    {
        return powershell_output(&[
            "-NoProfile",
            "-Command",
            "(Invoke-RestMethod -UseBasicParsing -Uri $args[0]).tag_name",
            DEFAULT_PATCH_LATEST_API_URL,
        ])
        .map_err(|e| format!("failed to resolve latest patch release: {e}"));
    }

    #[cfg(not(windows))]
    {
        let output = Command::new("curl")
            .args(["-fsSL", DEFAULT_PATCH_LATEST_API_URL])
            .output()
            .map_err(|e| format!("failed to query latest patch release: {e}"))?;
        if !output.status.success() {
            return Err("failed to resolve latest patch release".to_string());
        }

        extract_tag_name(&String::from_utf8_lossy(&output.stdout))
            .ok_or_else(|| "failed to parse latest patch release tag".to_string())
    }
}

#[cfg(not(windows))]
fn extract_tag_name(body: &str) -> Option<String> {
    let marker = "\"tag_name\":\"";
    let start = body.find(marker)? + marker.len();
    let end = body[start..].find('"')?;
    Some(body[start..start + end].to_string())
}

fn extract_archive(tarball: &std::path::Path, dir: &std::path::Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        return run_powershell(&[
            "-NoProfile",
            "-Command",
            "Expand-Archive -LiteralPath $args[0] -DestinationPath $args[1] -Force",
            &tarball.display().to_string(),
            &dir.display().to_string(),
        ])
        .map_err(|e| format!("failed to extract {}: {e}", tarball.display()));
    }

    #[cfg(not(windows))]
    {
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

#[cfg(windows)]
fn run_powershell(args: &[&str]) -> Result<(), String> {
    let output = Command::new("powershell")
        .args(args)
        .output()
        .map_err(|e| format!("failed to run powershell: {e}"))?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        Err("powershell command failed".to_string())
    } else {
        Err(stderr)
    }
}

#[cfg(windows)]
fn powershell_output(args: &[&str]) -> Result<String, String> {
    let output = Command::new("powershell")
        .args(args)
        .output()
        .map_err(|e| format!("failed to run powershell: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return if stderr.is_empty() {
            Err("powershell command failed".to_string())
        } else {
            Err(stderr)
        };
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        Err("powershell command returned no output".to_string())
    } else {
        Ok(stdout)
    }
}
