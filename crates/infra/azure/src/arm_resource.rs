use std::fmt::{Display, Formatter, Result as FormatResult};

pub const API_VERSION_RESOURCE_GROUPS: &str = "2021-04-01";
pub const API_VERSION_NETWORK: &str = "2024-05-01";
pub const API_VERSION_COMPUTE: &str = "2024-07-01";

const NETWORK_PROVIDER: &str = "Microsoft.Network";
const COMPUTE_PROVIDER: &str = "Microsoft.Compute";

const VNET_NAME: &str = "byocvpn-vnet";
const SUBNET_NAME: &str = "byocvpn-subnet";
const NSG_NAME: &str = "byocvpn-nsg";

pub fn build_resource_group_name(location: &str) -> String {
    format!("byocvpn-{location}")
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IpVersion {
    V4,
    V6,
}

impl IpVersion {
    const fn get_resource_suffix(self) -> &'static str {
        match self {
            Self::V4 => "pip4",
            Self::V6 => "pip6",
        }
    }

    pub const fn get_address_version(self) -> &'static str {
        match self {
            Self::V4 => "IPv4",
            Self::V6 => "IPv6",
        }
    }
}

pub struct ArmResource {
    resource_type: &'static str,
    name: String,
    subscription_relative_path: String,
    api_version: &'static str,
}

impl ArmResource {
    pub fn resource_group(location: &str) -> Self {
        let name = build_resource_group_name(location);
        Self {
            resource_type: "resourceGroups",
            subscription_relative_path: format!("/resourceGroups/{name}"),
            name,
            api_version: API_VERSION_RESOURCE_GROUPS,
        }
    }

    pub fn network_security_group(location: &str) -> Self {
        Self::build_in_resource_group(
            &build_resource_group_name(location),
            NETWORK_PROVIDER,
            "networkSecurityGroups",
            NSG_NAME.to_string(),
            API_VERSION_NETWORK,
        )
    }

    pub fn virtual_network(location: &str) -> Self {
        Self::build_in_resource_group(
            &build_resource_group_name(location),
            NETWORK_PROVIDER,
            "virtualNetworks",
            VNET_NAME.to_string(),
            API_VERSION_NETWORK,
        )
    }

    pub fn subnet(location: &str) -> Self {
        Self::build_in_resource_group(
            &build_resource_group_name(location),
            NETWORK_PROVIDER,
            "virtualNetworks",
            format!("{VNET_NAME}/subnets/{SUBNET_NAME}"),
            API_VERSION_NETWORK,
        )
    }

    pub fn public_ip(resource_group: &str, vm_name: &str, version: IpVersion) -> Self {
        Self::build_in_resource_group(
            resource_group,
            NETWORK_PROVIDER,
            "publicIPAddresses",
            format!("{vm_name}-{}", version.get_resource_suffix()),
            API_VERSION_NETWORK,
        )
    }

    pub fn network_interface(resource_group: &str, vm_name: &str) -> Self {
        Self::build_in_resource_group(
            resource_group,
            NETWORK_PROVIDER,
            "networkInterfaces",
            format!("{vm_name}-nic"),
            API_VERSION_NETWORK,
        )
    }

    pub fn virtual_machine(resource_group: &str, vm_name: &str) -> Self {
        Self::build_in_resource_group(
            resource_group,
            COMPUTE_PROVIDER,
            "virtualMachines",
            vm_name.to_string(),
            API_VERSION_COMPUTE,
        )
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub const fn get_api_version(&self) -> &'static str {
        self.api_version
    }

    pub fn get_subscription_relative_path(&self) -> &str {
        &self.subscription_relative_path
    }

    fn build_in_resource_group(
        resource_group: &str,
        provider: &str,
        resource_type: &'static str,
        name: String,
        api_version: &'static str,
    ) -> Self {
        Self {
            resource_type,
            subscription_relative_path: format!(
                "/resourceGroups/{resource_group}/providers/{provider}/{resource_type}/{name}"
            ),
            name,
            api_version,
        }
    }
}

impl Display for ArmResource {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        write!(formatter, "{} '{}'", self.resource_type, self.name)
    }
}
