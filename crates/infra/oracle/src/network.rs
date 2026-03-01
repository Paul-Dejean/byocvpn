use byocvpn_core::error::{NetworkProvisioningError, Result};
use serde_json::{Value, json};
use tokio::time::{Duration, sleep};

use crate::client::OciClient;

const VCN_DISPLAY_NAME: &str = "byocvpn-vcn";
const VCN_CIDR: &str = "10.0.0.0/16";
const SUBNET_DISPLAY_NAME: &str = "byocvpn-subnet";
const SUBNET_CIDR: &str = "10.0.0.0/24";
const SECURITY_LIST_NAME: &str = "byocvpn-security-list";
const INTERNET_GATEWAY_NAME: &str = "byocvpn-igw";

// ---------------------------------------------------------------------------
// VCN
// ---------------------------------------------------------------------------

pub async fn get_vcn_by_name(
    client: &OciClient,
    compartment_ocid: &str,
) -> Result<Option<(String, String, String)>> {
    let url = format!(
        "{}/20160918/vcns?compartmentId={}&displayName={}",
        client.core_base_url(),
        compartment_ocid,
        VCN_DISPLAY_NAME
    );
    let response = client.get(&url).await?;
    let found = response
        .as_array()
        .and_then(|list| list.iter().find(|v| v["lifecycleState"] == "AVAILABLE"))
        .and_then(|v| {
            let id = v["id"].as_str()?.to_string();
            let default_route_table_id = v["defaultRouteTableId"].as_str()?.to_string();
            let ipv6_prefix = v["ipv6CidrBlocks"]
                .as_array()
                .and_then(|blocks| blocks.first())
                .and_then(|b| b.as_str())
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
    let url = format!("{}/20160918/vcns", client.core_base_url());
    let body = json!({
        "compartmentId": compartment_ocid,
        "displayName": VCN_DISPLAY_NAME,
        "cidrBlock": VCN_CIDR,
        "isIpv6Enabled": true,
        "freeformTags": { "byocvpn": "true" },
    });
    let response = client.post(&url, &body).await.map_err(|e| {
        NetworkProvisioningError::VpcCreationFailed {
            reason: e.to_string(),
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
        .and_then(|b| b.as_str())
        .unwrap_or_default()
        .to_string();
    Ok((vcn_id, default_route_table_id, ipv6_prefix))
}

/// Attempt to add an Oracle-allocated IPv6 /56 prefix to a VCN that lacks one.
///
/// Returns the prefix on success, or an empty string if OCI rejects the request
/// (e.g. the region doesn't support Oracle-provided GUA, or the IAM policy
/// doesn't allow it).  The caller can then decide to tear down and recreate
/// the VCN with `isIpv6Enabled: true`.
pub async fn ensure_vcn_ipv6(client: &OciClient, vcn_id: &str, current_prefix: &str) -> String {
    if !current_prefix.is_empty() {
        return current_prefix.to_string();
    }
    let url = format!(
        "{}/20160918/vcns/{}/actions/addIpv6VcnCidr",
        client.core_base_url(),
        vcn_id
    );
    if client
        .post(&url, &json!({ "isOracleGuaAllocationEnabled": true }))
        .await
        .is_err()
    {
        eprintln!(
            "[byocvpn] addIpv6VcnCidr failed for {} — will recreate VCN",
            vcn_id
        );
        return String::new();
    }
    // Re-fetch the VCN to get the newly assigned prefix.
    let vcn_url = format!("{}/20160918/vcns/{}", client.core_base_url(), vcn_id);
    let Ok(vcn) = client.get(&vcn_url).await else {
        return String::new();
    };
    vcn["ipv6CidrBlocks"]
        .as_array()
        .and_then(|blocks| blocks.first())
        .and_then(|b| b.as_str())
        .unwrap_or_default()
        .to_string()
}

/// Tear down all byocvpn-managed resources inside a VCN so the VCN itself can
/// be deleted.  This is called when a legacy (non-IPv6) VCN needs to be
/// replaced.  Safe to call when no instances are running in the region.
pub async fn teardown_vcn_resources(
    client: &OciClient,
    compartment_ocid: &str,
    vcn_id: &str,
    route_table_id: &str,
) -> Result<()> {
    // 1. Clear all route rules first (required before IGW can be deleted).
    let rt_url = format!(
        "{}/20160918/routeTables/{}",
        client.core_base_url(),
        route_table_id
    );
    client.put(&rt_url, &json!({ "routeRules": [] })).await.ok();

    // 2. Delete all internet gateways in this VCN.
    let igw_list_url = format!(
        "{}/20160918/internetGateways?compartmentId={}&vcnId={}",
        client.core_base_url(),
        compartment_ocid,
        vcn_id
    );
    if let Ok(igws) = client.get(&igw_list_url).await {
        for igw in igws.as_array().cloned().unwrap_or_default() {
            if let Some(igw_id) = igw["id"].as_str() {
                let del = format!(
                    "{}/20160918/internetGateways/{}",
                    client.core_base_url(),
                    igw_id
                );
                client.delete(&del).await.ok();
            }
        }
    }

    // 3. Delete byocvpn subnets.
    let subnet_list_url = format!(
        "{}/20160918/subnets?compartmentId={}&vcnId={}&displayName={}",
        client.core_base_url(),
        compartment_ocid,
        vcn_id,
        SUBNET_DISPLAY_NAME
    );
    if let Ok(subnets) = client.get(&subnet_list_url).await {
        for subnet in subnets.as_array().cloned().unwrap_or_default() {
            if let Some(subnet_id) = subnet["id"].as_str() {
                let del = format!("{}/20160918/subnets/{}", client.core_base_url(), subnet_id);
                client.delete(&del).await.ok();
            }
        }
    }

    // 4. Delete byocvpn security list.
    let sl_list_url = format!(
        "{}/20160918/securityLists?compartmentId={}&vcnId={}&displayName={}",
        client.core_base_url(),
        compartment_ocid,
        vcn_id,
        SECURITY_LIST_NAME
    );
    if let Ok(sls) = client.get(&sl_list_url).await {
        for sl in sls.as_array().cloned().unwrap_or_default() {
            if let Some(sl_id) = sl["id"].as_str() {
                let del = format!(
                    "{}/20160918/securityLists/{}",
                    client.core_base_url(),
                    sl_id
                );
                client.delete(&del).await.ok();
            }
        }
    }

    // 5. Delete the VCN itself (default route table + security list are cleaned up by OCI).
    let vcn_del_url = format!("{}/20160918/vcns/{}", client.core_base_url(), vcn_id);
    client.delete(&vcn_del_url).await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Internet Gateway
// ---------------------------------------------------------------------------

pub async fn get_or_create_internet_gateway(
    client: &OciClient,
    compartment_ocid: &str,
    vcn_id: &str,
) -> Result<String> {
    // Check if one already exists
    let url = format!(
        "{}/20160918/internetGateways?compartmentId={}&vcnId={}",
        client.core_base_url(),
        compartment_ocid,
        vcn_id
    );
    let response = client.get(&url).await?;
    if let Some(existing_id) = response
        .as_array()
        .and_then(|list| list.first())
        .and_then(|v| v["id"].as_str())
    {
        return Ok(existing_id.to_string());
    }

    let url = format!("{}/20160918/internetGateways", client.core_base_url());
    let body = json!({
        "compartmentId": compartment_ocid,
        "vcnId": vcn_id,
        "displayName": INTERNET_GATEWAY_NAME,
        "isEnabled": true,
        "freeformTags": { "byocvpn": "true" },
    });
    let response = client.post(&url, &body).await.map_err(|e| {
        NetworkProvisioningError::InternetGatewayOperationFailed {
            reason: e.to_string(),
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
        client.core_base_url(),
        route_table_id
    );
    // GET current rules first so we can append
    let current = client.get(&url).await?;
    let mut route_rules: Vec<Value> = current["routeRules"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    // Add default IPv4 route if not already present
    let already_has_default = route_rules
        .iter()
        .any(|r| r["destination"].as_str() == Some("0.0.0.0/0"));
    if !already_has_default {
        route_rules.push(json!({
            "networkEntityId": igw_id,
            "destination": "0.0.0.0/0",
            "destinationType": "CIDR_BLOCK",
        }));
    }

    // Add default IPv6 route only when the VCN has IPv6 enabled.
    // OCI returns 400 InvalidParameter if we try to add a ::/0 rule to a
    // non-IPv6 VCN (e.g. one created before isIpv6Enabled was set).
    if !ipv6_prefix.is_empty() {
        let already_has_default_ipv6 = route_rules
            .iter()
            .any(|r| r["destination"].as_str() == Some("::/0"));
        if !already_has_default_ipv6 {
            route_rules.push(json!({
                "networkEntityId": igw_id,
                "destination": "::/0",
                "destinationType": "CIDR_BLOCK",
            }));
        }
    }

    let update_url = format!(
        "{}/20160918/routeTables/{}",
        client.core_base_url(),
        route_table_id
    );
    let body = json!({ "routeRules": route_rules });
    // OCI UpdateRouteTable is a PUT, not POST.
    client.put(&update_url, &body).await.map_err(|e| {
        NetworkProvisioningError::RouteTableOperationFailed {
            reason: e.to_string(),
        }
    })?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Security List (allows WireGuard UDP 51820 in/out)
// ---------------------------------------------------------------------------

pub async fn get_or_create_security_list(
    client: &OciClient,
    compartment_ocid: &str,
    vcn_id: &str,
    ipv6_prefix: &str,
) -> Result<String> {
    let url = format!(
        "{}/20160918/securityLists?compartmentId={}&vcnId={}&displayName={}",
        client.core_base_url(),
        compartment_ocid,
        vcn_id,
        SECURITY_LIST_NAME
    );
    let response = client.get(&url).await?;

    // OCI rejects ::/0 rules on VCNs that were created without isIpv6Enabled.
    let mut ingress_rules = vec![json!({
        "protocol": "17",
        "source": "0.0.0.0/0",
        "sourceType": "CIDR_BLOCK",
        "isStateless": false,
        "udpOptions": { "destinationPortRange": { "min": 51820, "max": 51820 } }
    })];
    let mut egress_rules = vec![json!({
        "protocol": "all",
        "destination": "0.0.0.0/0",
        "destinationType": "CIDR_BLOCK",
        "isStateless": false,
    })];
    if !ipv6_prefix.is_empty() {
        ingress_rules.push(json!({
            "protocol": "17",
            "source": "::/0",
            "sourceType": "CIDR_BLOCK",
            "isStateless": false,
            "udpOptions": { "destinationPortRange": { "min": 51820, "max": 51820 } }
        }));
        egress_rules.push(json!({
            "protocol": "all",
            "destination": "::/0",
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
        // Always PUT the required rules — the list may have existed without them.
        let update_url = format!(
            "{}/20160918/securityLists/{}",
            client.core_base_url(),
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

    let create_url = format!("{}/20160918/securityLists", client.core_base_url());
    let body = json!({
        "compartmentId": compartment_ocid,
        "vcnId": vcn_id,
        "displayName": SECURITY_LIST_NAME,
        "freeformTags": { "byocvpn": "true" },
        "ingressSecurityRules": required_ingress,
        "egressSecurityRules": required_egress,
    });
    let response = client.post(&create_url, &body).await.map_err(|e| {
        NetworkProvisioningError::SecurityGroupCreationFailed {
            reason: e.to_string(),
        }
    })?;
    Ok(response["id"]
        .as_str()
        .ok_or(NetworkProvisioningError::MissingSecurityGroupIdentifier)?
        .to_string())
}

// ---------------------------------------------------------------------------
// Subnet
// ---------------------------------------------------------------------------

pub async fn get_subnet_by_name(
    client: &OciClient,
    compartment_ocid: &str,
    vcn_id: &str,
) -> Result<Option<(String, bool)>> {
    let url = format!(
        "{}/20160918/subnets?compartmentId={}&vcnId={}&displayName={}",
        client.core_base_url(),
        compartment_ocid,
        vcn_id,
        SUBNET_DISPLAY_NAME
    );
    let response = client.get(&url).await?;
    // Accept any state except TERMINATING/TERMINATED — a subnet in PROVISIONING
    // is still usable for instance launches; OCI will retry internally if needed.
    Ok(response
        .as_array()
        .and_then(|list| {
            list.iter().find(|v| {
                !matches!(
                    v["lifecycleState"].as_str(),
                    Some("TERMINATING") | Some("TERMINATED") | None
                )
            })
        })
        .and_then(|v| {
            let id = v["id"].as_str()?.to_string();
            let has_ipv6 = v["ipv6CidrBlocks"]
                .as_array()
                .map(|blocks| !blocks.is_empty())
                .unwrap_or(false);
            Some((id, has_ipv6))
        }))
}

/// Derive the first /64 subnet CIDR from a VCN IPv6 prefix (typically /56).
///
/// OCI VCNs get an Oracle-assigned /56 such as `2603:c020:4000::/56`.
/// The first /64 within it uses the same network address: `2603:c020:4000::/64`.
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
    let url = format!("{}/20160918/subnets", client.core_base_url());
    // Omitting `availabilityDomain` creates a regional subnet that spans all ADs.
    // This is the modern OCI pattern and works uniformly across all regions,
    // including newer ones where the availabilityDomains API may behave differently.
    let mut body = json!({
        "compartmentId": compartment_ocid,
        "vcnId": vcn_id,
        "displayName": SUBNET_DISPLAY_NAME,
        "cidrBlock": SUBNET_CIDR,
        "securityListIds": [security_list_id],
        "routeTableId": route_table_id,
        "freeformTags": { "byocvpn": "true" },
    });
    if !ipv6_prefix.is_empty() {
        body["ipv6CidrBlocks"] = json!([derive_first_subnet_ipv6_64(ipv6_prefix)]);
    }
    let response = client.post(&url, &body).await.map_err(|e| {
        NetworkProvisioningError::SubnetCreationFailed {
            reason: e.to_string(),
        }
    })?;
    Ok(response["id"]
        .as_str()
        .ok_or(NetworkProvisioningError::MissingSubnetIdentifier)?
        .to_string())
}

/// Ensure the subnet has our security list attached.
///
/// Called after both the find and create paths so that subnets from earlier
/// (partial) runs — created before the security list was properly set up —
/// get it attached retroactively.
pub async fn ensure_subnet_security_list(
    client: &OciClient,
    subnet_id: &str,
    security_list_id: &str,
) -> Result<()> {
    let url = format!("{}/20160918/subnets/{}", client.core_base_url(), subnet_id);
    client
        .put(&url, &json!({ "securityListIds": [security_list_id] }))
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Images — find the latest Oracle Linux 8 or Ubuntu 22.04 image in a region
// ---------------------------------------------------------------------------

/// Look up a known Ubuntu 22.04 image in the current region.
/// Falls back to listing all images and picking the first "Canonical-Ubuntu-22.04" one.
pub async fn get_ubuntu_image(client: &OciClient, compartment_ocid: &str) -> Result<String> {
    let url = format!(
        "{}/20160918/images?compartmentId={}&operatingSystem=Canonical+Ubuntu&operatingSystemVersion=22.04&shape={}&lifecycleState=AVAILABLE",
        client.core_base_url(),
        compartment_ocid,
        "VM.Standard.A1.Flex",
    );
    let response = client.get(&url).await?;
    response
        .as_array()
        .and_then(|list| list.first())
        .and_then(|v| v["id"].as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: "No Ubuntu 22.04 ARM image found for VM.Standard.A1.Flex".to_string(),
            }
            .into()
        })
}

// ---------------------------------------------------------------------------
// Regions
// ---------------------------------------------------------------------------

/// Subscribes to `region_name` if the tenancy is not already subscribed, then
/// polls until the region status is `READY`. No-op if already ready.
/// Must be called with the **home-region** client (Identity API is home-region routed).
pub async fn ensure_region_subscribed(
    client: &OciClient,
    tenancy_ocid: &str,
    region_name: &str,
) -> Result<()> {
    let subscriptions_url = format!(
        "{}/20160918/tenancies/{}/regionSubscriptions",
        client.identity_base_url(),
        tenancy_ocid
    );

    // Check if already subscribed and READY.
    let subscriptions = client.get(&subscriptions_url).await.map_err(|e| {
        NetworkProvisioningError::NetworkQueryFailed {
            reason: e.to_string(),
        }
    })?;
    let already_ready = subscriptions
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .any(|s| {
            s["regionName"].as_str() == Some(region_name) && s["status"].as_str() == Some("READY")
        });
    if already_ready {
        return Ok(());
    }

    // Look up the region key (e.g. "FRA") needed by the subscription endpoint.
    let all_regions_url = format!("{}/20160918/regions", client.identity_base_url());
    let all_regions = client.get(&all_regions_url).await.map_err(|e| {
        NetworkProvisioningError::NetworkQueryFailed {
            reason: e.to_string(),
        }
    })?;
    let region_key = all_regions
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .find_map(|r| {
            if r["name"].as_str() == Some(region_name) {
                r["key"].as_str().map(|k| k.to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| NetworkProvisioningError::NetworkQueryFailed {
            reason: format!("Region '{}' not found in OCI region list", region_name),
        })?;

    // POST to subscribe.
    let body = json!({ "regionKey": region_key });
    client.post(&subscriptions_url, &body).await.map_err(|e| {
        NetworkProvisioningError::NetworkQueryFailed {
            reason: format!("Failed to subscribe to region {}: {}", region_name, e),
        }
    })?;
    println!(
        "Subscribed to region {} (key {}), waiting for it to become READY...",
        region_name, region_key
    );

    // Poll until READY — subscription typically takes 2–5 minutes.
    for attempt in 1..=40u32 {
        sleep(Duration::from_secs(15)).await;
        let subs = client.get(&subscriptions_url).await.map_err(|e| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: e.to_string(),
            }
        })?;
        let is_ready = subs
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .any(|s| {
                s["regionName"].as_str() == Some(region_name)
                    && s["status"].as_str() == Some("READY")
            });
        if is_ready {
            println!(
                "Region {} is now READY (attempt {}/40).",
                region_name, attempt
            );
            return Ok(());
        }
        println!(
            "Region {} not yet ready (attempt {}/40), retrying in 15 s...",
            region_name, attempt
        );
    }

    Err(NetworkProvisioningError::NetworkQueryFailed {
        reason: format!(
            "Timed out waiting for region '{}' to become READY after subscription",
            region_name
        ),
    }
    .into())
}

/// Returns every OCI region available globally (used for the region selector UI).
/// Uses `GET /20160918/regions` which requires no tenancy scoping.
pub async fn list_all_regions(client: &OciClient) -> Result<Vec<(String, String)>> {
    let url = format!("{}/20160918/regions", client.identity_base_url());
    let response =
        client
            .get(&url)
            .await
            .map_err(|e| NetworkProvisioningError::NetworkQueryFailed {
                reason: e.to_string(),
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
        .filter_map(|r| {
            let name = r["name"].as_str()?.to_string();
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

/// Returns only the regions the tenancy is subscribed to.
/// Used for scanning instances — querying unsubscribed regions returns 401.
pub async fn list_regions(client: &OciClient, tenancy_ocid: &str) -> Result<Vec<(String, String)>> {
    // ListRegionSubscriptions returns only the regions the tenancy has subscribed
    // to, whereas ListRegions returns all ~50 global regions (most of which would
    // return 401 for an unsubscribed tenancy).
    let url = format!(
        "{}/20160918/tenancies/{}/regionSubscriptions",
        client.identity_base_url(),
        tenancy_ocid
    );
    let response =
        client
            .get(&url)
            .await
            .map_err(|e| NetworkProvisioningError::NetworkQueryFailed {
                reason: e.to_string(),
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
        .filter_map(|r| {
            let name = r["regionName"].as_str()?.to_string();
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
