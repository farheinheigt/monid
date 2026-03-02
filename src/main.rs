use std::env;
use std::io::IsTerminal;
use std::process::{Command, ExitCode};
use std::process::Stdio;

const LINE: &str = "------------------------------";

struct Styles {
    bold: &'static str,
    header: &'static str,
    line: &'static str,
    reset: &'static str,
}

impl Styles {
    fn detect() -> Self {
        if std::io::stdout().is_terminal() {
            Self {
                bold: "\x1b[1m",
                header: "\x1b[36m",
                line: "\x1b[90m",
                reset: "\x1b[0m",
            }
        } else {
            Self {
                bold: "",
                header: "",
                line: "",
                reset: "",
            }
        }
    }
}

#[derive(Debug, Default)]
struct Identity {
    uid: String,
    uname: String,
    gid: String,
    gname: String,
    groups: Vec<(u32, String)>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            println!("{message}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let user = match args.as_slice() {
        [] => default_user()?,
        [arg] if arg == "-h" || arg == "--help" => {
            print_usage();
            return Ok(());
        }
        [user] => user.clone(),
        _ => {
            print_usage();
            return Err("❌ Erreur : arguments invalides.".to_string());
        }
    };

    let id_cmd = resolve_id_cmd();
    if !command_status_ok(&id_cmd, &[user.as_str()]) {
        return Err(format!("❌ Erreur : utilisateur introuvable : {user}"));
    }

    let identity = load_identity(&id_cmd, &user)?;
    let styles = Styles::detect();

    print_section(&styles, "Utilisateur");
    println!("  {:>5} {}", identity.uid, identity.uname);
    println!();

    print_section(&styles, "Groupe principal");
    println!("  {:>5} {}", identity.gid, identity.gname);
    println!();

    print_section(&styles, "Groupes secondaires (tries)");
    for (gid, name) in identity.groups {
        println!("  {:>5} {}", gid, name);
    }

    Ok(())
}

fn print_usage() {
    println!(
        "\
Usage: monid [user]

Affiche UID, GID et groupes de l'utilisateur cible.

Options:
  -h, --help  Afficher l'aide"
    );
}

fn print_section(styles: &Styles, title: &str) {
    println!("{}{}{}", styles.line, LINE, styles.reset);
    println!("{}{}{}{}", styles.bold, styles.header, title, styles.reset);
}

fn default_user() -> Result<String, String> {
    if let Ok(user) = env::var("USER") {
        if !user.trim().is_empty() {
            return Ok(user);
        }
    }

    command_output(&resolve_id_cmd(), &["-un"])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "❌ Erreur : impossible de déterminer l'utilisateur courant.".to_string())
}

fn resolve_id_cmd() -> String {
    let macos_id = "/usr/bin/id";
    if std::path::Path::new(macos_id).is_file() {
        macos_id.to_string()
    } else {
        "id".to_string()
    }
}

fn command_status_ok(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn command_output(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() { None } else { Some(text) }
}

fn load_identity(id_cmd: &str, user: &str) -> Result<Identity, String> {
    let mut identity = Identity {
        uid: command_output(id_cmd, &["-u", user]).unwrap_or_default(),
        uname: command_output(id_cmd, &["-un", user]).unwrap_or_default(),
        gid: command_output(id_cmd, &["-g", user]).unwrap_or_default(),
        gname: command_output(id_cmd, &["-gn", user]).unwrap_or_default(),
        groups: load_groups_from_flags(id_cmd, user).unwrap_or_default(),
    };

    if identity.uid.is_empty()
        || identity.uname.is_empty()
        || identity.gid.is_empty()
        || identity.gname.is_empty()
        || identity.groups.is_empty()
    {
        let raw = command_output(id_cmd, &[user])
            .ok_or_else(|| format!("❌ Erreur : utilisateur introuvable : {user}"))?;
        fill_from_fallback_line(&mut identity, &raw);
    }

    if identity.uid.is_empty() || identity.uname.is_empty() || identity.gid.is_empty() || identity.gname.is_empty() {
        return Err(format!("❌ Erreur : impossible de lire l'identité de {user}."));
    }

    if identity.groups.is_empty() {
        identity.groups.push((
            identity.gid.parse::<u32>().unwrap_or_default(),
            identity.gname.clone(),
        ));
    }

    identity.groups.sort_by_key(|(gid, _)| *gid);
    Ok(identity)
}

fn load_groups_from_flags(id_cmd: &str, user: &str) -> Option<Vec<(u32, String)>> {
    let gids = command_output(id_cmd, &["-G", user])?;
    let names = command_output(id_cmd, &["-Gn", user])?;

    let gid_list: Vec<u32> = gids
        .split_whitespace()
        .filter_map(|item| item.parse::<u32>().ok())
        .collect();
    let name_list: Vec<String> = names.split_whitespace().map(ToString::to_string).collect();

    if gid_list.is_empty() || gid_list.len() != name_list.len() {
        return None;
    }

    Some(gid_list.into_iter().zip(name_list).collect())
}

fn fill_from_fallback_line(identity: &mut Identity, raw: &str) {
    if identity.uid.is_empty() {
        identity.uid = extract_numeric_value(raw, "uid=").unwrap_or_default();
    }
    if identity.uname.is_empty() {
        identity.uname = extract_name_value(raw, "uid=").unwrap_or_default();
    }
    if identity.gid.is_empty() {
        identity.gid = extract_numeric_value(raw, "gid=").unwrap_or_default();
    }
    if identity.gname.is_empty() {
        identity.gname = extract_name_value(raw, "gid=").unwrap_or_default();
    }
    if identity.groups.is_empty() {
        identity.groups = extract_groups(raw);
    }
}

fn extract_numeric_value(raw: &str, key: &str) -> Option<String> {
    let start = raw.find(key)? + key.len();
    let tail = &raw[start..];
    let digits: String = tail.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    if digits.is_empty() { None } else { Some(digits) }
}

fn extract_name_value(raw: &str, key: &str) -> Option<String> {
    let start = raw.find(key)? + key.len();
    let tail = &raw[start..];
    let open = tail.find('(')?;
    let after_open = &tail[open + 1..];
    let close = after_open.find(')')?;
    Some(after_open[..close].to_string())
}

fn extract_groups(raw: &str) -> Vec<(u32, String)> {
    let Some(groups_pos) = raw.find("groups=") else {
        return Vec::new();
    };

    raw[groups_pos + "groups=".len()..]
        .split(',')
        .filter_map(|chunk| {
            let chunk = chunk.trim();
            let open = chunk.find('(')?;
            let close = chunk[open + 1..].find(')')?;
            let gid = chunk[..open].trim().parse::<u32>().ok()?;
            let name = chunk[open + 1..open + 1 + close].trim().to_string();
            Some((gid, name))
        })
        .collect()
}
