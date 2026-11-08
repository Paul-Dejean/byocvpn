use std::collections::HashSet;

use aws_sdk_ec2::types::{Filter, IpPermission, IpRange, Ipv6Range, SecurityGroup};
use byocvpn_core::{
    error::{NetworkProvisioningError, Result},
    managed_resources::{ANY_IPV4_CIDR, ANY_IPV6_CIDR, WIREGUARD_PORT},
};
use log::info;

use crate::{client::AwsClient, error::extract_error_message};

const SECURITY_GROUP_NAME: &str = "byocvpn-security-group";
const SECURITY_GROUP_DESCRIPTION: &str = "BYOC VPN server";

pub async fn ensure_security_group(client: &AwsClient, vpc_id: &str) -> Result<String> {
    if let Some(security_group_id) = get_security_group_by_name(client).await? {
        reconcile_ingress_rules(client, &security_group_id).await?;
        return Ok(security_group_id);
    }
    create_security_group(client, vpc_id).await
}

async fn get_security_group_by_name(client: &AwsClient) -> Result<Option<String>> {
    let response = client
        .ec2
        .describe_security_groups()
        .filters(
            Filter::builder()
                .name("group-name")
                .values(SECURITY_GROUP_NAME)
                .build(),
        )
        .send()
        .await
        .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
            reason: extract_error_message(&error),
        })?;

    Ok(response
        .security_groups()
        .first()
        .and_then(|security_group| security_group.group_id())
        .map(ToString::to_string))
}

async fn create_security_group(client: &AwsClient, vpc_id: &str) -> Result<String> {
    let response = client
        .ec2
        .create_security_group()
        .vpc_id(vpc_id)
        .group_name(SECURITY_GROUP_NAME)
        .description(SECURITY_GROUP_DESCRIPTION)
        .send()
        .await
        .map_err(
            |error| NetworkProvisioningError::SecurityGroupCreationFailed {
                reason: extract_error_message(&error),
            },
        )?;

    let security_group_id = response
        .group_id()
        .ok_or(NetworkProvisioningError::MissingSecurityGroupIdentifier)?
        .to_string();
    info!("[AWS] Created security group {security_group_id}");

    let desired_permissions = build_desired_ingress_rules()
        .iter()
        .map(IngressRule::build_ip_permission)
        .collect();
    authorize_ingress(client, &security_group_id, desired_permissions).await?;
    Ok(security_group_id)
}

async fn reconcile_ingress_rules(client: &AwsClient, security_group_id: &str) -> Result<()> {
    let response = client
        .ec2
        .describe_security_groups()
        .group_ids(security_group_id)
        .send()
        .await
        .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
            reason: extract_error_message(&error),
        })?;

    let current_rules: HashSet<IngressRule> = response
        .security_groups()
        .first()
        .map(SecurityGroup::ip_permissions)
        .unwrap_or_default()
        .iter()
        .flat_map(IngressRule::from_ip_permission)
        .collect();
    let desired_rules = build_desired_ingress_rules();

    let rules_to_add: Vec<IpPermission> = desired_rules
        .difference(&current_rules)
        .map(IngressRule::build_ip_permission)
        .collect();
    let rules_to_remove: Vec<IpPermission> = current_rules
        .difference(&desired_rules)
        .map(IngressRule::build_ip_permission)
        .collect();

    if !rules_to_add.is_empty() {
        authorize_ingress(client, security_group_id, rules_to_add).await?;
    }
    if !rules_to_remove.is_empty() {
        revoke_ingress(client, security_group_id, rules_to_remove).await?;
    }
    Ok(())
}

async fn authorize_ingress(
    client: &AwsClient,
    security_group_id: &str,
    permissions: Vec<IpPermission>,
) -> Result<()> {
    client
        .ec2
        .authorize_security_group_ingress()
        .group_id(security_group_id)
        .set_ip_permissions(Some(permissions))
        .send()
        .await
        .map_err(
            |error| NetworkProvisioningError::SecurityGroupRuleConfigurationFailed {
                reason: extract_error_message(&error),
            },
        )?;
    Ok(())
}

async fn revoke_ingress(
    client: &AwsClient,
    security_group_id: &str,
    permissions: Vec<IpPermission>,
) -> Result<()> {
    client
        .ec2
        .revoke_security_group_ingress()
        .group_id(security_group_id)
        .set_ip_permissions(Some(permissions))
        .send()
        .await
        .map_err(
            |error| NetworkProvisioningError::SecurityGroupRuleConfigurationFailed {
                reason: extract_error_message(&error),
            },
        )?;
    Ok(())
}

fn build_desired_ingress_rules() -> HashSet<IngressRule> {
    [
        ("udp", ANY_IPV4_CIDR),
        ("tcp", ANY_IPV4_CIDR),
        ("udp", ANY_IPV6_CIDR),
        ("tcp", ANY_IPV6_CIDR),
    ]
    .into_iter()
    .map(|(protocol, cidr)| IngressRule {
        protocol: protocol.to_string(),
        port: i32::from(WIREGUARD_PORT),
        cidr: cidr.to_string(),
    })
    .collect()
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct IngressRule {
    protocol: String,
    port: i32,
    cidr: String,
}

impl IngressRule {
    fn from_ip_permission(permission: &IpPermission) -> Vec<Self> {
        let protocol = permission.ip_protocol().unwrap_or_default().to_string();
        let port = permission.from_port().unwrap_or(0);
        let ipv4_cidrs = permission
            .ip_ranges()
            .iter()
            .filter_map(|range| range.cidr_ip());
        let ipv6_cidrs = permission
            .ipv6_ranges()
            .iter()
            .filter_map(|range| range.cidr_ipv6());
        ipv4_cidrs
            .chain(ipv6_cidrs)
            .map(|cidr| Self {
                protocol: protocol.clone(),
                port,
                cidr: cidr.to_string(),
            })
            .collect()
    }

    fn build_ip_permission(&self) -> IpPermission {
        let builder = IpPermission::builder()
            .ip_protocol(&self.protocol)
            .from_port(self.port)
            .to_port(self.port);
        if self.cidr.contains(':') {
            builder
                .ipv6_ranges(Ipv6Range::builder().cidr_ipv6(&self.cidr).build())
                .build()
        } else {
            builder
                .ip_ranges(IpRange::builder().cidr_ip(&self.cidr).build())
                .build()
        }
    }
}
