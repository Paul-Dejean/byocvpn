use byocvpn_core::error::{Result, SystemError};
use log::*;
use std::io::Write;
use std::process::{Command, Stdio};

const ANCHOR_NAME: &str = "byocvpn";
const ANCHOR_FILE: &str = "/tmp/byocvpn-killswitch.pf";
const ANCHOR_DIRECTIVE: &str = "anchor \"byocvpn\"";

pub fn apply(server_ip: &str, tun_name: &str) -> Result<()> {
    let anchor_rules = format!(
        "block out all\nblock in all\npass on lo0\npass out proto udp to {server_ip} port 51820\npass on {tun_name}\n"
    );

    std::fs::write(ANCHOR_FILE, &anchor_rules).map_err(|error| SystemError::KillSwitchFailed {
        reason: format!("failed to write pf anchor file: {}", error),
    })?;

    ensure_anchor_in_main_ruleset()?;
    run_pfctl(&["-a", ANCHOR_NAME, "-f", ANCHOR_FILE])?;
    enable_pf();

    info!("[KillSwitch] pf anchor applied (server={}, tun={})", server_ip, tun_name);
    Ok(())
}

pub fn remove() -> Result<()> {
    run_pfctl(&["-a", ANCHOR_NAME, "-F", "all"])?;
    remove_anchor_from_main_ruleset()?;
    let _ = std::fs::remove_file(ANCHOR_FILE);
    info!("[KillSwitch] pf anchor removed");
    Ok(())
}

fn ensure_anchor_in_main_ruleset() -> Result<()> {
    let current_rules = read_active_rules();
    if current_rules.lines().any(|line| line.trim() == ANCHOR_DIRECTIVE) {
        return Ok(());
    }
    let updated_rules = format!("{ANCHOR_DIRECTIVE}\n{current_rules}");
    reload_main_rules(&updated_rules)
}

fn remove_anchor_from_main_ruleset() -> Result<()> {
    let current_rules = read_active_rules();
    if !current_rules.lines().any(|line| line.trim() == ANCHOR_DIRECTIVE) {
        return Ok(());
    }
    let updated_rules = current_rules
        .lines()
        .filter(|line| line.trim() != ANCHOR_DIRECTIVE)
        .collect::<Vec<_>>()
        .join("\n");
    reload_main_rules(&updated_rules)
}

fn read_active_rules() -> String {
    Command::new("pfctl")
        .args(["-sr"])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default()
}

fn reload_main_rules(rules: &str) -> Result<()> {
    let mut child = Command::new("pfctl")
        .args(["-f", "-"])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|error| SystemError::KillSwitchFailed {
            reason: format!("pfctl -f - spawn failed: {}", error),
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(rules.as_bytes()).map_err(|error| SystemError::KillSwitchFailed {
            reason: format!("failed to write rules to pfctl stdin: {}", error),
        })?;
    }

    let status = child.wait().map_err(|error| SystemError::KillSwitchFailed {
        reason: format!("pfctl -f - wait failed: {}", error),
    })?;

    if !status.success() {
        return Err(SystemError::KillSwitchFailed {
            reason: "pfctl failed to reload main ruleset".to_string(),
        }
        .into());
    }
    Ok(())
}

fn enable_pf() {
    // pfctl -e returns non-zero when pf is already enabled — that's fine
    let _ = Command::new("pfctl").args(["-e"]).output();
}

fn run_pfctl(args: &[&str]) -> Result<()> {
    let output = Command::new("pfctl")
        .args(args)
        .output()
        .map_err(|error| SystemError::KillSwitchFailed {
            reason: format!("pfctl exec failed: {}", error),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SystemError::KillSwitchFailed {
            reason: format!("pfctl {}: {}", args.join(" "), stderr.trim()),
        }
        .into());
    }
    Ok(())
}
