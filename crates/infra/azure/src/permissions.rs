use byocvpn_core::cloud_provider::PermissionStatus;
use serde::Deserialize;
use tracing::{debug, warn};

use crate::{client::AzureClient, error::Result};

const API_VERSION_AUTHORIZATION: &str = "2018-07-01";

const REQUIRED_ACTIONS: &[&str] = &[
    "Microsoft.Compute/register/action",
    "Microsoft.Compute/locations/operations/read",
    "Microsoft.Compute/virtualMachines/read",
    "Microsoft.Compute/virtualMachines/write",
    "Microsoft.Compute/virtualMachines/delete",
    "Microsoft.Network/register/action",
    "Microsoft.Network/locations/operations/read",
    "Microsoft.Network/locations/operationResults/read",
    "Microsoft.Network/networkInterfaces/join/action",
    "Microsoft.Network/networkInterfaces/read",
    "Microsoft.Network/networkInterfaces/write",
    "Microsoft.Network/networkInterfaces/delete",
    "Microsoft.Network/networkSecurityGroups/join/action",
    "Microsoft.Network/networkSecurityGroups/read",
    "Microsoft.Network/networkSecurityGroups/write",
    "Microsoft.Network/networkSecurityGroups/delete",
    "Microsoft.Network/publicIPAddresses/join/action",
    "Microsoft.Network/publicIPAddresses/read",
    "Microsoft.Network/publicIPAddresses/write",
    "Microsoft.Network/publicIPAddresses/delete",
    "Microsoft.Network/virtualNetworks/read",
    "Microsoft.Network/virtualNetworks/write",
    "Microsoft.Network/virtualNetworks/delete",
    "Microsoft.Network/virtualNetworks/subnets/join/action",
    "Microsoft.Network/virtualNetworks/subnets/read",
    "Microsoft.Network/virtualNetworks/subnets/write",
    "Microsoft.Network/virtualNetworks/subnets/delete",
    "Microsoft.Resources/register/action",
    "Microsoft.Resources/subscriptions/locations/read",
    "Microsoft.Resources/subscriptions/providers/read",
    "Microsoft.Resources/subscriptions/resourceGroups/read",
    "Microsoft.Resources/subscriptions/resourceGroups/write",
    "Microsoft.Resources/subscriptions/resourceGroups/delete",
];

#[derive(Deserialize)]
struct PermissionsResponse {
    #[serde(default)]
    value: Vec<PermissionSet>,
}

#[derive(Deserialize, Default)]
struct PermissionSet {
    #[serde(default)]
    actions: Vec<String>,
    #[serde(default, rename = "notActions")]
    not_actions: Vec<String>,
}

pub async fn verify_permissions(client: &AzureClient) -> Result<Vec<PermissionStatus>> {
    let path = client.build_subscription_path("/providers/Microsoft.Authorization/permissions");
    let url = AzureClient::build_arm_url(&path, API_VERSION_AUTHORIZATION);

    let response: PermissionsResponse = client.get(&url).await?;

    Ok(REQUIRED_ACTIONS
        .iter()
        .map(|action| {
            let granted = is_action_granted(action, &response.value);
            if granted {
                debug!("permission check {action}: authorized");
            } else {
                warn!("permission check {action}: denied");
            }
            PermissionStatus {
                permission: (*action).to_string(),
                granted,
            }
        })
        .collect())
}

fn is_action_granted(action: &str, permission_sets: &[PermissionSet]) -> bool {
    permission_sets.iter().any(|permission_set| {
        let allowed = permission_set
            .actions
            .iter()
            .any(|pattern| wildcard_matches(pattern, action));
        let denied = permission_set
            .not_actions
            .iter()
            .any(|pattern| wildcard_matches(pattern, action));
        allowed && !denied
    })
}

fn wildcard_matches(pattern: &str, action: &str) -> bool {
    let pattern: Vec<char> = pattern.to_lowercase().chars().collect();
    let action: Vec<char> = action.to_lowercase().chars().collect();
    let mut pattern_index = 0;
    let mut action_index = 0;
    let mut star_pattern_index: Option<usize> = None;
    let mut star_action_index = 0;

    while let Some(action_char) = action.get(action_index) {
        match pattern.get(pattern_index) {
            Some(pattern_char) if pattern_char == action_char => {
                pattern_index += 1;
                action_index += 1;
            }
            Some('*') => {
                star_pattern_index = Some(pattern_index);
                star_action_index = action_index;
                pattern_index += 1;
            }
            _ => match star_pattern_index {
                Some(star_index) => {
                    pattern_index = star_index + 1;
                    star_action_index += 1;
                    action_index = star_action_index;
                }
                None => return false,
            },
        }
    }

    while pattern.get(pattern_index) == Some(&'*') {
        pattern_index += 1;
    }
    pattern_index == pattern.len()
}
