use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub fn byocvpn_tags() -> HashMap<String, String> {
    HashMap::from([("created-by".to_string(), "byocvpn".to_string())])
}

// ── Async operation polling ───────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct AsyncOperationResponse {
    pub status: Option<String>,
    pub error: Option<AsyncOperationError>,
}

#[derive(Deserialize)]
pub struct AsyncOperationError {
    pub message: Option<String>,
}

// ── Provider registration ─────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderRegistrationResponse {
    pub registration_state: Option<String>,
}

#[derive(Serialize)]
pub struct EmptyRequest {}

// ── Resource group ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ResourceGroupRequest {
    pub location: String,
    pub tags: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct ResourceGroupResponse {
    pub id: Option<String>,
}

// ── NSG ───────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct NsgRequest {
    pub location: String,
    pub tags: HashMap<String, String>,
    pub properties: NsgProperties,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NsgProperties {
    pub security_rules: Vec<SecurityRule>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SecurityRule {
    pub name: String,
    pub properties: SecurityRuleProperties,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SecurityRuleProperties {
    pub priority: u32,
    pub protocol: String,
    pub access: String,
    pub direction: String,
    pub source_address_prefix: String,
    pub source_port_range: String,
    pub destination_address_prefix: String,
    pub destination_port_range: String,
}

#[derive(Deserialize)]
pub struct NsgResponse {
    pub id: Option<String>,
    pub location: Option<String>,
    pub tags: Option<serde_json::Value>,
    pub properties: Option<NsgResponseProperties>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NsgResponseProperties {
    pub security_rules: Option<Vec<SecurityRule>>,
}

// ── VNet ──────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct VnetRequest {
    pub location: String,
    pub tags: HashMap<String, String>,
    pub properties: VnetProperties,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VnetProperties {
    pub address_space: AddressSpace,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressSpace {
    pub address_prefixes: Vec<String>,
}

#[derive(Deserialize)]
pub struct VnetResponse {
    pub id: Option<String>,
}

// ── Subnet ────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct SubnetRequest {
    pub properties: SubnetRequestProperties,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubnetRequestProperties {
    pub address_prefixes: Vec<String>,
    pub network_security_group: ResourceReference,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ResourceReference {
    pub id: String,
}

#[derive(Deserialize)]
pub struct SubnetResponse {
    pub id: Option<String>,
    pub properties: Option<SubnetResponseProperties>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubnetResponseProperties {
    pub address_prefixes: Option<Vec<String>>,
    pub network_security_group: Option<ResourceReference>,
}

// ── Locations ─────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LocationListResponse {
    pub value: Option<Vec<LocationItem>>,
}

#[derive(Deserialize)]
pub struct LocationItem {
    pub name: Option<String>,
    pub metadata: Option<LocationMetadata>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationMetadata {
    pub region_category: Option<String>,
}

// ── Public IP ─────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct PublicIpRequest {
    pub location: String,
    pub sku: PublicIpSku,
    pub tags: HashMap<String, String>,
    pub properties: PublicIpProperties,
}

#[derive(Serialize)]
pub struct PublicIpSku {
    pub name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicIpProperties {
    pub public_ip_allocation_method: String,
    #[serde(rename = "publicIPAddressVersion")]
    pub public_ip_address_version: String,
}

#[derive(Deserialize)]
pub struct PublicIpResponse {
    pub id: Option<String>,
    pub properties: Option<PublicIpResponseProperties>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicIpResponseProperties {
    pub ip_address: Option<String>,
}

// ── NIC ───────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct NicRequest {
    pub location: String,
    pub tags: HashMap<String, String>,
    pub properties: NicProperties,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NicProperties {
    pub network_security_group: ResourceReference,
    pub ip_configurations: Vec<IpConfiguration>,
}

#[derive(Serialize)]
pub struct IpConfiguration {
    pub name: String,
    pub properties: IpConfigurationProperties,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IpConfigurationProperties {
    pub private_ip_allocation_method: String,
    #[serde(rename = "privateIPAddressVersion")]
    pub private_ip_address_version: String,
    pub subnet: ResourceReference,
    pub public_ip_address: ResourceReference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
}

#[derive(Deserialize)]
pub struct NicResponse {
    pub id: Option<String>,
}

// ── VM ────────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct VmRequest {
    pub location: String,
    pub tags: HashMap<String, String>,
    pub properties: VmProperties,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VmProperties {
    pub hardware_profile: HardwareProfile,
    pub os_profile: OsProfile,
    pub storage_profile: StorageProfile,
    pub network_profile: NetworkProfile,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareProfile {
    pub vm_size: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OsProfile {
    pub computer_name: String,
    pub admin_username: String,
    pub admin_password: String,
    pub linux_configuration: LinuxConfiguration,
    pub custom_data: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinuxConfiguration {
    pub disable_password_authentication: bool,
    pub provision_vm_agent: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageProfile {
    pub image_reference: ImageReference,
    pub os_disk: OsDisk,
}

#[derive(Serialize)]
pub struct ImageReference {
    pub publisher: String,
    pub offer: String,
    pub sku: String,
    pub version: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OsDisk {
    pub create_option: String,
    pub delete_option: String,
    pub disk_size_gb: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkProfile {
    pub network_interfaces: Vec<NetworkInterfaceReference>,
}

#[derive(Serialize)]
pub struct NetworkInterfaceReference {
    pub id: String,
    pub properties: NetworkInterfaceReferenceProperties,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterfaceReferenceProperties {
    pub delete_option: String,
}

#[derive(Deserialize)]
pub struct VmListResponse {
    pub value: Option<Vec<VmResponse>>,
}

#[derive(Deserialize)]
pub struct VmResponse {
    pub id: Option<String>,
    pub name: Option<String>,
    pub location: Option<String>,
    pub tags: Option<HashMap<String, String>>,
    pub properties: Option<VmResponseProperties>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmResponseProperties {
    pub provisioning_state: Option<String>,
    pub hardware_profile: Option<HardwareProfileResponse>,
    pub time_created: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareProfileResponse {
    pub vm_size: Option<String>,
}
