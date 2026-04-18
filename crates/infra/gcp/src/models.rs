use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub status: Option<String>,
    pub self_link: Option<String>,
    pub error: Option<OperationError>,
    pub name: Option<String>,
    pub done: Option<bool>,
}

#[derive(Deserialize)]
pub struct OperationError {
    pub errors: Option<Vec<OperationErrorDetail>>,
    pub message: Option<String>,
}

#[derive(Deserialize)]
pub struct OperationErrorDetail {
    pub message: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVpcRequest {
    pub name: String,
    pub auto_create_subnetworks: bool,
    pub description: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VpcResponse {
    pub self_link: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFirewallRuleRequest {
    pub name: String,
    pub network: String,
    pub description: String,
    pub direction: String,
    pub priority: u32,
    pub target_tags: Vec<String>,
    pub allowed: Vec<FirewallAllowed>,
    pub source_ranges: Vec<String>,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct FirewallAllowed {
    #[serde(rename = "IPProtocol")]
    pub ip_protocol: String,
    pub ports: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchFirewallRuleRequest {
    pub allowed: Vec<FirewallAllowed>,
    pub source_ranges: Vec<String>,
    pub target_tags: Vec<String>,
    pub direction: String,
    pub priority: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallRuleResponse {
    pub allowed: Option<Vec<FirewallAllowed>>,
    pub source_ranges: Option<Vec<String>>,
    pub target_tags: Option<Vec<String>>,
    pub direction: Option<String>,
    pub priority: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubnetRequest {
    pub name: String,
    pub network: String,
    pub region: String,
    pub ip_cidr_range: String,
    pub description: String,
    pub private_ip_google_access: bool,
    pub stack_type: String,
    pub ipv6_access_type: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchSubnetRequest {
    pub stack_type: String,
    pub ipv6_access_type: String,
    pub fingerprint: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubnetResponse {
    pub self_link: Option<String>,
    pub stack_type: Option<String>,
    pub fingerprint: Option<String>,
}

#[derive(Deserialize)]
pub struct RegionListResponse {
    pub items: Option<Vec<RegionItem>>,
}

#[derive(Deserialize)]
pub struct RegionItem {
    pub name: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct ServiceResponse {
    pub state: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageResponse {
    pub self_link: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstanceRequest {
    pub name: String,
    pub machine_type: String,
    pub disks: Vec<AttachedDisk>,
    pub network_interfaces: Vec<NetworkInterface>,
    pub metadata: InstanceMetadata,
    pub labels: HashMap<String, String>,
    pub tags: InstanceTags,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachedDisk {
    pub boot: bool,
    pub auto_delete: bool,
    pub initialize_params: DiskInitializeParams,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInitializeParams {
    pub source_image: String,
    pub disk_size_gb: String,
    pub disk_type: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterface {
    pub subnetwork: String,
    pub stack_type: String,
    pub access_configs: Vec<AccessConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_access_configs: Option<Vec<Ipv6AccessConfig>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessConfig {
    #[serde(rename = "type")]
    pub access_type: String,
    pub name: String,
    pub network_tier: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ipv6AccessConfig {
    #[serde(rename = "type")]
    pub access_type: String,
    pub name: String,
    pub network_tier: String,
}

#[derive(Serialize)]
pub struct InstanceMetadata {
    pub items: Vec<MetadataItem>,
}

#[derive(Serialize)]
pub struct MetadataItem {
    pub key: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct InstanceTags {
    pub items: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceResponse {
    pub name: Option<String>,
    pub status: Option<String>,
    pub machine_type: Option<String>,
    pub creation_timestamp: Option<String>,
    pub network_interfaces: Option<Vec<NetworkInterfaceResponse>>,
    pub self_link: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterfaceResponse {
    pub access_configs: Option<Vec<AccessConfigResponse>>,
    pub ipv6_access_configs: Option<Vec<Ipv6AccessConfigResponse>>,
}

#[derive(Deserialize)]
pub struct AccessConfigResponse {
    #[serde(rename = "natIP")]
    pub nat_ip: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ipv6AccessConfigResponse {
    pub external_ipv6: Option<String>,
}

#[derive(Deserialize)]
pub struct ZoneInstanceListResponse {
    pub items: Option<HashMap<String, ZoneInstances>>,
}

#[derive(Deserialize)]
pub struct ZoneInstances {
    pub instances: Option<Vec<InstanceResponse>>,
}

#[derive(Deserialize)]
pub struct ZoneOperationResponse {
    pub status: Option<String>,
    pub error: Option<OperationError>,
}

#[derive(Serialize)]
pub struct EmptyRequest {}
