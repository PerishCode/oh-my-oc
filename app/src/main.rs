use std::fs;
use std::path::PathBuf;
use std::process::Command;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const OPENCODE_JSON: &str = include_str!("../../resources/patch/opencode/opencode.json");
const COMMANDER_MD: &str = include_str!("../../resources/patch/opencode/agent/commander.md");
const EXPLORER_MD: &str = include_str!("../../resources/patch/opencode/agent/explorer.md");
const CODER_MD: &str = include_str!("../../resources/patch/opencode/agent/coder.md");
const ADVISOR_MD: &str = include_str!("../../resources/patch/opencode/agent/advisor.md");

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

            let resource_url_template = std::env::var("OH_MY_OC_PATCH_RESOURCE_URL_TEMPLATE").ok();

            if let Err(error) = patch(&path, &version, resource_url_template.as_deref(), force) {
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

fn patch(
    target: &std::path::Path,
    version: &str,
    resource_url_template: Option<&str>,
    force: bool,
) -> Result<(), String> {
    fs::create_dir_all(target)
        .map_err(|e| format!("failed to create {}: {}", target.display(), e))?;

    let files = [
        "opencode.json",
        "agent/commander.md",
        "agent/explorer.md",
        "agent/coder.md",
        "agent/advisor.md",
    ];

    for relative in files {
        let path = target.join(relative);
        if path.exists() && !force {
            return Err(format!("{} already exists", path.display()));
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create {}: {}", parent.display(), e))?;
        }
        let contents = patch_resource(relative, version, resource_url_template)?;
        fs::write(&path, contents)
            .map_err(|e| format!("failed to write {}: {}", path.display(), e))?;
    }

    Ok(())
}

fn patch_resource(
    path: &str,
    version: &str,
    resource_url_template: Option<&str>,
) -> Result<String, String> {
    if let Some(template) = resource_url_template {
        if !template.contains("{path}") {
            return Err("OH_MY_OC_PATCH_RESOURCE_URL_TEMPLATE must include {path}".to_string());
        }
        let url = template
            .replace("{version}", version)
            .replace("{path}", path);
        let output = Command::new("curl")
            .args(["-fsSL", &url])
            .output()
            .map_err(|e| format!("failed to run curl for {url}: {e}"))?;
        if !output.status.success() {
            return Err(format!("failed to fetch {url}"));
        }
        return String::from_utf8(output.stdout)
            .map_err(|e| format!("fetched {url} was not valid utf-8: {e}"));
    }

    if version != CURRENT_VERSION {
        return Err(format!(
            "patch version {version} is not available without OH_MY_OC_PATCH_RESOURCE_URL_TEMPLATE"
        ));
    }

    Ok(match path {
        "opencode.json" => OPENCODE_JSON,
        "agent/commander.md" => COMMANDER_MD,
        "agent/explorer.md" => EXPLORER_MD,
        "agent/coder.md" => CODER_MD,
        "agent/advisor.md" => ADVISOR_MD,
        _ => unreachable!(),
    }
    .to_string())
}
