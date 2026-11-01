use byocvpn_core::error::Result;
use log::{debug, info};

use crate::arm_resource::ArmResource;
use crate::client::AzureClient;
use crate::models::{
    NsgProperties, NsgRequest, NsgResponse, SecurityRule, SecurityRuleProperties, byocvpn_tags,
};
use crate::resource_operations::{get_existing, put_and_wait};

const WIREGUARD_PORT: &str = "51820";

pub async fn ensure_nsg(client: &AzureClient, location: &str) -> Result<String> {
    let resource = ArmResource::network_security_group(location);
    let desired_rules = build_wireguard_security_rules();

    match get_existing::<NsgResponse>(client, &resource).await? {
        Some(existing) if has_desired_rules(&existing, &desired_rules) => {
            debug!("[Azure] {resource} already exists in {location} with current rules.");
        }
        Some(existing) => {
            info!("[Azure] {resource} security rules have drifted, patching...");
            let body = NsgRequest {
                location: existing.location.unwrap_or_else(|| location.to_string()),
                tags: existing.tags.unwrap_or_else(byocvpn_tags),
                properties: NsgProperties {
                    security_rules: desired_rules,
                },
            };
            put_and_wait(client, &resource, &body)
                .await
                .map_err(|error| error.with_context("patching network security group rules"))?;
            info!("[Azure] {resource} security rules patched.");
        }
        None => {
            info!("[Azure] Creating {resource} in {location}...");
            let body = NsgRequest {
                location: location.to_string(),
                tags: byocvpn_tags(),
                properties: NsgProperties {
                    security_rules: desired_rules,
                },
            };
            put_and_wait(client, &resource, &body)
                .await
                .map_err(|error| error.with_context("creating network security group"))?;
            info!("[Azure] {resource} created in {location}.");
        }
    }

    Ok(client.build_resource_id(&resource))
}

fn build_wireguard_security_rules() -> Vec<SecurityRule> {
    vec![
        build_inbound_allow_rule("AllowWireGuard", 1000, "UDP"),
        build_inbound_allow_rule("AllowHealthEndpoint", 1001, "TCP"),
    ]
}

fn build_inbound_allow_rule(name: &str, priority: u32, protocol: &str) -> SecurityRule {
    SecurityRule {
        name: name.to_string(),
        properties: SecurityRuleProperties {
            priority,
            protocol: protocol.to_string(),
            access: "Allow".to_string(),
            direction: "Inbound".to_string(),
            source_address_prefix: "*".to_string(),
            source_port_range: "*".to_string(),
            destination_address_prefix: "*".to_string(),
            destination_port_range: WIREGUARD_PORT.to_string(),
        },
    }
}

fn has_desired_rules(existing: &NsgResponse, desired_rules: &[SecurityRule]) -> bool {
    let existing_rules = existing
        .properties
        .as_ref()
        .and_then(|properties| properties.security_rules.as_deref())
        .unwrap_or(&[]);
    desired_rules.iter().all(|desired| {
        existing_rules
            .iter()
            .any(|rule| matches_desired_rule(rule, desired))
    })
}

fn matches_desired_rule(rule: &SecurityRule, desired: &SecurityRule) -> bool {
    rule.name == desired.name
        && rule.properties.priority == desired.properties.priority
        && rule
            .properties
            .protocol
            .eq_ignore_ascii_case(&desired.properties.protocol)
        && rule.properties.access == desired.properties.access
        && rule.properties.direction == desired.properties.direction
        && rule.properties.destination_port_range == desired.properties.destination_port_range
}
