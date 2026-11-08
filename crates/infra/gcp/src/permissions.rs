use std::collections::HashSet;

use byocvpn_core::cloud_provider::PermissionStatus;
use log::{info, warn};
use serde::{Deserialize, Serialize};

use crate::{account::ensure_cloud_resource_manager_api_enabled, client::GcpClient, error::Result};

const REQUIRED_PERMISSIONS: &[&str] = &[
    "compute.disks.create",
    "compute.disks.delete",
    "compute.firewalls.create",
    "compute.firewalls.delete",
    "compute.firewalls.get",
    "compute.firewalls.update",
    "compute.globalOperations.get",
    "compute.images.get",
    "compute.instances.create",
    "compute.instances.delete",
    "compute.instances.get",
    "compute.instances.list",
    "compute.instances.setLabels",
    "compute.instances.setMetadata",
    "compute.instances.setTags",
    "compute.networks.create",
    "compute.networks.delete",
    "compute.networks.get",
    "compute.networks.updatePolicy",
    "compute.regionOperations.get",
    "compute.regions.list",
    "compute.subnetworks.create",
    "compute.subnetworks.delete",
    "compute.subnetworks.get",
    "compute.subnetworks.list",
    "compute.subnetworks.update",
    "compute.subnetworks.use",
    "compute.subnetworks.useExternalIp",
    "compute.zoneOperations.get",
    "serviceusage.operations.get",
    "serviceusage.services.enable",
    "serviceusage.services.get",
];

#[derive(Serialize)]
struct TestIamPermissionsRequest {
    permissions: Vec<String>,
}

#[derive(Deserialize, Default)]
struct TestIamPermissionsResponse {
    #[serde(default)]
    permissions: Vec<String>,
}

pub async fn verify_permissions(client: &GcpClient) -> Result<Vec<PermissionStatus>> {
    ensure_cloud_resource_manager_api_enabled(client).await?;

    let url = client.build_project_url(":testIamPermissions");
    let request = TestIamPermissionsRequest {
        permissions: REQUIRED_PERMISSIONS
            .iter()
            .map(ToString::to_string)
            .collect(),
    };
    let response: TestIamPermissionsResponse = client.post(&url, &request).await?;
    let granted_permissions: HashSet<&str> =
        response.permissions.iter().map(String::as_str).collect();

    Ok(REQUIRED_PERMISSIONS
        .iter()
        .map(|permission| {
            let granted = granted_permissions.contains(permission);
            if granted {
                info!("[GCP] permission check {permission}: authorized");
            } else {
                warn!("[GCP] permission check {permission}: denied");
            }
            PermissionStatus {
                permission: (*permission).to_string(),
                granted,
            }
        })
        .collect())
}
