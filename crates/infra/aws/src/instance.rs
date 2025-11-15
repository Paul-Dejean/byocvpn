use aws_sdk_ec2::{
    Client as Ec2Client,
    client::Waiters,
    types::{ResourceType, Tag, TagSpecification},
};
use base64::{Engine, engine::general_purpose};
use byocvpn_core::{
    cloud_provider::InstanceInfo,
    error::{ComputeProvisioningError, NetworkProvisioningError, Result},
};
use tokio::time::Duration;

use crate::{AwsProvider, cloud_init, config, network};
pub(super) async fn spawn_instance(
    provider: &AwsProvider,
    subnet_id: &str,
    server_private_key: &str,
    client_public_key: &str,
) -> Result<(String, String, String)> {
    let user_data =
        cloud_init::generate_wireguard_cloud_init(server_private_key, client_public_key);

    println!("{:?}", user_data);
    let encoded_user_data = general_purpose::STANDARD.encode(user_data);

    let ami_id = config::get_al2023_ami(&provider.ssm_client).await?;
    println!("AMI ID: {}", ami_id);

    let group_name = "byocvpn-security-group";
    let security_group_id = network::get_security_group_by_name(&provider.ec2_client, group_name)
        .await?
        .ok_or_else(|| NetworkProvisioningError::SecurityGroupNotFound {
            group_name: group_name.to_string(),
        })?;

    println!("Security group ID: {}", security_group_id);

    let tags = TagSpecification::builder()
        .resource_type(ResourceType::Instance)
        .tags(Tag::builder().key("Name").value("byocvpn-server").build())
        .build();
    let resp = provider
        .ec2_client
        .run_instances()
        .subnet_id(subnet_id)
        .image_id(ami_id)
        .security_group_ids(security_group_id)
        .instance_type(aws_sdk_ec2::types::InstanceType::T2Micro)
        .user_data(encoded_user_data)
        // .key_name("vpn")
        .min_count(1)
        .max_count(1)
        .tag_specifications(tags)
        .send()
        .await
        .map_err(|error| ComputeProvisioningError::InstanceSpawnFailed {
            region_name: provider.get_region_name(),
            reason: error.to_string(),
        })?;
    let instance = resp
        .instances()
        .first()
        .ok_or_else(|| ComputeProvisioningError::NoInstanceInResponse)?;
    let instance_id = instance
        .instance_id()
        .ok_or_else(|| ComputeProvisioningError::InstanceMissingId)?
        .to_string();

    provider
        .ec2_client
        .wait_until_instance_running()
        .instance_ids(&instance_id)
        .wait(Duration::from_secs(60))
        .await
        .map_err(|error| ComputeProvisioningError::InstanceWaitFailed {
            reason: error.to_string(),
        })?;

    let desc = provider
        .ec2_client
        .describe_instances()
        .instance_ids(&instance_id)
        .send()
        .await
        .map_err(|error| ComputeProvisioningError::InstanceSpawnFailed {
            region_name: provider.get_region_name(),
            reason: error.to_string(),
        })?;

    let public_ip_v4 = desc
        .reservations()
        .iter()
        .flat_map(|r| r.instances())
        .filter_map(|i| i.public_ip_address())
        .next()
        .ok_or_else(|| ComputeProvisioningError::MissingPublicIpv4)?
        .to_string();

    let public_ip_v6 = desc
        .reservations()
        .iter()
        .flat_map(|r| r.instances())
        .filter_map(|i| i.ipv6_address())
        .next()
        .ok_or_else(|| ComputeProvisioningError::MissingPublicIpv6)?
        .to_string();
    println!("Instance ID: {}", instance_id);
    println!("Public IPv4: {}", public_ip_v4);
    println!("Public IPv6: {}", public_ip_v6);

    Ok((instance_id, public_ip_v4, public_ip_v6))
}

pub async fn terminate_instance(ec2_client: &Ec2Client, instance_id: &str) -> Result<()> {
    ec2_client
        .terminate_instances()
        .instance_ids(instance_id)
        .send()
        .await
        .map_err(
            |error| ComputeProvisioningError::InstanceTerminationFailed {
                instance_identifier: instance_id.to_string(),
                reason: error.to_string(),
            },
        )?;

    Ok(())
}

pub(super) async fn list_instances(ec2_client: &Ec2Client) -> Result<Vec<InstanceInfo>> {
    let resp = ec2_client
        .describe_instances()
        .send()
        .await
        .map_err(|error| ComputeProvisioningError::InstanceSpawnFailed {
            region_name: "unknown".to_string(), // We don't have provider here
            reason: error.to_string(),
        })?;

    let instances = resp
        .reservations()
        .iter()
        .flat_map(|r| r.instances())
        .filter_map(|i| {
            let id = i.instance_id()?.to_string();

            let state = i
                .state()
                .and_then(|s| s.name().map(|s| s.as_str()))?
                .to_string();

            if state != "running" {
                return None;
            }
            let name = i.tags().iter().find_map(|tag| {
                tag.key()
                    .filter(|key| *key == "Name")
                    .and_then(|_| tag.value().map(ToString::to_string))
            });

            let public_ip_v4 = i.public_ip_address().map(|ip| ip.to_string())?;
            let public_ip_v6 = i.ipv6_address().map(|ip| ip.to_string())?;

            Some(InstanceInfo {
                id,
                name,
                state,
                public_ip_v4,
                public_ip_v6,
            })
        })
        .collect();
    Ok(instances)
}
