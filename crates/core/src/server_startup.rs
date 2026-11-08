use handlebars::Handlebars;
use log::debug;
use serde::Serialize;

use crate::error::{ConfigurationError, Result};

const STARTUP_SCRIPT_TEMPLATE: &str =
    include_str!("templates/server_startup/server_startup_script.sh.hbs");
const WIREGUARD_SERVER_CONFIG_TEMPLATE: &str =
    include_str!("templates/server_startup/wireguard_server_config.hbs");
const STATUS_SERVER_SCRIPT: &str = include_str!("templates/server_startup/status_server.py");

const APT_INSTALL_COMMAND: &str =
    "apt-get update -y\napt-get install -y --no-install-recommends wireguard-tools iptables";
const DNF_INSTALL_COMMAND: &str = "dnf install -y wireguard-tools iptables";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Apt,
    Dnf,
}

impl PackageManager {
    const fn install_command(self) -> &'static str {
        match self {
            Self::Apt => APT_INSTALL_COMMAND,
            Self::Dnf => DNF_INSTALL_COMMAND,
        }
    }
}

pub fn generate_server_startup_script(
    package_manager: PackageManager,
    server_private_key: &str,
    client_public_key: &str,
) -> Result<String> {
    let wg_config = generate_wireguard_server_config(server_private_key, client_public_key)?;
    let context = StartupScriptContext {
        install_packages_command: package_manager.install_command(),
        wg_config,
        status_server_script: STATUS_SERVER_SCRIPT,
    };
    let script = render_template(STARTUP_SCRIPT_TEMPLATE, &context)?;
    debug!(
        "Generated server startup script for {package_manager:?} ({} bytes)",
        script.len()
    );
    Ok(script)
}

fn generate_wireguard_server_config(
    server_private_key: &str,
    client_public_key: &str,
) -> Result<String> {
    let context = WireguardServerConfigContext {
        server_private_key,
        client_public_key,
    };
    render_template(WIREGUARD_SERVER_CONFIG_TEMPLATE, &context)
}

fn render_template<Context: Serialize>(template_text: &str, context: &Context) -> Result<String> {
    Ok(Handlebars::new()
        .render_template(template_text, context)
        .map_err(ConfigurationError::TemplateRender)?)
}

#[derive(Serialize)]
struct StartupScriptContext {
    install_packages_command: &'static str,
    wg_config: String,
    status_server_script: &'static str,
}

#[derive(Serialize)]
struct WireguardServerConfigContext<'a> {
    server_private_key: &'a str,
    client_public_key: &'a str,
}
