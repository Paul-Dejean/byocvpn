use byocvpn_core::error::{Result, SystemError};
use log::*;
use std::process::Command;

const CHAIN: &str = "BYOCVPN_KS";

pub fn apply(server_ip: &str, tun_name: &str) -> Result<()> {
    remove_if_exists();

    run_iptables(&["-N", CHAIN])?;
    run_iptables(&["-A", CHAIN, "-o", "lo", "-j", "ACCEPT"])?;
    run_iptables(&["-A", CHAIN, "-i", "lo", "-j", "ACCEPT"])?;
    run_iptables(&[
        "-A", CHAIN, "-d", &format!("{}/32", server_ip),
        "-p", "udp", "--dport", "51820", "-j", "ACCEPT",
    ])?;
    run_iptables(&["-A", CHAIN, "-o", tun_name, "-j", "ACCEPT"])?;
    run_iptables(&["-A", CHAIN, "-i", tun_name, "-j", "ACCEPT"])?;
    run_iptables(&["-A", CHAIN, "-j", "DROP"])?;

    run_iptables(&["-I", "OUTPUT", "1", "-j", CHAIN])?;
    run_iptables(&["-I", "INPUT", "1", "-j", CHAIN])?;

    info!("[KillSwitch] iptables chain applied (server={}, tun={})", server_ip, tun_name);
    Ok(())
}

pub fn remove() -> Result<()> {
    remove_if_exists();
    info!("[KillSwitch] iptables chain removed");
    Ok(())
}

fn remove_if_exists() {
    let _ = run_iptables(&["-D", "OUTPUT", "-j", CHAIN]);
    let _ = run_iptables(&["-D", "INPUT", "-j", CHAIN]);
    let _ = run_iptables(&["-F", CHAIN]);
    let _ = run_iptables(&["-X", CHAIN]);
}

fn run_iptables(args: &[&str]) -> Result<()> {
    let output =
        Command::new("iptables")
            .args(args)
            .output()
            .map_err(|error| SystemError::KillSwitchFailed {
                reason: format!("iptables exec failed: {}", error),
            })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SystemError::KillSwitchFailed {
            reason: format!("iptables {}: {}", args.join(" "), stderr.trim()),
        }
        .into());
    }
    Ok(())
}
