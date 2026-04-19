use byocvpn_core::{
    error::{NetworkProvisioningError, Result},
    retry::retry,
};
use log::*;
use serde_json::Value;
use tokio::time::Duration;

use crate::{
    client::OciClient,
    models::{
        CreateInternetGatewayRequest, CreateSecurityListRequest, CreateSubnetRequest,
        CreateVcnRequest, EgressSecurityRule, Image, IngressSecurityRule, InternetGateway,
        PortRange, Region, RegionSubscription, RouteRule, RouteTableResponse, SecurityListResponse,
        SubnetResponse, SubscribeRegionRequest, TcpOptions, UdpOptions, UpdateRouteTableRequest,
        UpdateSecurityListRequest, UpdateSubnetSecurityListRequest, Vcn, byocvpn_tags,
    },
};

const VCN_DISPLAY_NAME: &str = "byocvpn-vcn";
const VCN_CIDR: &str = "10.0.0.0/16";
const SUBNET_DISPLAY_NAME: &str = "byocvpn-subnet";
const SUBNET_CIDR: &str = "10.0.0.0/24";
const SECURITY_LIST_NAME: &str = "byocvpn-security-list";
const INTERNET_GATEWAY_NAME: &str = "byocvpn-igw";
const IPV4_ALL_CIDR: &str = "0.0.0.0/0";
const IPV6_ALL_CIDR: &str = "::/0";

async fn get_vcn(
    client: &OciClient,
    compartment_ocid: &str,
) -> Result<Option<(String, String, String)>> {
    let url = client.build_core_url(&format!("/vcns?compartmentId={}&displayName={}", compartment_ocid, VCN_DISPLAY_NAME));
    let vcns: Vec<Vcn> = client.get(&url).await?;
    let vcn = vcns
        .into_iter()
        .find(|vcn| vcn.lifecycle_state == "AVAILABLE")
        .map(|vcn| -> Result<_> {
            let ipv6_prefix = vcn
                .ipv6_cidr_blocks
                .and_then(|blocks| blocks.into_iter().next())
                .ok_or(NetworkProvisioningError::VpcCreationFailed {
                    reason: "VCN has no IPv6 CIDR block".to_string(),
                })?;
            Ok((vcn.id, vcn.default_route_table_id, ipv6_prefix))
        })
        .transpose()?;
    Ok(vcn)
}

async fn create_vcn(
    client: &OciClient,
    compartment_ocid: &str,
) -> Result<(String, String, String)> {
    let url = client.build_core_url("/vcns");
    let body = CreateVcnRequest {
        compartment_id: compartment_ocid.to_string(),
        display_name: VCN_DISPLAY_NAME.to_string(),
        cidr_block: VCN_CIDR.to_string(),
        is_ipv6_enabled: true,
        freeform_tags: byocvpn_tags(),
    };
    let vcn: Vcn = client.post(&url, &body).await.map_err(|error| {
        NetworkProvisioningError::VpcCreationFailed {
            reason: error.to_string(),
        }
    })?;
    let ipv6_prefix = vcn
        .ipv6_cidr_blocks
        .and_then(|blocks| blocks.into_iter().next())
        .ok_or(NetworkProvisioningError::VpcCreationFailed {
            reason: "created VCN has no IPv6 CIDR block".to_string(),
        })?;
    Ok((vcn.id, vcn.default_route_table_id, ipv6_prefix))
}

pub async fn ensure_vcn(
    client: &OciClient,
    compartment_ocid: &str,
) -> Result<(String, String, String)> {
    if let Some(existing) = get_vcn(client, compartment_ocid).await? {
        return Ok(existing);
    }
    create_vcn(client, compartment_ocid).await
}

async fn get_internet_gateway(
    client: &OciClient,
    compartment_ocid: &str,
    vcn_id: &str,
) -> Result<Option<String>> {
    let url = client.build_core_url(&format!("/internetGateways?compartmentId={}&vcnId={}", compartment_ocid, vcn_id));
    let existing: Vec<InternetGateway> = client.get(&url).await?;
    Ok(existing.into_iter().next().map(|igw| igw.id))
}

async fn create_internet_gateway(
    client: &OciClient,
    compartment_ocid: &str,
    vcn_id: &str,
) -> Result<String> {
    let url = client.build_core_url("/internetGateways");
    let body = CreateInternetGatewayRequest {
        compartment_id: compartment_ocid.to_string(),
        vcn_id: vcn_id.to_string(),
        display_name: INTERNET_GATEWAY_NAME.to_string(),
        is_enabled: true,
        freeform_tags: byocvpn_tags(),
    };
    let igw: InternetGateway = client.post(&url, &body).await.map_err(|error| {
        NetworkProvisioningError::InternetGatewayOperationFailed {
            reason: error.to_string(),
        }
    })?;
    Ok(igw.id)
}

pub async fn ensure_internet_gateway(
    client: &OciClient,
    compartment_ocid: &str,
    vcn_id: &str,
) -> Result<String> {
    if let Some(igw_id) = get_internet_gateway(client, compartment_ocid, vcn_id).await? {
        return Ok(igw_id);
    }
    create_internet_gateway(client, compartment_ocid, vcn_id).await
}

pub async fn add_default_route_to_table(
    client: &OciClient,
    route_table_id: &str,
    igw_id: &str,
    ipv6_prefix: &str,
) -> Result<()> {
    let url = client.build_core_url(&format!("/routeTables/{}", route_table_id));

    let current: RouteTableResponse = client.get(&url).await?;
    let mut route_rules = current.route_rules;

    if !route_rules
        .iter()
        .any(|rule| rule.destination == IPV4_ALL_CIDR)
    {
        route_rules.push(RouteRule {
            network_entity_id: igw_id.to_string(),
            destination: IPV4_ALL_CIDR.to_string(),
            destination_type: "CIDR_BLOCK".to_string(),
        });
    }

    if !ipv6_prefix.is_empty()
        && !route_rules
            .iter()
            .any(|rule| rule.destination == IPV6_ALL_CIDR)
    {
        route_rules.push(RouteRule {
            network_entity_id: igw_id.to_string(),
            destination: IPV6_ALL_CIDR.to_string(),
            destination_type: "CIDR_BLOCK".to_string(),
        });
    }

    client
        .put(&url, &UpdateRouteTableRequest { route_rules })
        .await
        .map_err(|error| {
            NetworkProvisioningError::RouteTableOperationFailed {
                reason: error.to_string(),
            }
            .into()
        })
}

async fn get_security_list(
    client: &OciClient,
    compartment_ocid: &str,
    vcn_id: &str,
) -> Result<Option<SecurityListResponse>> {
    let url = client.build_core_url(&format!("/securityLists?compartmentId={}&vcnId={}&displayName={}", compartment_ocid, vcn_id, SECURITY_LIST_NAME));
    let existing: Vec<SecurityListResponse> = client.get(&url).await?;
    Ok(existing.into_iter().next())
}

fn security_list_contains_all_desired_rules(
    existing: &SecurityListResponse,
    desired_ingress: &[IngressSecurityRule],
    desired_egress: &[EgressSecurityRule],
) -> bool {
    let existing_ingress = existing.ingress_security_rules.as_deref().unwrap_or(&[]);
    let existing_egress = existing.egress_security_rules.as_deref().unwrap_or(&[]);
    desired_ingress.iter().all(|rule| existing_ingress.contains(rule))
        && desired_egress.iter().all(|rule| existing_egress.contains(rule))
}

async fn create_security_list(
    client: &OciClient,
    compartment_ocid: &str,
    vcn_id: &str,
    ipv6_prefix: &str,
) -> Result<String> {
    let create_url = client.build_core_url("/securityLists");
    let body = CreateSecurityListRequest {
        compartment_id: compartment_ocid.to_string(),
        vcn_id: vcn_id.to_string(),
        display_name: SECURITY_LIST_NAME.to_string(),
        freeform_tags: byocvpn_tags(),
        ingress_security_rules: build_ingress_rules(ipv6_prefix),
        egress_security_rules: build_egress_rules(ipv6_prefix),
    };
    let security_list: SecurityListResponse =
        client.post(&create_url, &body).await.map_err(|error| {
            NetworkProvisioningError::SecurityGroupCreationFailed {
                reason: error.to_string(),
            }
        })?;
    Ok(security_list.id)
}

pub async fn ensure_security_list(
    client: &OciClient,
    compartment_ocid: &str,
    vcn_id: &str,
    ipv6_prefix: &str,
) -> Result<String> {
    if let Some(existing) = get_security_list(client, compartment_ocid, vcn_id).await? {
        let desired_ingress = build_ingress_rules(ipv6_prefix);
        let desired_egress = build_egress_rules(ipv6_prefix);
        if !security_list_contains_all_desired_rules(&existing, &desired_ingress, &desired_egress) {
            let update_url = client.build_core_url(&format!("/securityLists/{}", existing.id));
            client
                .put(
                    &update_url,
                    &UpdateSecurityListRequest {
                        ingress_security_rules: desired_ingress,
                        egress_security_rules: desired_egress,
                    },
                )
                .await?;
        }
        return Ok(existing.id);
    }
    create_security_list(client, compartment_ocid, vcn_id, ipv6_prefix).await
}

fn build_ingress_rules(ipv6_prefix: &str) -> Vec<IngressSecurityRule> {
    let wireguard_port = PortRange {
        min: 51820,
        max: 51820,
    };
    let mut rules = vec![
        IngressSecurityRule {
            protocol: "17".to_string(),
            source: IPV4_ALL_CIDR.to_string(),
            source_type: "CIDR_BLOCK".to_string(),
            is_stateless: false,
            udp_options: Some(UdpOptions {
                destination_port_range: wireguard_port.clone(),
            }),
            tcp_options: None,
        },
        IngressSecurityRule {
            protocol: "6".to_string(),
            source: IPV4_ALL_CIDR.to_string(),
            source_type: "CIDR_BLOCK".to_string(),
            is_stateless: false,
            udp_options: None,
            tcp_options: Some(TcpOptions {
                destination_port_range: wireguard_port.clone(),
            }),
        },
    ];
    if !ipv6_prefix.is_empty() {
        rules.push(IngressSecurityRule {
            protocol: "17".to_string(),
            source: IPV6_ALL_CIDR.to_string(),
            source_type: "CIDR_BLOCK".to_string(),
            is_stateless: false,
            udp_options: Some(UdpOptions {
                destination_port_range: wireguard_port.clone(),
            }),
            tcp_options: None,
        });
        rules.push(IngressSecurityRule {
            protocol: "6".to_string(),
            source: IPV6_ALL_CIDR.to_string(),
            source_type: "CIDR_BLOCK".to_string(),
            is_stateless: false,
            udp_options: None,
            tcp_options: Some(TcpOptions {
                destination_port_range: wireguard_port,
            }),
        });
    }
    rules
}

fn build_egress_rules(ipv6_prefix: &str) -> Vec<EgressSecurityRule> {
    let mut rules = vec![EgressSecurityRule {
        protocol: "all".to_string(),
        destination: IPV4_ALL_CIDR.to_string(),
        destination_type: "CIDR_BLOCK".to_string(),
        is_stateless: false,
    }];
    if !ipv6_prefix.is_empty() {
        rules.push(EgressSecurityRule {
            protocol: "all".to_string(),
            destination: IPV6_ALL_CIDR.to_string(),
            destination_type: "CIDR_BLOCK".to_string(),
            is_stateless: false,
        });
    }
    rules
}

async fn get_subnet(
    client: &OciClient,
    compartment_ocid: &str,
    vcn_id: &str,
) -> Result<Option<(String, bool, Vec<String>)>> {
    let url = client.build_core_url(&format!("/subnets?compartmentId={}&vcnId={}&displayName={}", compartment_ocid, vcn_id, SUBNET_DISPLAY_NAME));
    let subnets: Vec<SubnetResponse> = client.get(&url).await?;
    let found = subnets
        .into_iter()
        .find(|subnet| {
            !matches!(
                subnet.lifecycle_state.as_str(),
                "TERMINATING" | "TERMINATED"
            )
        })
        .map(|subnet| {
            let has_ipv6 = subnet
                .ipv6_cidr_blocks
                .map(|blocks| !blocks.is_empty())
                .unwrap_or(false);
            let security_list_ids = subnet.security_list_ids.unwrap_or_default();
            (subnet.id, has_ipv6, security_list_ids)
        });
    Ok(found)
}

fn derive_first_subnet_ipv6_64(vcn_ipv6_prefix: &str) -> String {
    if let Some(slash_pos) = vcn_ipv6_prefix.rfind('/') {
        format!("{}/64", &vcn_ipv6_prefix[..slash_pos])
    } else {
        format!("{}/64", vcn_ipv6_prefix)
    }
}

async fn create_subnet(
    client: &OciClient,
    compartment_ocid: &str,
    vcn_id: &str,
    security_list_id: &str,
    route_table_id: &str,
    ipv6_prefix: &str,
) -> Result<String> {
    let url = client.build_core_url("/subnets");
    let body = CreateSubnetRequest {
        compartment_id: compartment_ocid.to_string(),
        vcn_id: vcn_id.to_string(),
        display_name: SUBNET_DISPLAY_NAME.to_string(),
        cidr_block: SUBNET_CIDR.to_string(),
        security_list_ids: vec![security_list_id.to_string()],
        route_table_id: route_table_id.to_string(),
        freeform_tags: byocvpn_tags(),
        ipv6_cidr_blocks: (!ipv6_prefix.is_empty())
            .then(|| vec![derive_first_subnet_ipv6_64(ipv6_prefix)]),
    };
    let subnet: SubnetResponse = client.post(&url, &body).await.map_err(|error| {
        NetworkProvisioningError::SubnetCreationFailed {
            reason: error.to_string(),
        }
    })?;
    Ok(subnet.id)
}

async fn ensure_subnet_security_list(
    client: &OciClient,
    subnet_id: &str,
    security_list_id: &str,
) -> Result<()> {
    let url = client.build_core_url(&format!("/subnets/{}", subnet_id));
    client
        .put(
            &url,
            &UpdateSubnetSecurityListRequest {
                security_list_ids: vec![security_list_id.to_string()],
            },
        )
        .await
}

pub async fn ensure_subnet(
    client: &OciClient,
    compartment_ocid: &str,
    vcn_id: &str,
    security_list_id: &str,
    route_table_id: &str,
    ipv6_prefix: &str,
) -> Result<String> {
    if let Some((existing_id, _, existing_security_list_ids)) = get_subnet(client, compartment_ocid, vcn_id).await? {
        if !existing_security_list_ids.contains(&security_list_id.to_string()) {
            ensure_subnet_security_list(client, &existing_id, security_list_id).await?;
        }
        return Ok(existing_id);
    }
    create_subnet(client, compartment_ocid, vcn_id, security_list_id, route_table_id, ipv6_prefix).await
}

pub async fn get_ubuntu_image(client: &OciClient, compartment_ocid: &str) -> Result<String> {
    let url = client.build_core_url(&format!("/images?compartmentId={}&operatingSystem=Canonical+Ubuntu&operatingSystemVersion=22.04&shape=VM.Standard.A1.Flex&lifecycleState=AVAILABLE", compartment_ocid));
    let images: Vec<Image> = client.get(&url).await?;
    images
        .into_iter()
        .next()
        .map(|image| image.id)
        .ok_or_else(|| {
            NetworkProvisioningError::BaseImageNotFound {
                image: "Ubuntu 22.04 ARM".to_string(),
            }
            .into()
        })
}

pub async fn ensure_region_subscribed(
    client: &OciClient,
    tenancy_ocid: &str,
    region_name: &str,
) -> Result<()> {
    let subscriptions_url = client.build_identity_url(&format!("/tenancies/{}/regionSubscriptions", tenancy_ocid));

    let subscriptions: Vec<RegionSubscription> =
        client.get(&subscriptions_url).await.map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: error.to_string(),
            }
        })?;
    let already_ready = subscriptions.iter().any(|subscription| {
        subscription.region_name == region_name && subscription.status == "READY"
    });
    if already_ready {
        return Ok(());
    }

    let all_regions_url = client.build_identity_url("/regions");
    let all_regions: Vec<Region> = client.get(&all_regions_url).await.map_err(|error| {
        NetworkProvisioningError::NetworkQueryFailed {
            reason: error.to_string(),
        }
    })?;
    let region_key = all_regions
        .into_iter()
        .find(|region| region.name == region_name)
        .map(|region| region.key)
        .ok_or_else(|| NetworkProvisioningError::ProviderSetupFailed {
            step: "region subscription".to_string(),
            reason: format!("region '{}' not found", region_name),
        })?;

    client
        .post::<_, Value>(
            &subscriptions_url,
            &SubscribeRegionRequest {
                region_key: region_key.clone(),
            },
        )
        .await
        .map_err(|error| NetworkProvisioningError::ProviderSetupFailed {
            step: format!("subscribe to region {}", region_name),
            reason: error.to_string(),
        })?;
    info!(
        "Subscribed to region {} (key {}), waiting for it to become READY...",
        region_name, region_key
    );

    retry(
        || {
            let url = subscriptions_url.clone();
            async move {
                let subscriptions: Vec<RegionSubscription> =
                    client.get(&url).await.map_err(|error| {
                        NetworkProvisioningError::NetworkQueryFailed {
                            reason: error.to_string(),
                        }
                    })?;
                let is_ready = subscriptions.iter().any(|subscription| {
                    subscription.region_name == region_name && subscription.status == "READY"
                });
                if is_ready {
                    info!("Region {} is now READY.", region_name);
                    Ok(())
                } else {
                    info!("Region {} not yet ready, retrying in 15 s...", region_name);
                    Err(NetworkProvisioningError::CloudOperationTimedOut {
                        operation: format!("region {} subscription", region_name),
                    }
                    .into())
                }
            }
        },
        40,
        Duration::from_secs(15),
    )
    .await
}

pub async fn list_all_regions(client: &OciClient) -> Result<Vec<(String, String)>> {
    let url = client.build_identity_url("/regions");
    let regions: Vec<Region> =
        client
            .get(&url)
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: error.to_string(),
            })?;
    let region_geography = [
        ("us-", "North America"),
        ("eu-", "Europe"),
        ("uk-", "Europe"),
        ("ap-", "Asia Pacific"),
        ("sa-", "South America"),
        ("ca-", "North America"),
        ("me-", "Middle East"),
        ("af-", "Africa"),
        ("il-", "Middle East"),
        ("mx-", "North America"),
    ];
    Ok(regions
        .into_iter()
        .map(|region| {
            let country = region_geography
                .iter()
                .find(|(prefix, _)| region.name.starts_with(prefix))
                .map(|(_, country)| country.to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            (region.name, country)
        })
        .collect())
}

pub async fn list_regions(client: &OciClient, tenancy_ocid: &str) -> Result<Vec<(String, String)>> {
    let url = client.build_identity_url(&format!("/tenancies/{}/regionSubscriptions", tenancy_ocid));
    let subscriptions: Vec<RegionSubscription> =
        client
            .get(&url)
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: error.to_string(),
            })?;
    let region_geography = [
        ("us-", "North America"),
        ("eu-", "Europe"),
        ("uk-", "Europe"),
        ("ap-", "Asia Pacific"),
        ("sa-", "South America"),
        ("ca-", "North America"),
        ("me-", "Middle East"),
        ("af-", "Africa"),
        ("il-", "Middle East"),
        ("mx-", "North America"),
    ];
    Ok(subscriptions
        .into_iter()
        .map(|subscription| {
            let country = region_geography
                .iter()
                .find(|(prefix, _)| subscription.region_name.starts_with(prefix))
                .map(|(_, country)| country.to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            (subscription.region_name, country)
        })
        .collect())
}
