use byocvpn_core::{
    error::{NetworkProvisioningError, Result},
    managed_resources::{ANY_IPV4_CIDR, ANY_IPV6_CIDR, WIREGUARD_PORT},
};
use log::info;

use crate::{
    client::OciClient,
    models::{
        CreateSecurityListRequest, EgressSecurityRule, IngressSecurityRule, PortRange,
        SecurityListResponse, TcpOptions, UdpOptions, UpdateSecurityListRequest,
        build_managed_tags,
    },
    network::vcn::Vcn,
};

const SECURITY_LIST_NAME: &str = "byocvpn-security-list";
const CIDR_BLOCK_SOURCE_TYPE: &str = "CIDR_BLOCK";
const UDP_PROTOCOL_NUMBER: &str = "17";
const TCP_PROTOCOL_NUMBER: &str = "6";
const ALL_PROTOCOLS: &str = "all";

pub async fn ensure_security_list(client: &OciClient, vcn: &Vcn) -> Result<String> {
    let desired_ingress = build_ingress_rules(vcn);
    let desired_egress = build_egress_rules(vcn);

    match get_security_list(client, vcn).await? {
        Some(existing) => {
            if !contains_all_rules(&existing, &desired_ingress, &desired_egress) {
                info!(
                    "[OCI] Security list {} has drifted, updating...",
                    existing.id
                );
                let url = client.build_core_url(&format!("/securityLists/{}", existing.id));
                let body = UpdateSecurityListRequest {
                    ingress_security_rules: desired_ingress,
                    egress_security_rules: desired_egress,
                };
                client.put(&url, &body).await?;
            }
            Ok(existing.id)
        }
        None => create_security_list(client, vcn, desired_ingress, desired_egress).await,
    }
}

async fn get_security_list(client: &OciClient, vcn: &Vcn) -> Result<Option<SecurityListResponse>> {
    let url = client.build_core_url(&format!(
        "/securityLists?compartmentId={}&vcnId={}&displayName={SECURITY_LIST_NAME}",
        client.get_compartment_ocid(),
        vcn.id
    ));
    let security_lists: Vec<SecurityListResponse> = client.get(&url).await?;
    Ok(security_lists.into_iter().next())
}

async fn create_security_list(
    client: &OciClient,
    vcn: &Vcn,
    ingress_security_rules: Vec<IngressSecurityRule>,
    egress_security_rules: Vec<EgressSecurityRule>,
) -> Result<String> {
    let body = CreateSecurityListRequest {
        compartment_id: client.get_compartment_ocid().to_string(),
        vcn_id: vcn.id.clone(),
        display_name: SECURITY_LIST_NAME.to_string(),
        freeform_tags: build_managed_tags(),
        ingress_security_rules,
        egress_security_rules,
    };
    let security_list: SecurityListResponse = client
        .post(&client.build_core_url("/securityLists"), &body)
        .await
        .map_err(
            |error| NetworkProvisioningError::SecurityGroupCreationFailed {
                reason: error.to_string(),
            },
        )?;
    info!("[OCI] Created security list {}", security_list.id);
    Ok(security_list.id)
}

fn contains_all_rules(
    existing: &SecurityListResponse,
    desired_ingress: &[IngressSecurityRule],
    desired_egress: &[EgressSecurityRule],
) -> bool {
    let existing_ingress = existing.ingress_security_rules.as_deref().unwrap_or(&[]);
    let existing_egress = existing.egress_security_rules.as_deref().unwrap_or(&[]);
    desired_ingress
        .iter()
        .all(|rule| existing_ingress.contains(rule))
        && desired_egress
            .iter()
            .all(|rule| existing_egress.contains(rule))
}

fn build_ingress_rules(vcn: &Vcn) -> Vec<IngressSecurityRule> {
    build_source_cidrs(vcn)
        .into_iter()
        .flat_map(|source| {
            [
                build_udp_ingress_rule(source),
                build_tcp_ingress_rule(source),
            ]
        })
        .collect()
}

fn build_egress_rules(vcn: &Vcn) -> Vec<EgressSecurityRule> {
    build_source_cidrs(vcn)
        .into_iter()
        .map(|destination| EgressSecurityRule {
            protocol: ALL_PROTOCOLS.to_string(),
            destination: destination.to_string(),
            destination_type: CIDR_BLOCK_SOURCE_TYPE.to_string(),
            is_stateless: false,
        })
        .collect()
}

fn build_source_cidrs(vcn: &Vcn) -> Vec<&'static str> {
    if vcn.has_ipv6() {
        vec![ANY_IPV4_CIDR, ANY_IPV6_CIDR]
    } else {
        vec![ANY_IPV4_CIDR]
    }
}

fn build_udp_ingress_rule(source: &str) -> IngressSecurityRule {
    IngressSecurityRule {
        protocol: UDP_PROTOCOL_NUMBER.to_string(),
        source: source.to_string(),
        source_type: CIDR_BLOCK_SOURCE_TYPE.to_string(),
        is_stateless: false,
        udp_options: Some(UdpOptions {
            destination_port_range: build_wireguard_port_range(),
        }),
        tcp_options: None,
    }
}

fn build_tcp_ingress_rule(source: &str) -> IngressSecurityRule {
    IngressSecurityRule {
        protocol: TCP_PROTOCOL_NUMBER.to_string(),
        source: source.to_string(),
        source_type: CIDR_BLOCK_SOURCE_TYPE.to_string(),
        is_stateless: false,
        udp_options: None,
        tcp_options: Some(TcpOptions {
            destination_port_range: build_wireguard_port_range(),
        }),
    }
}

const fn build_wireguard_port_range() -> PortRange {
    PortRange {
        min: WIREGUARD_PORT,
        max: WIREGUARD_PORT,
    }
}
