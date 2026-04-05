use aws_sdk_ec2::{
    Client as Ec2Client,
    client::Waiters,
    types::{ResourceType, Tag, TagSpecification},
};
use aws_sdk_ssm::Client as SsmClient;
use base64::{Engine, engine::general_purpose};
use byocvpn_core::{
    cloud_provider::{CloudProviderName, InstanceInfo, InstanceState},
    error::{ComputeProvisioningError, NetworkProvisioningError, Result},
};
use chrono::{DateTime, Utc};
use log::*;
use tokio::time::Duration;

use crate::{aws_error::sdk_error_message, config, network, startup_script, state::Ec2InstanceState};

pub(super) async fn spawn_instance(
    ec2_client: &Ec2Client,
    ssm_client: &SsmClient,
    region: &str,
    server_private_key: &str,
    client_public_key: &str,
) -> Result<InstanceInfo> {
    let vpc_id = network::get_vpc_by_name(&ec2_client, "byocvpn-vpc")
        .await?
        .ok_or_else(|| NetworkProvisioningError::VpcNotFound {
            vpc_name: "byocvpn-vpc".to_string(),
        })?;

    let subnets = network::get_subnets_in_vpc(&ec2_client, &vpc_id).await?;

    let subnet_id = subnets[0]
        .subnet_id
        .clone()
        .ok_or(NetworkProvisioningError::SubnetMissingIdentifier {})?;
    let user_data =
        startup_script::generate_server_startup_script(server_private_key, client_public_key)?;

    info!("{:?}", user_data);
    let encoded_user_data = general_purpose::STANDARD.encode(user_data);

    let ami_id = config::get_al2023_ami(&ssm_client).await?;
    info!("AMI ID: {}", ami_id);

    let group_name = "byocvpn-security-group";
    let security_group_id = network::get_security_group_by_name(&ec2_client, group_name)
        .await?
        .ok_or_else(|| NetworkProvisioningError::SecurityGroupNotFound {
            group_name: group_name.to_string(),
        })?;

    info!("Security group ID: {}", security_group_id);

    let tags = TagSpecification::builder()
        .resource_type(ResourceType::Instance)
        .tags(Tag::builder().key("Name").value("byocvpn-server").build())
        .build();
    let resp = ec2_client
        .run_instances()
        .subnet_id(subnet_id)
        .image_id(ami_id)
        .security_group_ids(security_group_id)
        .instance_type(aws_sdk_ec2::types::InstanceType::T2Micro)
        .user_data(encoded_user_data)
        .min_count(1)
        .max_count(1)
        .tag_specifications(tags)
        .send()
        .await
        .map_err(|error| ComputeProvisioningError::InstanceSpawnFailed {
            region_name: region.to_string(),
            reason: sdk_error_message(&error),
        })?;
    let instance = resp
        .instances()
        .first()
        .ok_or_else(|| ComputeProvisioningError::NoInstanceInResponse)?;
    let instance_id = instance
        .instance_id()
        .ok_or_else(|| ComputeProvisioningError::MissingInstanceIdentifier)?
        .to_string();

    ec2_client
        .wait_until_instance_running()
        .instance_ids(&instance_id)
        .wait(Duration::from_secs(60))
        .await
        .map_err(|error| ComputeProvisioningError::InstanceWaitFailed {
            reason: error.to_string(),
        })?;

    let desc = ec2_client
        .describe_instances()
        .instance_ids(&instance_id)
        .send()
        .await
        .map_err(|error| ComputeProvisioningError::InstanceSpawnFailed {
            region_name: region.to_string(),
            reason: sdk_error_message(&error),
        })?;

    let public_ip_v4 = desc
        .reservations()
        .iter()
        .flat_map(|reservation| reservation.instances())
        .filter_map(|instance| instance.public_ip_address())
        .next()
        .ok_or_else(|| ComputeProvisioningError::MissingPublicIpv4)?
        .to_string();

    let public_ip_v6 = desc
        .reservations()
        .iter()
        .flat_map(|reservation| reservation.instances())
        .filter_map(|instance| instance.ipv6_address())
        .next()
        .ok_or_else(|| ComputeProvisioningError::MissingPublicIpv6)?
        .to_string();
    info!("Instance ID: {}", instance_id);
    info!("Public IPv4: {}", public_ip_v4);
    info!("Public IPv6: {}", public_ip_v6);

    Ok(InstanceInfo {
        id: instance_id,
        name: Some("byocvpn-server".to_string()),
        state: InstanceState::Running,
        public_ip_v4,
        public_ip_v6,
        region: region.to_string(),
        provider: CloudProviderName::Aws,
        instance_type: "t2.micro".to_string(),
        launched_at: Some(Utc::now()),
    })
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
                reason: sdk_error_message(&error),
            },
        )?;

    Ok(())
}

pub(super) async fn list_instances_in_region(
    ec2_client: &Ec2Client,
    region: &str,
) -> Result<Vec<InstanceInfo>> {
    let resp = ec2_client
        .describe_instances()
        .send()
        .await
        .map_err(|error| ComputeProvisioningError::InstanceSpawnFailed {
            region_name: "unknown".to_string(),
            reason: sdk_error_message(&error),
        })?;

    let instances = resp
        .reservations()
        .iter()
        .flat_map(|reservation| reservation.instances())
        .filter_map(|instance| {
            let id = instance.instance_id()?.to_string();

            let raw_state = instance
                .state()
                .and_then(|instance_state| {
                    instance_state.name().map(|state_name| state_name.as_str())
                })
                .unwrap_or("unknown");

            let state: InstanceState = Ec2InstanceState::from(raw_state).into();
            let name = instance.tags().iter().find_map(|tag| {
                tag.key()
                    .filter(|key| *key == "Name")
                    .and_then(|_| tag.value().map(ToString::to_string))
            });

            let public_ip_v4 = instance
                .public_ip_address()
                .map(|address| address.to_string())?;
            let public_ip_v6 = instance.ipv6_address().map(|address| address.to_string())?;

            let instance_type = instance
                .instance_type()
                .map(|type_value| type_value.as_str().to_string())
                .unwrap_or_default();

            let launched_at = instance.launch_time().and_then(|timestamp| {
                DateTime::parse_from_rfc3339(&timestamp.to_string())
                    .ok()
                    .map(|datetime| datetime.with_timezone(&Utc))
            });

            Some(InstanceInfo {
                id,
                name,
                state,
                public_ip_v4,
                public_ip_v6,
                region: region.to_string(),
                provider: CloudProviderName::Aws,
                instance_type,
                launched_at,
            })
        })
        .collect();
    Ok(instances)
}
