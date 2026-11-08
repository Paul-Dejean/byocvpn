use byocvpn_core::{
    error::{NetworkProvisioningError, Result},
    managed_resources::{ANY_IPV4_CIDR, ANY_IPV6_CIDR, WIREGUARD_PORT},
};
use log::info;

use crate::{
    client::GcpClient,
    models::{
        CreateFirewallRuleRequest, FirewallAllowed, FirewallRuleResponse, Operation,
        PatchFirewallRuleRequest,
    },
    network::vpc::build_vpc_link,
    operations::wait_for_compute_operation,
};

pub const FIREWALL_TARGET_TAG: &str = "byocvpn";
const FIREWALL_DIRECTION: &str = "INGRESS";
const FIREWALL_PRIORITY: u32 = 1000;

struct FirewallRuleSpec {
    name: &'static str,
    source_range: &'static str,
    description: &'static str,
}

const FIREWALL_RULES: [FirewallRuleSpec; 2] = [
    FirewallRuleSpec {
        name: "byocvpn-wireguard-ipv4",
        source_range: ANY_IPV4_CIDR,
        description: "Allow WireGuard UDP and health TCP on 51820 (IPv4) for byocvpn",
    },
    FirewallRuleSpec {
        name: "byocvpn-wireguard-ipv6",
        source_range: ANY_IPV6_CIDR,
        description: "Allow WireGuard UDP and health TCP on 51820 (IPv6) for byocvpn",
    },
];

pub async fn ensure_firewall_rules(client: &GcpClient) -> Result<()> {
    for rule in &FIREWALL_RULES {
        match get_firewall_rule(client, rule.name).await? {
            Some(existing_rule) if matches_desired_rule(&existing_rule, rule) => {}
            Some(_) => {
                info!(
                    "[GCP] Firewall rule '{}' has drifted, patching...",
                    rule.name
                );
                patch_firewall_rule(client, rule).await?;
            }
            None => create_firewall_rule(client, rule).await?,
        }
    }
    Ok(())
}

async fn get_firewall_rule(client: &GcpClient, name: &str) -> Result<Option<FirewallRuleResponse>> {
    let url = client.build_compute_url(&format!("global/firewalls/{name}"));
    match client.get::<FirewallRuleResponse>(&url).await {
        Ok(rule) => Ok(Some(rule)),
        Err(error) if error.is_not_found() => Ok(None),
        Err(error) => Err(error),
    }
}

fn matches_desired_rule(existing_rule: &FirewallRuleResponse, rule: &FirewallRuleSpec) -> bool {
    existing_rule.allowed.as_deref() == Some(&build_desired_allowed())
        && existing_rule.source_ranges.as_deref() == Some(&[rule.source_range.to_string()])
        && existing_rule.target_tags.as_deref() == Some(&[FIREWALL_TARGET_TAG.to_string()])
}

async fn patch_firewall_rule(client: &GcpClient, rule: &FirewallRuleSpec) -> Result<()> {
    let url = client.build_compute_url(&format!("global/firewalls/{}", rule.name));
    let body = PatchFirewallRuleRequest {
        allowed: build_desired_allowed(),
        source_ranges: vec![rule.source_range.to_string()],
        target_tags: vec![FIREWALL_TARGET_TAG.to_string()],
        direction: FIREWALL_DIRECTION.to_string(),
        priority: FIREWALL_PRIORITY,
    };
    let operation: Operation = client.patch(&url, &body).await.map_err(|error| {
        NetworkProvisioningError::SecurityGroupCreationFailed {
            reason: format!("Failed to patch firewall rule '{}': {error}", rule.name),
        }
    })?;
    wait_for_compute_operation(client, &operation).await?;
    info!("[GCP] Firewall rule '{}' patched.", rule.name);
    Ok(())
}

async fn create_firewall_rule(client: &GcpClient, rule: &FirewallRuleSpec) -> Result<()> {
    let body = CreateFirewallRuleRequest {
        name: rule.name.to_string(),
        network: build_vpc_link(client),
        description: rule.description.to_string(),
        direction: FIREWALL_DIRECTION.to_string(),
        priority: FIREWALL_PRIORITY,
        target_tags: vec![FIREWALL_TARGET_TAG.to_string()],
        allowed: build_desired_allowed(),
        source_ranges: vec![rule.source_range.to_string()],
    };
    let operation: Operation = client
        .post(&client.build_compute_url("global/firewalls"), &body)
        .await
        .map_err(
            |error| NetworkProvisioningError::SecurityGroupCreationFailed {
                reason: error.to_string(),
            },
        )?;
    wait_for_compute_operation(client, &operation).await?;
    info!("[GCP] Firewall rule '{}' created.", rule.name);
    Ok(())
}

fn build_desired_allowed() -> Vec<FirewallAllowed> {
    ["udp", "tcp"]
        .into_iter()
        .map(|protocol| FirewallAllowed {
            ip_protocol: protocol.to_string(),
            ports: vec![WIREGUARD_PORT.to_string()],
        })
        .collect()
}
