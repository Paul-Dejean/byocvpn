use byocvpn_core::error::{ConfigurationError, Result};
use handlebars::Handlebars;
use serde::Serialize;

#[derive(Serialize)]
struct WireguardCloudInitContext {
    wg_config: String,
}
pub(super) fn generate_wireguard_cloud_init(
    server_private_key: &str,
    client_public_key: &str,
) -> Result<String> {
    let wg_config = generate_server_config(server_private_key, client_public_key)?;

    let template_text: &str = include_str!("templates/user_data.hbs");

    // 2. Build the context (the data injected into the template)
    let context = WireguardCloudInitContext {
        wg_config: wg_config,
    };

    // 3. Render the template
    let handlebars_registry = Handlebars::new();

    let config = handlebars_registry
        .render_template(template_text, &context)
        .map_err(|e| ConfigurationError::TemplateRender {
            reason: e.to_string(),
        })?;
    println!("{}", &config);
    Ok(config)
}

#[derive(Serialize)]
struct ServerConfigContext {
    server_private_key: String,
    client_public_key: String,
}

fn generate_server_config(server_private_key: &str, client_public_key: &str) -> Result<String> {
    let template_text: &str = include_str!("templates/server_config.hbs");

    // 2. Build the context (the data injected into the template)
    let context = ServerConfigContext {
        server_private_key: server_private_key.to_string(),
        client_public_key: client_public_key.to_string(),
    };

    // 3. Render the template
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
