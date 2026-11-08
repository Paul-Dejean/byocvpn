use byocvpn_core::error::{Result, SystemError};
use log::{debug, info};
use std::process::Command;

const ANCHOR_NAME: &str = "byocvpn";
const ANCHOR_FILE: &str = "/tmp/byocvpn-killswitch.pf";
const ANCHOR_DIRECTIVE: &str = "anchor \"byocvpn\"";
const PF_CONF: &str = "/etc/pf.conf";
const PFCTL_COMMAND: &str = "pfctl";

pub fn apply(server_ip: &str, tun_name: &str) -> Result<()> {
    let anchor_rules = format!(
        "pass quick on lo0 all no state\nblock out all\nblock in all\npass out proto udp to {server_ip} port 51820\npass on {tun_name}\n"
    );

    std::fs::write(ANCHOR_FILE, &anchor_rules).map_err(|source| SystemError::KillSwitchFile {
        operation: "write",
        path: ANCHOR_FILE,
        source,
    })?;

    ensure_anchor_in_pf_conf()?;
    run_pfctl(&["-a", ANCHOR_NAME, "-f", ANCHOR_FILE])?;
    enable_pf()?;

    info!("[KillSwitch] pf anchor applied (server={server_ip}, tun={tun_name})");
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
    let content = read_pf_conf()?;

    if content.lines().any(is_anchor_line) {
        return Ok(());
    }

    let updated = format!("{}\n{}\n", content.trim_end(), ANCHOR_DIRECTIVE);
    write_pf_conf(updated)?;

    run_pfctl(&["-f", PF_CONF])?;
    Ok(())
}

fn remove_anchor_from_pf_conf() -> Result<()> {
    let content = read_pf_conf()?;

    if !content.lines().any(is_anchor_line) {
        return Ok(());
    }

    let updated = content
        .lines()
        .filter(|line| !is_anchor_line(line))
        .collect::<Vec<_>>()
        .join("\n");

    write_pf_conf(format!("{updated}\n"))?;

    run_pfctl(&["-f", PF_CONF])?;
    Ok(())
}

fn enable_pf() -> Result<()> {
    let output = Command::new(PFCTL_COMMAND)
        .args(["-e"])
        .output()
        .map_err(|source| SystemError::KillSwitchCommand {
            command: PFCTL_COMMAND,
            source,
        })?;
    if !output.status.success() {
        debug!(
            "[KillSwitch] pfctl -e exited with {}, pf is already enabled",
            output.status
        );
    }
    Ok(())
}

fn run_pfctl(args: &[&str]) -> Result<()> {
    let output = Command::new(PFCTL_COMMAND)
        .args(args)
        .output()
        .map_err(|source| SystemError::KillSwitchCommand {
            command: PFCTL_COMMAND,
            source,
        })?;

    if !output.status.success() {
        return Err(SystemError::KillSwitchCommandRejected {
            command: PFCTL_COMMAND,
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        }
        .into());
    }
    Ok(())
}

fn read_pf_conf() -> Result<String> {
    std::fs::read_to_string(PF_CONF).map_err(|source| {
        SystemError::KillSwitchFile {
            operation: "read",
            path: PF_CONF,
            source,
        }
        .into()
    })
}

fn write_pf_conf(content: String) -> Result<()> {
    std::fs::write(PF_CONF, content).map_err(|source| {
        SystemError::KillSwitchFile {
            operation: "write",
            path: PF_CONF,
            source,
        }
        .into()
    })
}
