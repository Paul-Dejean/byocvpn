use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub fn byocvpn_tags() -> HashMap<String, String> {
    HashMap::from([("created-by".to_string(), "byocvpn".to_string())])
}

// ── Instance ─────────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchInstanceRequest {
    pub compartment_id: String,
    pub display_name: String,
    pub availability_domain: String,
    pub shape: String,
    pub shape_config: ShapeConfig,
    pub source_details: SourceDetails,
    pub create_vnic_details: CreateVnicDetails,
    pub metadata: HashMap<String, String>,
    pub freeform_tags: HashMap<String, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeConfig {
    pub ocpus: f32,
    pub memory_in_g_bs: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceDetails {
    pub source_type: String,
    pub image_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVnicDetails {
    pub subnet_id: String,
    pub assign_public_ip: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assign_ipv6_ip: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceResponse {
    pub id: String,
    pub display_name: Option<String>,
    pub lifecycle_state: String,
    pub shape: String,
    pub time_created: Option<String>,
    pub freeform_tags: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
pub struct AvailabilityDomain {
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VnicAttachment {
    pub vnic_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vnic {
    pub public_ip: Option<String>,
    pub ipv6_addresses: Option<Vec<String>>,
}

// ── Network ───────────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVcnRequest {
    pub compartment_id: String,
    pub display_name: String,
    pub cidr_block: String,
    pub is_ipv6_enabled: bool,
    pub freeform_tags: HashMap<String, String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vcn {
    pub id: String,
    pub default_route_table_id: String,
    pub ipv6_cidr_blocks: Option<Vec<String>>,
    pub lifecycle_state: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInternetGatewayRequest {
    pub compartment_id: String,
    pub vcn_id: String,
    pub display_name: String,
    pub is_enabled: bool,
    pub freeform_tags: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct InternetGateway {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteRule {
    pub network_entity_id: String,
    pub destination: String,
    pub destination_type: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteTableResponse {
    pub route_rules: Vec<RouteRule>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRouteTableRequest {
    pub route_rules: Vec<RouteRule>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortRange {
    pub min: u16,
    pub max: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UdpOptions {
    pub destination_port_range: PortRange,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcpOptions {
    pub destination_port_range: PortRange,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngressSecurityRule {
    pub protocol: String,
    pub source: String,
    pub source_type: String,
    pub is_stateless: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp_options: Option<UdpOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_options: Option<TcpOptions>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EgressSecurityRule {
    pub protocol: String,
    pub destination: String,
    pub destination_type: String,
    pub is_stateless: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSecurityListRequest {
    pub compartment_id: String,
    pub vcn_id: String,
    pub display_name: String,
    pub freeform_tags: HashMap<String, String>,
    pub ingress_security_rules: Vec<IngressSecurityRule>,
    pub egress_security_rules: Vec<EgressSecurityRule>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSecurityListRequest {
    pub ingress_security_rules: Vec<IngressSecurityRule>,
    pub egress_security_rules: Vec<EgressSecurityRule>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityListResponse {
    pub id: String,
    pub ingress_security_rules: Option<Vec<IngressSecurityRule>>,
    pub egress_security_rules: Option<Vec<EgressSecurityRule>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubnetResponse {
    pub id: String,
    pub lifecycle_state: String,
    pub ipv6_cidr_blocks: Option<Vec<String>>,
    pub security_list_ids: Option<Vec<String>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubnetRequest {
    pub compartment_id: String,
    pub vcn_id: String,
    pub display_name: String,
    pub cidr_block: String,
    pub security_list_ids: Vec<String>,
    pub route_table_id: String,
    pub freeform_tags: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_cidr_blocks: Option<Vec<String>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubnetSecurityListRequest {
    pub security_list_ids: Vec<String>,
}

#[derive(Deserialize)]
pub struct Image {
    pub id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionSubscription {
    pub region_name: String,
    pub status: String,
}

#[derive(Deserialize)]
pub struct Region {
    pub name: String,
    pub key: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscribeRegionRequest {
    pub region_key: String,
}
