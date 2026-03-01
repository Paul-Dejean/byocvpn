use byocvpn_core::error::{ConfigurationError, Result};
use handlebars::Handlebars;
use serde::Serialize;

#[derive(Serialize)]
struct WireguardCloudInitContext {
    wg_config: String,
}

pub fn generate_wireguard_cloud_init(
    server_private_key: &str,
    client_public_key: &str,
) -> Result<String> {
    let wg_config = generate_server_config(server_private_key, client_public_key)?;

    let template_text: &str = include_str!("templates/user_data.hbs");
    let context = WireguardCloudInitContext { wg_config };

    let handlebars_registry = Handlebars::new();
    let rendered = handlebars_registry
        .render_template(template_text, &context)
        .map_err(|e| ConfigurationError::TemplateRender {
            reason: e.to_string(),
        })?;
    eprintln!("[OCI] cloud-init script:\n{}", rendered);
    Ok(rendered)
}

#[derive(Serialize)]
struct ServerConfigContext {
    server_private_key: String,
    client_public_key: String,
}

fn generate_server_config(server_private_key: &str, client_public_key: &str) -> Result<String> {
    let template_text: &str = include_str!("templates/server_config.hbs");
    let context = ServerConfigContext {
        server_private_key: server_private_key.to_string(),
        client_public_key: client_public_key.to_string(),
    };
    let handlebars_registry = Handlebars::new();
    handlebars_registry
        .render_template(template_text, &context)
        .map_err(|e| {
            ConfigurationError::TemplateRender {
                reason: e.to_string(),
            }
            .into()
        })
}
