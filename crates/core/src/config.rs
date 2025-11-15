use handlebars::Handlebars;
use serde::Serialize;

#[derive(Serialize)]
struct ClientConfigContext {
    client_private_key: String,
    server_public_key: String,
    server_ip_v4: String,
}

pub fn generate_client_config(
    client_private_key: &str,
    server_public_key: &str,
    server_ip_v4: &str,
) -> String {
    let template_text: &str = include_str!("templates/client_config.hbs");

    // 2. Build the context (the data injected into the template)
    let context = ClientConfigContext {
        client_private_key: client_private_key.to_string(),
        server_public_key: server_public_key.to_string(),
        server_ip_v4: server_ip_v4.to_string(),
    };

    // 3. Render the template
    let handlebars_registry = Handlebars::new();

    let config = handlebars_registry
        .render_template(template_text, &context)
        .expect("Failed to render client configuration template");
    println!("{}", &config);
    config
}
