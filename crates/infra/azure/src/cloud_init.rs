/// Generates the cloud-init startup script for Azure VMs.
///
/// Azure passes `customData` to the VM as base64-encoded bytes.  Cloud-init
/// on Ubuntu 22.04 will execute the content directly when it starts with `#!/`.
///
/// The returned string is already base64-encoded and ready to embed in the
/// ARM `osProfile.customData` field.
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
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

/// Render the WireGuard startup script and return it base64-encoded for use
/// as the ARM `customData` value.
pub fn generate_wireguard_startup_script(
    server_private_key: &str,
    client_public_key: &str,
) -> Result<String> {
    let wg_config = render_server_config(server_private_key, client_public_key)?;

    let template_text: &str = include_str!("templates/startup_script.sh.hbs");
    let context = StartupScriptContext { wg_config };

    let handlebars = Handlebars::new();
    let script = handlebars
        .render_template(template_text, &context)
        .map_err(|error| ConfigurationError::TemplateRender {
            reason: error.to_string(),
        })?;

    eprintln!("[Azure] startup script:\n{}", script);

    // Azure customData must be base64-encoded.
    Ok(BASE64.encode(script.as_bytes()))
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
