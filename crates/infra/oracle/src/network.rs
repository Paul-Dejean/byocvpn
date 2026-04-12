use byocvpn_core::error::{NetworkProvisioningError, Result};
use serde_json::{Value, json};
use tokio::time::{Duration, sleep};

use crate::client::OciClient;
use log::*;

pub(crate) const VCN_DISPLAY_NAME: &str = "byocvpn-vcn";
const VCN_CIDR: &str = "10.0.0.0/16";
const SUBNET_DISPLAY_NAME: &str = "byocvpn-subnet";
const SUBNET_CIDR: &str = "10.0.0.0/24";
const SECURITY_LIST_NAME: &str = "byocvpn-security-list";
const INTERNET_GATEWAY_NAME: &str = "byocvpn-igw";
const IPV4_ALL_CIDR: &str = "0.0.0.0/0";
const IPV6_ALL_CIDR: &str = "::/0";

pub async fn get_vcn_by_name(
    client: &OciClient,
    compartment_ocid: &str,
) -> Result<Option<(String, String, String)>> {
    let url = format!(
        "{}/20160918/vcns?compartmentId={}&displayName={}",
        client.build_core_base_url(),
        compartment_ocid,
        VCN_DISPLAY_NAME
    );
    let response = client.get(&url).await?;
    let found = response
        .as_array()
        .and_then(|list| list.iter().find(|vcn| vcn["lifecycleState"] == "AVAILABLE"))
        .and_then(|vcn| {
            let id = vcn["id"].as_str()?.to_string();
            let default_route_table_id = vcn["defaultRouteTableId"].as_str()?.to_string();
            let ipv6_prefix = vcn["ipv6CidrBlocks"]
                .as_array()
                .and_then(|blocks| blocks.first())
                .and_then(|cidr_block| cidr_block.as_str())
                .unwrap_or_default()
                .to_string();
            Some((id, default_route_table_id, ipv6_prefix))
        });
    Ok(found)
}

pub async fn create_vcn(
    client: &OciClient,
    compartment_ocid: &str,
) -> Result<(String, String, String)> {
    let url = format!("{}/20160918/vcns", client.build_core_base_url());
    let body = json!({
        "compartmentId": compartment_ocid,
        "displayName": VCN_DISPLAY_NAME,
        "cidrBlock": VCN_CIDR,
        "isIpv6Enabled": true,
        "freeformTags": { "created-by": "byocvpn" },
    });
    let response = client.post(&url, &body).await.map_err(|error| {
        NetworkProvisioningError::VpcCreationFailed {
            reason: error.to_string(),
        }
    })?;
    let vcn_id = response["id"]
        .as_str()
        .ok_or(NetworkProvisioningError::MissingVpcIdentifier)?
        .to_string();
    let default_route_table_id = response["defaultRouteTableId"]
        .as_str()
        .ok_or(NetworkProvisioningError::MissingMainRouteTable {
            vpc_id: vcn_id.clone(),
        })?
        .to_string();
    let ipv6_prefix = response["ipv6CidrBlocks"]
        .as_array()
        .and_then(|blocks| blocks.first())
        .and_then(|cidr_block| cidr_block.as_str())
        .unwrap_or_default()
        .to_string();
    Ok((vcn_id, default_route_table_id, ipv6_prefix))
}

pub async fn ensure_vcn_ipv6(client: &OciClient, vcn_id: &str, current_prefix: &str) -> String {
    if !current_prefix.is_empty() {
        return current_prefix.to_string();
    }
    let url = format!(
        "{}/20160918/vcns/{}/actions/addIpv6VcnCidr",
        client.build_core_base_url(),
        vcn_id
    );
    if client
        .post(&url, &json!({ "isOracleGuaAllocationEnabled": true }))
        .await
        .is_err()
    {
        warn!(
            "[byocvpn] addIpv6VcnCidr failed for {} — will recreate VCN",
            vcn_id
        );
        return String::new();
    }

    let vcn_url = format!("{}/20160918/vcns/{}", client.build_core_base_url(), vcn_id);
    let Ok(vcn) = client.get(&vcn_url).await else {
        warn!("[Oracle] Failed to fetch VCN {} after adding IPv6 CIDR", vcn_id);
        return String::new();
    };
    vcn["ipv6CidrBlocks"]
        .as_array()
        .and_then(|blocks| blocks.first())
        .and_then(|cidr_block| cidr_block.as_str())
        .unwrap_or_default()
        .to_string()
}

pub async fn teardown_vcn_resources(
    client: &OciClient,
    compartment_ocid: &str,
    vcn_id: &str,
    route_table_id: &str,
) -> Result<()> {
    let rt_url = format!(
        "{}/20160918/routeTables/{}",
        client.build_core_base_url(),
        route_table_id
    );
    if let Err(error) = client.put(&rt_url, &json!({ "routeRules": [] })).await {
        warn!("[Oracle] Failed to clear route table rules for {}: {}", route_table_id, error);
    }

    let igw_list_url = format!(
        "{}/20160918/internetGateways?compartmentId={}&vcnId={}",
        client.build_core_base_url(),
        compartment_ocid,
        vcn_id
    );
    if let Ok(igws) = client.get(&igw_list_url).await {
        for igw in igws.as_array().cloned().unwrap_or_default() {
            if let Some(igw_id) = igw["id"].as_str() {
                let del = format!(
                    "{}/20160918/internetGateways/{}",
                    client.build_core_base_url(),
                    igw_id
                );
                if let Err(error) = client.delete(&del).await {
                    warn!("[Oracle] Failed to delete internet gateway {}: {}", igw_id, error);
                }
            }
        }
    }

    let subnet_list_url = format!(
        "{}/20160918/subnets?compartmentId={}&vcnId={}&displayName={}",
        client.build_core_base_url(),
        compartment_ocid,
        vcn_id,
        SUBNET_DISPLAY_NAME
    );
    if let Ok(subnets) = client.get(&subnet_list_url).await {
        for subnet in subnets.as_array().cloned().unwrap_or_default() {
            if let Some(subnet_id) = subnet["id"].as_str() {
                let del = format!(
                    "{}/20160918/subnets/{}",
                    client.build_core_base_url(),
                    subnet_id
                );
                if let Err(error) = client.delete(&del).await {
                    warn!("[Oracle] Failed to delete subnet {}: {}", subnet_id, error);
                }
            }
        }
    }

    let sl_list_url = format!(
        "{}/20160918/securityLists?compartmentId={}&vcnId={}&displayName={}",
        client.build_core_base_url(),
        compartment_ocid,
        vcn_id,
        SECURITY_LIST_NAME
    );
    if let Ok(sls) = client.get(&sl_list_url).await {
        for sl in sls.as_array().cloned().unwrap_or_default() {
            if let Some(sl_id) = sl["id"].as_str() {
                let del = format!(
                    "{}/20160918/securityLists/{}",
                    client.build_core_base_url(),
                    sl_id
                );
                if let Err(error) = client.delete(&del).await {
                    warn!("[Oracle] Failed to delete security list {}: {}", sl_id, error);
                }
            }
        }
    }

    let vcn_del_url = format!("{}/20160918/vcns/{}", client.build_core_base_url(), vcn_id);
    client.delete(&vcn_del_url).await?;

    Ok(())
}

pub async fn get_or_create_internet_gateway(
    client: &OciClient,
    compartment_ocid: &str,
    vcn_id: &str,
) -> Result<String> {
    let url = format!(
        "{}/20160918/internetGateways?compartmentId={}&vcnId={}",
        client.build_core_base_url(),
        compartment_ocid,
        vcn_id
    );
    let response = client.get(&url).await?;
    if let Some(existing_id) = response
        .as_array()
        .and_then(|list| list.first())
        .and_then(|existing_gateway| existing_gateway["id"].as_str())
    {
        return Ok(existing_id.to_string());
    }

    let url = format!("{}/20160918/internetGateways", client.build_core_base_url());
    let body = json!({
        "compartmentId": compartment_ocid,
        "vcnId": vcn_id,
        "displayName": INTERNET_GATEWAY_NAME,
        "isEnabled": true,
        "freeformTags": { "created-by": "byocvpn" },
    });
    let response = client.post(&url, &body).await.map_err(|error| {
        NetworkProvisioningError::InternetGatewayOperationFailed {
            reason: error.to_string(),
        }
    })?;
    Ok(response["id"]
        .as_str()
        .ok_or(NetworkProvisioningError::MissingInternetGatewayIdentifier)?
        .to_string())
}

pub async fn add_default_route_to_table(
    client: &OciClient,
    route_table_id: &str,
    igw_id: &str,
    ipv6_prefix: &str,
) -> Result<()> {
    let url = format!(
        "{}/20160918/routeTables/{}",
        client.build_core_base_url(),
        route_table_id
    );

    let current = client.get(&url).await?;
    let mut route_rules: Vec<Value> = current["routeRules"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let already_has_default = route_rules
        .iter()
        .any(|route_rule| route_rule["destination"].as_str() == Some(IPV4_ALL_CIDR));
    if !already_has_default {
        route_rules.push(json!({
            "networkEntityId": igw_id,
            "destination": IPV4_ALL_CIDR,
            "destinationType": "CIDR_BLOCK",
        }));
    }

    if !ipv6_prefix.is_empty() {
        let already_has_default_ipv6 = route_rules
            .iter()
            .any(|route_rule| route_rule["destination"].as_str() == Some(IPV6_ALL_CIDR));
        if !already_has_default_ipv6 {
            route_rules.push(json!({
                "networkEntityId": igw_id,
                "destination": IPV6_ALL_CIDR,
                "destinationType": "CIDR_BLOCK",
            }));
        }
    }

    let update_url = format!(
        "{}/20160918/routeTables/{}",
        client.build_core_base_url(),
        route_table_id
    );
    let body = json!({ "routeRules": route_rules });

    client.put(&update_url, &body).await.map_err(|error| {
        NetworkProvisioningError::RouteTableOperationFailed {
            reason: error.to_string(),
        }
    })?;
    Ok(())
}

pub async fn get_or_create_security_list(
    client: &OciClient,
    compartment_ocid: &str,
    vcn_id: &str,
    ipv6_prefix: &str,
) -> Result<String> {
    let url = format!(
        "{}/20160918/securityLists?compartmentId={}&vcnId={}&displayName={}",
        client.build_core_base_url(),
        compartment_ocid,
        vcn_id,
        SECURITY_LIST_NAME
    );
    let response = client.get(&url).await?;

    let mut ingress_rules = vec![
        json!({
            "protocol": "17",
            "source": IPV4_ALL_CIDR,
            "sourceType": "CIDR_BLOCK",
            "isStateless": false,
            "udpOptions": { "destinationPortRange": { "min": 51820, "max": 51820 } }
        }),
        json!({
            "protocol": "6",
            "source": IPV4_ALL_CIDR,
            "sourceType": "CIDR_BLOCK",
            "isStateless": false,
            "tcpOptions": { "destinationPortRange": { "min": 51820, "max": 51820 } }
        }),
    ];
    let mut egress_rules = vec![json!({
        "protocol": "all",
        "destination": IPV4_ALL_CIDR,
        "destinationType": "CIDR_BLOCK",
        "isStateless": false,
    })];
    if !ipv6_prefix.is_empty() {
        ingress_rules.push(json!({
            "protocol": "17",
            "source": IPV6_ALL_CIDR,
            "sourceType": "CIDR_BLOCK",
            "isStateless": false,
            "udpOptions": { "destinationPortRange": { "min": 51820, "max": 51820 } }
        }));

        ingress_rules.push(json!({
            "protocol": "6",
            "source": IPV6_ALL_CIDR,
            "sourceType": "CIDR_BLOCK",
            "isStateless": false,
            "tcpOptions": { "destinationPortRange": { "min": 51820, "max": 51820 } }
        }));
        egress_rules.push(json!({
            "protocol": "all",
            "destination": IPV6_ALL_CIDR,
            "destinationType": "CIDR_BLOCK",
            "isStateless": false,
        }));
    }
    let required_ingress = json!(ingress_rules);
    let required_egress = json!(egress_rules);

    if let Some(existing) = response.as_array().and_then(|list| list.first()) {
        let existing_id = existing["id"]
            .as_str()
            .ok_or(NetworkProvisioningError::MissingSecurityGroupIdentifier)?
            .to_string();

        let update_url = format!(
            "{}/20160918/securityLists/{}",
            client.build_core_base_url(),
            existing_id
        );
        client
            .put(
                &update_url,
                &json!({
                    "ingressSecurityRules": required_ingress,
                    "egressSecurityRules": required_egress,
                }),
            )
            .await?;
        return Ok(existing_id);
    }

    let create_url = format!("{}/20160918/securityLists", client.build_core_base_url());
    let body = json!({
        "compartmentId": compartment_ocid,
        "vcnId": vcn_id,
        "displayName": SECURITY_LIST_NAME,
        "freeformTags": { "created-by": "byocvpn" },
        "ingressSecurityRules": required_ingress,
        "egressSecurityRules": required_egress,
    });
    let response = client.post(&create_url, &body).await.map_err(|error| {
        NetworkProvisioningError::SecurityGroupCreationFailed {
            reason: error.to_string(),
        }
    })?;
    Ok(response["id"]
        .as_str()
        .ok_or(NetworkProvisioningError::MissingSecurityGroupIdentifier)?
        .to_string())
}

pub async fn get_subnet_by_name(
    client: &OciClient,
    compartment_ocid: &str,
    vcn_id: &str,
) -> Result<Option<(String, bool)>> {
    let url = format!(
        "{}/20160918/subnets?compartmentId={}&vcnId={}&displayName={}",
        client.build_core_base_url(),
        compartment_ocid,
        vcn_id,
        SUBNET_DISPLAY_NAME
    );
    let response = client.get(&url).await?;

    Ok(response
        .as_array()
        .and_then(|list| {
            list.iter().find(|subnet| {
                !matches!(
                    subnet["lifecycleState"].as_str(),
                    Some("TERMINATING") | Some("TERMINATED") | None
                )
            })
        })
        .and_then(|subnet| {
            let id = subnet["id"].as_str()?.to_string();
            let has_ipv6 = subnet["ipv6CidrBlocks"]
                .as_array()
                .map(|blocks| !blocks.is_empty())
                .unwrap_or(false);
            Some((id, has_ipv6))
        }))
}

fn derive_first_subnet_ipv6_64(vcn_ipv6_prefix: &str) -> String {
    if let Some(slash_pos) = vcn_ipv6_prefix.rfind('/') {
        format!("{}/64", &vcn_ipv6_prefix[..slash_pos])
    } else {
        format!("{}/64", vcn_ipv6_prefix)
    }
}

pub async fn create_subnet(
    client: &OciClient,
    compartment_ocid: &str,
    vcn_id: &str,
    security_list_id: &str,
    route_table_id: &str,
    ipv6_prefix: &str,
) -> Result<String> {
    let url = format!("{}/20160918/subnets", client.build_core_base_url());

    let mut body = json!({
        "compartmentId": compartment_ocid,
        "vcnId": vcn_id,
        "displayName": SUBNET_DISPLAY_NAME,
        "cidrBlock": SUBNET_CIDR,
        "securityListIds": [security_list_id],
        "routeTableId": route_table_id,
        "freeformTags": { "created-by": "byocvpn" },
    });
    if !ipv6_prefix.is_empty() {
        body["ipv6CidrBlocks"] = json!([derive_first_subnet_ipv6_64(ipv6_prefix)]);
    }
    let response = client.post(&url, &body).await.map_err(|error| {
        NetworkProvisioningError::SubnetCreationFailed {
            reason: error.to_string(),
        }
    })?;
    Ok(response["id"]
        .as_str()
        .ok_or(NetworkProvisioningError::MissingSubnetIdentifier)?
        .to_string())
}

pub async fn ensure_subnet_security_list(
    client: &OciClient,
    subnet_id: &str,
    security_list_id: &str,
) -> Result<()> {
    let url = format!(
        "{}/20160918/subnets/{}",
        client.build_core_base_url(),
        subnet_id
    );
    client
        .put(&url, &json!({ "securityListIds": [security_list_id] }))
        .await?;
    Ok(())
}

pub async fn get_ubuntu_image(client: &OciClient, compartment_ocid: &str) -> Result<String> {
    let url = format!(
        "{}/20160918/images?compartmentId={}&operatingSystem=Canonical+Ubuntu&operatingSystemVersion=22.04&shape={}&lifecycleState=AVAILABLE",
        client.build_core_base_url(),
        compartment_ocid,
        "VM.Standard.A1.Flex",
    );
    let response = client.get(&url).await?;
    response
        .as_array()
        .and_then(|list| list.first())
        .and_then(|image| image["id"].as_str())
        .map(|image_id| image_id.to_string())
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
    let subscriptions_url = format!(
        "{}/20160918/tenancies/{}/regionSubscriptions",
        client.build_identity_base_url(),
        tenancy_ocid
    );

    let subscriptions = client.get(&subscriptions_url).await.map_err(|error| {
        NetworkProvisioningError::NetworkQueryFailed {
            reason: error.to_string(),
        }
    })?;
    let already_ready = subscriptions
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .any(|subscription| {
            subscription["regionName"].as_str() == Some(region_name)
                && subscription["status"].as_str() == Some("READY")
        });
    if already_ready {
        return Ok(());
    }

    let all_regions_url = format!("{}/20160918/regions", client.build_identity_base_url());
    let all_regions = client.get(&all_regions_url).await.map_err(|error| {
        NetworkProvisioningError::NetworkQueryFailed {
            reason: error.to_string(),
        }
    })?;
    let region_key = all_regions
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .find_map(|region| {
            if region["name"].as_str() == Some(region_name) {
                region["key"]
                    .as_str()
                    .map(|region_key| region_key.to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| NetworkProvisioningError::ProviderSetupFailed {
            step: "region subscription".to_string(),
            reason: format!("region '{}' not found", region_name),
        })?;

    let body = json!({ "regionKey": region_key });
    client
        .post(&subscriptions_url, &body)
        .await
        .map_err(|error| NetworkProvisioningError::ProviderSetupFailed {
            step: format!("subscribe to region {}", region_name),
            reason: error.to_string(),
        })?;
    info!(
        "Subscribed to region {} (key {}), waiting for it to become READY...",
        region_name, region_key
    );

    for attempt in 1..=40u32 {
        sleep(Duration::from_secs(15)).await;
        let subs = client.get(&subscriptions_url).await.map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: error.to_string(),
            }
        })?;
        let is_ready = subs
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .any(|subscription| {
                subscription["regionName"].as_str() == Some(region_name)
                    && subscription["status"].as_str() == Some("READY")
            });
        if is_ready {
            info!(
                "Region {} is now READY (attempt {}/40).",
                region_name, attempt
            );
            return Ok(());
        }
        info!(
            "Region {} not yet ready (attempt {}/40), retrying in 15 s...",
            region_name, attempt
        );
    }

    Err(NetworkProvisioningError::CloudOperationTimedOut {
        operation: format!("region {} subscription", region_name),
    }
    .into())
}

pub async fn list_all_regions(client: &OciClient) -> Result<Vec<(String, String)>> {
    let url = format!("{}/20160918/regions", client.build_identity_base_url());
    let response =
        client
            .get(&url)
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: error.to_string(),
            })?;
    let region_geography = [
        ("us-", "United States"),
        ("eu-", "Europe"),
        ("ap-", "Asia Pacific"),
        ("sa-", "South America"),
        ("ca-", "Canada"),
        ("me-", "Middle East"),
        ("af-", "Africa"),
        ("il-", "Israel"),
        ("mx-", "Mexico"),
    ];
    let regions = response
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|region| {
            let name = region["name"].as_str()?.to_string();
            let country = region_geography
                .iter()
                .find(|(prefix, _)| name.starts_with(prefix))
                .map(|(_, country)| country.to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            Some((name, country))
        })
        .collect();
    Ok(regions)
}

pub async fn list_regions(client: &OciClient, tenancy_ocid: &str) -> Result<Vec<(String, String)>> {
    let url = format!(
        "{}/20160918/tenancies/{}/regionSubscriptions",
        client.build_identity_base_url(),
        tenancy_ocid
    );
    let response =
        client
            .get(&url)
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: error.to_string(),
            })?;
    let region_geography = [
        ("us-", "United States"),
        ("eu-", "Europe"),
        ("ap-", "Asia Pacific"),
        ("sa-", "South America"),
        ("ca-", "Canada"),
        ("me-", "Middle East"),
        ("af-", "Africa"),
        ("il-", "Israel"),
        ("mx-", "Mexico"),
    ];
    let regions = response
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|region| {
            let name = region["regionName"].as_str()?.to_string();
            let country = region_geography
                .iter()
                .find(|(prefix, _)| name.starts_with(prefix))
                .map(|(_, country)| country.to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            Some((name, country))
        })
        .collect();
    Ok(regions)
}
