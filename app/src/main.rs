use std::ffi::OsString;
use std::fs;
#[cfg(windows)]
use std::fs::File;
use std::path::{Path, PathBuf};
#[cfg(not(windows))]
use std::process::Command;

use tracing::level_filters::LevelFilter;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_PATCH_RELEASE_BASE_URL: &str =
    "https://github.com/PerishCode/resources/releases/download";
const DEFAULT_PATCH_LATEST_API_URL: &str =
    "https://api.github.com/repos/PerishCode/resources/releases/latest";
const DEFAULT_LOG_LEVEL: &str = "info";

#[cfg(windows)]
const PATCH_ARCHIVE_EXTENSION: &str = "zip";
#[cfg(not(windows))]
const PATCH_ARCHIVE_EXTENSION: &str = "tar.gz";
const MANAGED_FILES: &[&str] = &[
    "opencode.json",
    "agent/commander.md",
    "agent/explorer.md",
    "agent/coder.md",
    "agent/advisor.md",
];

fn main() {
    let state = AppState::from_env_and_args();
    init_tracing(state.config.log_level, state.config.ansi);

    if let Err(error) = run(&state) {
        fail(&format!("error: {error}"));
    }
}

struct AppState {
    config: ConfigStore,
}

impl AppState {
    fn from_env_and_args() -> Self {
        Self {
            config: ConfigStore::from_env_and_args(),
        }
    }
}

struct ConfigStore {
    command: CommandConfig,
    log_level: LevelFilter,
    ansi: bool,
}

enum CommandConfig {
    Help,
    Version,
    Patch(PatchConfig),
}

struct PatchConfig {
    target: PathBuf,
    version: Option<(String, VersionSource)>,
    force: bool,
}

#[derive(Clone, Copy)]
enum VersionSource {
    Argument,
    Environment,
    Latest,
}

impl VersionSource {
    fn label(self) -> &'static str {
        match self {
            VersionSource::Argument => "CLI argument",
            VersionSource::Environment => "OH_MY_OC_PATCH_VERSION",
            VersionSource::Latest => "latest release",
        }
    }
}

impl ConfigStore {
    fn from_env_and_args() -> Self {
        let mut args = std::env::args().skip(1);
        let Some(first) = args.next() else {
            return Self {
                command: CommandConfig::Help,
                log_level: LevelFilter::INFO,
                ansi: true,
            };
        };

        match first.as_str() {
            "--help" => {
                if args.next().is_some() {
                    fail("error: unexpected extra arguments");
                }
                Self {
                    command: CommandConfig::Help,
                    log_level: resolve_log_level(None),
                    ansi: resolve_ansi(None),
                }
            }
            "--version" => {
                if args.next().is_some() {
                    fail("error: unexpected extra arguments");
                }
                Self {
                    command: CommandConfig::Version,
                    log_level: resolve_log_level(None),
                    ansi: resolve_ansi(None),
                }
            }
            "patch" => {
                let mut path = None;
                let mut version = None;
                let mut force = false;
                let mut log_level = None;
                let mut ansi = None;

                while let Some(arg) = args.next() {
                    match arg.as_str() {
                        "--path" => path = Some(next_value("--path", &mut args)),
                        "--version" => version = Some(next_value("--version", &mut args)),
                        "--force" => force = true,
                        "--log-level" => log_level = Some(next_value("--log-level", &mut args)),
                        "--ansi" => ansi = Some(next_value("--ansi", &mut args)),
                        "--help" => {
                            if args.next().is_some() {
                                fail("error: unexpected extra arguments");
                            }
                            return Self {
                                command: CommandConfig::Help,
                                log_level: resolve_log_level(log_level.as_deref()),
                                ansi: resolve_ansi(ansi.as_deref()),
                            };
                        }
                        _ => fail(&format!("error: unknown argument: {arg}")),
                    }
                }

                let target = path
                    .or_else(|| std::env::var("OH_MY_OC_PATCH_PATH").ok())
                    .map(PathBuf::from)
                    .unwrap_or_else(default_patch_path);

                let version = match version.filter(|value| !value.is_empty()) {
                    Some(value) => Some((value, VersionSource::Argument)),
                    None => std::env::var("OH_MY_OC_PATCH_VERSION")
                        .ok()
                        .filter(|value| !value.is_empty())
                        .map(|value| (value, VersionSource::Environment)),
                };

                Self {
                    command: CommandConfig::Patch(PatchConfig {
                        target,
                        version,
                        force,
                    }),
                    log_level: resolve_log_level(log_level.as_deref()),
                    ansi: resolve_ansi(ansi.as_deref()),
                }
            }
            other => fail(&format!("error: unknown argument or command: {other}")),
        }
    }
}

fn resolve_ansi(value: Option<&str>) -> bool {
    let resolved = value
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .or_else(|| std::env::var("OH_MY_OC_ANSI").ok())
        .unwrap_or_else(|| "true".to_string());

    match resolved.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        other => fail(&format!(
            "error: invalid ansi setting: {other} (expected true/false, on/off, yes/no, or 1/0)"
        )),
    }
}

fn resolve_log_level(value: Option<&str>) -> LevelFilter {
    let resolved = value
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .or_else(|| std::env::var("OH_MY_OC_LOG_LEVEL").ok())
        .unwrap_or_else(|| DEFAULT_LOG_LEVEL.to_string());

    match resolved.to_ascii_lowercase().as_str() {
        "off" => LevelFilter::OFF,
        "error" => LevelFilter::ERROR,
        "warn" => LevelFilter::WARN,
        "info" => LevelFilter::INFO,
        "debug" => LevelFilter::DEBUG,
        "trace" => LevelFilter::TRACE,
        other => fail(&format!(
            "error: invalid log level: {other} (expected off, error, warn, info, debug, or trace)"
        )),
    }
}

fn init_tracing(log_level: LevelFilter, ansi: bool) {
    let subscriber = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(log_level)
        .with_ansi(ansi)
        .with_target(false)
        .without_time()
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .unwrap_or_else(|e| fail(&format!("error: failed to initialize tracing: {e}")));
}

fn run(state: &AppState) -> Result<(), String> {
    match &state.config.command {
        CommandConfig::Help => {
            print_help();
            Ok(())
        }
        CommandConfig::Version => {
            println!("oh-my-oc {}", CURRENT_VERSION);
            Ok(())
        }
        CommandConfig::Patch(config) => patch(config),
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

fn print_help() {
    println!("oh-my-oc {}", CURRENT_VERSION);
    println!();
    println!("Usage:");
    println!(
        "  oh-my-oc patch [--path <value>] [--version <value>] [--force] [--log-level <value>] [--ansi <value>]"
    );
    println!("  oh-my-oc --help");
    println!("  oh-my-oc --version");
    println!();
    println!("Commands:");
    println!("  patch      Download and apply managed Opencode config files");
    println!();
    println!("Patch options:");
    println!("  --path <value>       Target config directory");
    println!("                       Default: ~/.config/opencode");
    println!("                       Env: OH_MY_OC_PATCH_PATH");
    println!("  --version <value>    Patch version to apply");
    println!("                       Default: latest release");
    println!("                       Env: OH_MY_OC_PATCH_VERSION");
    println!("  --force              Overwrite existing managed files");
    println!("  --log-level <value>  Log level: off, error, warn, info, debug, trace");
    println!("                       Default: info");
    println!("                       Env: OH_MY_OC_LOG_LEVEL");
    println!("  --ansi <value>       ANSI color output: true or false");
    println!("                       Default: true");
    println!("                       Env: OH_MY_OC_ANSI");
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

fn patch(config: &PatchConfig) -> Result<(), String> {
    tracing::info!(target = %config.target.display(), "patching target");
    if config.force {
        tracing::info!("overwrite mode enabled (--force)");
    }

    preflight_target(config)?;

    let (resolved_version, version_source) = match &config.version {
        Some((version, source)) => (version.clone(), *source),
        None => {
            tracing::info!("resolving latest patch version from GitHub releases");
            (latest_patch_version()?, VersionSource::Latest)
        }
    };

    tracing::info!(
        version = %resolved_version,
        source = %version_source.label(),
        "using patch version"
    );

    let archive_name = format!("oh-my-oc-{resolved_version}.{PATCH_ARCHIVE_EXTENSION}");
    let archive_url = format!("{DEFAULT_PATCH_RELEASE_BASE_URL}/{resolved_version}/{archive_name}");
    let tmpdir = temp_dir()?;
    let archive = tmpdir.join(&archive_name);

    tracing::info!(archive = %archive_name, "downloading patch archive");
    fetch_file(&archive_url, &archive).map_err(|e| format!("download step failed: {e}"))?;

    tracing::info!(archive = %archive_name, "extracting patch archive");
    extract_archive(&archive, &tmpdir).map_err(|e| format!("extract step failed: {e}"))?;

    let source_root = tmpdir.join("oh-my-oc").join("opencode");
    if !source_root.is_dir() {
        return Err(format!(
            "missing oh-my-oc/opencode/ directory in {}",
            archive_name
        ));
    }

    tracing::info!("verifying patch contents");
    let mut prepared = Vec::with_capacity(MANAGED_FILES.len());
    for relative in MANAGED_FILES {
        let path = config.target.join(relative);
        let source = source_root.join(relative);
        let contents = fs::read_to_string(&source)
            .map_err(|e| format!("failed to read {}: {}", source.display(), e))?;
        prepared.push((path, contents));
    }

    let staging = tmpdir.join("staging");
    fs::create_dir_all(&staging)
        .map_err(|e| format!("failed to create {}: {}", staging.display(), e))?;

    tracing::info!("applying managed files");
    for (path, contents) in &prepared {
        let staged_path = staging.join(path.strip_prefix(&config.target).unwrap_or(path));
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
        let staged_path = staging.join(path.strip_prefix(&config.target).unwrap_or(&path));
        replace_file(&staged_path, &path)?;
        tracing::info!(path = %path.display(), "applied file");
    }

    tracing::info!(target = %config.target.display(), "patch applied");
    Ok(())
}

fn preflight_target(config: &PatchConfig) -> Result<(), String> {
    if config.target.exists() && !config.target.is_dir() {
        return Err(format!(
            "target exists but is not a directory: {}",
            config.target.display()
        ));
    }

    if config.force {
        return Ok(());
    }

    let existing = MANAGED_FILES
        .iter()
        .map(|relative| config.target.join(relative))
        .filter(|path| path.exists())
        .collect::<Vec<_>>();

    if existing.is_empty() {
        return Ok(());
    }

    let mut message = String::from("managed files already exist; rerun with --force to overwrite:");
    for path in existing {
        message.push_str("\n  ");
        message.push_str(&path.display().to_string());
    }
    Err(message)
}

fn replace_file(staged_path: &Path, path: &Path) -> Result<(), String> {
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

fn backup_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(OsString::from)
        .unwrap_or_else(|| OsString::from("backup"));
    name.push(".oh-my-oc-backup");
    path.with_file_name(name)
}

fn fetch_file(url: &str, output: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        let response = ureq::get(url)
            .set("User-Agent", "oh-my-oc")
            .call()
            .map_err(|e| format!("failed to fetch {url}: {e}"))?;
        let mut file = File::create(output)
            .map_err(|e| format!("failed to create {}: {}", output.display(), e))?;
        std::io::copy(&mut response.into_reader(), &mut file)
            .map_err(|e| format!("failed to write {}: {}", output.display(), e))?;
        return Ok(());
    }

    #[cfg(not(windows))]
    {
        let output = Command::new("curl")
            .args(["-fsSL", url, "-o"])
            .arg(output)
            .output()
            .map_err(|e| format!("failed to run curl for {url}: {e}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr = stderr.trim();
            if stderr.is_empty() {
                return Err(format!(
                    "failed to fetch {url} (curl exited with {})",
                    output.status
                ));
            }
            return Err(format!(
                "failed to fetch {url} (curl exited with {}): {stderr}",
                output.status
            ));
        }
        Ok(())
    }
}

fn latest_patch_version() -> Result<String, String> {
    let response = ureq::get(DEFAULT_PATCH_LATEST_API_URL)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "oh-my-oc")
        .call()
        .map_err(|e| format!("failed to query latest patch release: {e}"))?;

    let body = response
        .into_string()
        .map_err(|e| format!("failed to read latest patch release response: {e}"))?;

    extract_tag_name(&body).ok_or_else(|| "failed to parse latest patch release tag".to_string())
}

fn extract_tag_name(body: &str) -> Option<String> {
    let key = "\"tag_name\"";
    let start = body.find(key)? + key.len();
    let rest = body[start..].trim_start();
    let rest = rest.strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_archive(tarball: &Path, dir: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        let file = File::open(tarball)
            .map_err(|e| format!("failed to open {}: {}", tarball.display(), e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("failed to read {}: {}", tarball.display(), e))?;
        fs::create_dir_all(dir)
            .map_err(|e| format!("failed to create {}: {}", dir.display(), e))?;
        for index in 0..archive.len() {
            let mut entry = archive
                .by_index(index)
                .map_err(|e| format!("failed to read {}: {}", tarball.display(), e))?;
            let outpath = dir.join(entry.mangled_name());
            if entry.is_dir() {
                fs::create_dir_all(&outpath)
                    .map_err(|e| format!("failed to create {}: {}", outpath.display(), e))?;
            } else {
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("failed to create {}: {}", parent.display(), e))?;
                }
                let mut outfile = File::create(&outpath)
                    .map_err(|e| format!("failed to create {}: {}", outpath.display(), e))?;
                std::io::copy(&mut entry, &mut outfile)
                    .map_err(|e| format!("failed to write {}: {}", outpath.display(), e))?;
            }
        }
        return Ok(());
    }

    #[cfg(not(windows))]
    {
        let status = std::process::Command::new("tar")
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
