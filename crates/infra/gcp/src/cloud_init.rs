use byocvpn_core::error::{ConfigurationError, Result};
use handlebars::Handlebars;
use serde::Serialize;

#[derive(Serialize)]
struct StartupScriptContext {
    wg_config: String,
}

#[derive(Serialize)]
struct ServerConfigContext {
    server_private_key: String,
    client_public_key: String,
}

/// Render the full GCP startup script embedding the WireGuard server config.
pub fn generate_wireguard_startup_script(
    server_private_key: &str,
    client_public_key: &str,
) -> Result<String> {
    let wg_config = render_server_config(server_private_key, client_public_key)?;

    let template_text: &str = include_str!("templates/startup_script.sh.hbs");
    let context = StartupScriptContext { wg_config };

    let handlebars = Handlebars::new();
    let rendered = handlebars
        .render_template(template_text, &context)
        .map_err(|error| ConfigurationError::TemplateRender {
            reason: error.to_string(),
        })?;

    eprintln!("[GCP] startup script:\n{}", rendered);
    Ok(rendered)
}

fn render_server_config(server_private_key: &str, client_public_key: &str) -> Result<String> {
    let template_text: &str = include_str!("templates/server_config.hbs");
    let context = ServerConfigContext {
        server_private_key: server_private_key.to_string(),
        client_public_key: client_public_key.to_string(),
    };
    let handlebars = Handlebars::new();
    handlebars
        .render_template(template_text, &context)
        .map_err(|error| {
            ConfigurationError::TemplateRender {
                reason: error.to_string(),
            }
            .into()
        })
}
