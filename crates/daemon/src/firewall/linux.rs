use std::process::Command;

use byocvpn_core::error::{Result, TunnelError};
use log::{debug, info};

const CHAIN: &str = "BYOCVPN_KS";
const IPTABLES_COMMAND: &str = "iptables";

pub fn apply(server_ip: &str, tun_name: &str) -> Result<()> {
    remove_if_exists();

    run_iptables(&["-N", CHAIN])?;
    run_iptables(&["-A", CHAIN, "-o", "lo", "-j", "ACCEPT"])?;
    run_iptables(&["-A", CHAIN, "-i", "lo", "-j", "ACCEPT"])?;
    run_iptables(&[
        "-A",
        CHAIN,
        "-d",
        &format!("{server_ip}/32"),
        "-p",
        "udp",
        "--dport",
        "51820",
        "-j",
        "ACCEPT",
    ])?;
    run_iptables(&["-A", CHAIN, "-o", tun_name, "-j", "ACCEPT"])?;
    run_iptables(&["-A", CHAIN, "-i", tun_name, "-j", "ACCEPT"])?;
    run_iptables(&["-A", CHAIN, "-j", "DROP"])?;

    run_iptables(&["-I", "OUTPUT", "1", "-j", CHAIN])?;
    run_iptables(&["-I", "INPUT", "1", "-j", CHAIN])?;

    info!("[KillSwitch] iptables chain applied (server={server_ip}, tun={tun_name})");
    Ok(())
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "the signature matches the other platforms' remove"
)]
pub fn remove() -> Result<()> {
    remove_if_exists();
    info!("[KillSwitch] iptables chain removed");
    Ok(())
}

fn remove_if_exists() {
    let teardown_steps: [&[&str]; 4] = [
        &["-D", "OUTPUT", "-j", CHAIN],
        &["-D", "INPUT", "-j", CHAIN],
        &["-F", CHAIN],
        &["-X", CHAIN],
    ];
    for arguments in teardown_steps {
        if let Err(error) = run_iptables(arguments) {
            debug!(
                "[KillSwitch] iptables {} skipped: {error}",
                arguments.join(" ")
            );
        }
    }
}

fn run_iptables(args: &[&str]) -> Result<()> {
    let output = Command::new(IPTABLES_COMMAND)
        .args(args)
        .output()
        .map_err(|source| TunnelError::KillSwitchCommand {
            command: IPTABLES_COMMAND,
            source,
        })?;

    if !output.status.success() {
        return Err(TunnelError::KillSwitchCommandRejected {
            command: IPTABLES_COMMAND,
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        }
        .into());
    }
    Ok(())
}
