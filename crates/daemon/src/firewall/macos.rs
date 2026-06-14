use byocvpn_core::error::{Result, SystemError};
use log::*;
use std::process::Command;

const ANCHOR_NAME: &str = "byocvpn";
const ANCHOR_FILE: &str = "/tmp/byocvpn-killswitch.pf";
const ANCHOR_DIRECTIVE: &str = "anchor \"byocvpn\"";
const PF_CONF: &str = "/etc/pf.conf";

pub fn apply(server_ip: &str, tun_name: &str) -> Result<()> {
    let anchor_rules = format!(
        "pass quick on lo0 all no state\nblock out all\nblock in all\npass out proto udp to {server_ip} port 51820\npass on {tun_name}\n"
    );

    std::fs::write(ANCHOR_FILE, &anchor_rules).map_err(|error| SystemError::KillSwitchFailed {
        reason: format!("failed to write pf anchor file: {}", error),
    })?;

    ensure_anchor_in_pf_conf()?;
    run_pfctl(&["-a", ANCHOR_NAME, "-f", ANCHOR_FILE])?;
    enable_pf();

    info!("[KillSwitch] pf anchor applied (server={}, tun={})", server_ip, tun_name);
    Ok(())
}

pub fn remove() -> Result<()> {
    run_pfctl(&["-a", ANCHOR_NAME, "-F", "all"])?;
    remove_anchor_from_pf_conf()?;
    let _ = std::fs::remove_file(ANCHOR_FILE);
    info!("[KillSwitch] pf anchor removed");
    Ok(())
}

fn is_anchor_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == ANCHOR_DIRECTIVE || trimmed.starts_with(&format!("{ANCHOR_DIRECTIVE} "))
}

fn ensure_anchor_in_pf_conf() -> Result<()> {
    let content = std::fs::read_to_string(PF_CONF).map_err(|error| SystemError::KillSwitchFailed {
        reason: format!("failed to read {}: {}", PF_CONF, error),
    })?;

    if content.lines().any(is_anchor_line) {
        return Ok(());
    }

    let updated = format!("{}\n{}\n", content.trim_end(), ANCHOR_DIRECTIVE);
    std::fs::write(PF_CONF, updated).map_err(|error| SystemError::KillSwitchFailed {
        reason: format!("failed to write {}: {}", PF_CONF, error),
    })?;

    run_pfctl(&["-f", PF_CONF])?;
    Ok(())
}

fn remove_anchor_from_pf_conf() -> Result<()> {
    let content = std::fs::read_to_string(PF_CONF).map_err(|error| SystemError::KillSwitchFailed {
        reason: format!("failed to read {}: {}", PF_CONF, error),
    })?;

    if !content.lines().any(is_anchor_line) {
        return Ok(());
    }

    let updated = content
        .lines()
        .filter(|line| !is_anchor_line(line))
        .collect::<Vec<_>>()
        .join("\n");

    std::fs::write(PF_CONF, format!("{}\n", updated)).map_err(|error| SystemError::KillSwitchFailed {
        reason: format!("failed to write {}: {}", PF_CONF, error),
    })?;

    run_pfctl(&["-f", PF_CONF])?;
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
