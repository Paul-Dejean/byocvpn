use byocvpn_core::managed_resources::{ANY_IPV4_CIDR, ANY_IPV6_CIDR, WIREGUARD_PORT};
use google_cloud_compute_v1::model::{
    Firewall,
    firewall::{Allowed, Direction},
};
use log::info;

use crate::{
    client::GcpClient,
    error::{LabelOperation, Result},
    network::vpc::build_vpc_link,
    operations::await_compute_operation,
};

pub const FIREWALL_TARGET_TAG: &str = "byocvpn";
const FIREWALL_PRIORITY: i32 = 1000;

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

async fn get_firewall_rule(client: &GcpClient, name: &str) -> Result<Option<Firewall>> {
    let lookup = client
        .firewalls
        .get()
        .set_project(&client.project_id)
        .set_firewall(name)
        .send()
        .await
        .label_operation("compute.firewalls.get");
    match lookup {
        Ok(rule) => Ok(Some(rule)),
        Err(error) if error.is_not_found() => Ok(None),
        Err(error) => Err(error),
    }
}

fn matches_desired_rule(existing_rule: &Firewall, rule: &FirewallRuleSpec) -> bool {
    existing_rule.allowed == build_desired_allowed()
        && existing_rule.source_ranges == [rule.source_range.to_string()]
        && existing_rule.target_tags == [FIREWALL_TARGET_TAG.to_string()]
}

async fn patch_firewall_rule(client: &GcpClient, rule: &FirewallRuleSpec) -> Result<()> {
    let patch_body = Firewall::new()
        .set_allowed(build_desired_allowed())
        .set_source_ranges([rule.source_range])
        .set_target_tags([FIREWALL_TARGET_TAG])
        .set_direction(Direction::Ingress)
        .set_priority(FIREWALL_PRIORITY);
    let patch = client
        .firewalls
        .patch()
        .set_project(&client.project_id)
        .set_firewall(rule.name)
        .set_body(patch_body)
        .poller();
    await_compute_operation("compute.firewalls.update", patch).await?;
    info!("[GCP] Firewall rule '{}' patched.", rule.name);
    Ok(())
}

async fn create_firewall_rule(client: &GcpClient, rule: &FirewallRuleSpec) -> Result<()> {
    let firewall = Firewall::new()
        .set_name(rule.name)
        .set_network(build_vpc_link(client))
        .set_description(rule.description)
        .set_direction(Direction::Ingress)
        .set_priority(FIREWALL_PRIORITY)
        .set_target_tags([FIREWALL_TARGET_TAG])
        .set_allowed(build_desired_allowed())
        .set_source_ranges([rule.source_range]);
    let insert = client
        .firewalls
        .insert()
        .set_project(&client.project_id)
        .set_body(firewall)
        .poller();
    await_compute_operation("compute.firewalls.create", insert).await?;
    info!("[GCP] Firewall rule '{}' created.", rule.name);
    Ok(())
}

fn build_desired_allowed() -> Vec<Allowed> {
    ["udp", "tcp"]
        .into_iter()
        .map(|protocol| {
            Allowed::new()
                .set_ip_protocol(protocol)
                .set_ports([WIREGUARD_PORT.to_string()])
        })
        .collect()
}
