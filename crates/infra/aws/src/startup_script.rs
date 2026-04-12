use byocvpn_core::error::{ConfigurationError, Result};
use handlebars::Handlebars;
use log::*;
use serde::Serialize;

#[derive(Serialize)]
struct WireguardCloudInitContext {
    wg_config: String,
}
pub(super) fn generate_server_startup_script(
    server_private_key: &str,
    client_public_key: &str,
) -> Result<String> {
    let wg_config = generate_wireguard_server_config(server_private_key, client_public_key)?;

    let template_text: &str = include_str!("templates/server_startup_script.sh.hbs");

    let context = WireguardCloudInitContext {
        wg_config: wg_config,
    };

    let handlebars_registry = Handlebars::new();

    let config = handlebars_registry
        .render_template(template_text, &context)
        .map_err(|error| ConfigurationError::TemplateRender {
            reason: error.to_string(),
        })?;
    info!("{}", &config);
    Ok(config)
}

#[derive(Serialize)]
struct ServerConfigContext {
    server_private_key: String,
    client_public_key: String,
}

fn generate_wireguard_server_config(
    server_private_key: &str,
    client_public_key: &str,
) -> Result<String> {
    let template_text: &str = include_str!("templates/wireguard_server_config.hbs");

    let context = ServerConfigContext {
        server_private_key: server_private_key.to_string(),
        client_public_key: client_public_key.to_string(),
    };

    let handlebars_registry = Handlebars::new();

    handlebars_registry
        .render_template(template_text, &context)
        .map_err(|error| {
            ConfigurationError::TemplateRender {
                reason: error.to_string(),
            }
            .into()
        })
}
