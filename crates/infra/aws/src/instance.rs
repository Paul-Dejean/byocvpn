use aws_sdk_ec2::{
    client::Waiters,
    types::{
        Filter, Instance, InstanceStateName, InstanceType, Reservation, ResourceType, Tag,
        TagSpecification,
    },
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use byocvpn_core::{
    cloud_provider::{CloudProviderName, InstanceInfo, InstanceState, SpawnInstanceParams},
    managed_resources::{MANAGED_BY_TAG_KEY, MANAGED_BY_TAG_VALUE, SPAWN_ID_TAG_KEY},
    server_startup::{PackageManager, generate_server_startup_script},
};
use chrono::{DateTime, Utc};
use log::{debug, info};
use tokio::time::Duration;

use crate::{
    client::AwsClient,
    error::{AwsError, Result},
    network::RegionNetwork,
    state::Ec2InstanceState,
};

const SERVER_INSTANCE_NAME: &str = "byocvpn-server";
const SERVER_INSTANCE_TYPE: &str = "t3.micro";
const NAME_TAG_KEY: &str = "Name";
const LISTED_INSTANCE_STATE_NAMES: [&str; 3] = ["running", "stopping", "stopped"];
const INSTANCE_RUNNING_TIMEOUT: Duration = Duration::from_secs(60);

pub async fn spawn_instance(
    client: &AwsClient,
    region_network: &RegionNetwork,
    image_id: &str,
    params: &SpawnInstanceParams<'_>,
) -> Result<InstanceInfo> {
    let region = params.region;
    let startup_script = generate_server_startup_script(
        PackageManager::Dnf,
        params.server_private_key,
        params.client_public_key,
    )?;
    debug!(
        "[AWS] Generated startup script ({} bytes)",
        startup_script.len()
    );

    let response = client
        .ec2
        .run_instances()
        .subnet_id(&region_network.subnet_id)
        .image_id(image_id)
        .security_group_ids(&region_network.security_group_id)
        .instance_type(InstanceType::from(SERVER_INSTANCE_TYPE))
        .user_data(BASE64.encode(startup_script))
        .min_count(1)
        .max_count(1)
        .tag_specifications(build_instance_tags(params.spawn_id))
        .send()
        .await?;
    let instance_id = response
        .instances()
        .first()
        .and_then(|instance| instance.instance_id())
        .ok_or(AwsError::MissingResponseField {
            operation: "ec2:RunInstances",
            field: "instanceId",
        })?
        .to_string();

    client
        .ec2
        .wait_until_instance_running()
        .instance_ids(&instance_id)
        .wait(INSTANCE_RUNNING_TIMEOUT)
        .await?;

    let (public_ip_v4, public_ip_v6) = get_public_ips(client, &instance_id).await?;
    info!(
        "[AWS] Instance {instance_id} spawned in {region} (IPv4 {public_ip_v4}, IPv6 {public_ip_v6})"
    );

    Ok(InstanceInfo {
        id: instance_id,
        name: Some(SERVER_INSTANCE_NAME.to_string()),
        region: region.to_string(),
        state: InstanceState::Running,
        error_reason: None,
        public_ip_v4,
        public_ip_v6,
        provider: CloudProviderName::Aws,
        instance_type: SERVER_INSTANCE_TYPE.to_string(),
        launched_at: Some(Utc::now()),
        spawn_id: Some(params.spawn_id.to_string()),
    })
}

pub async fn terminate_instance(client: &AwsClient, instance_id: &str) -> Result<()> {
    debug!("[AWS] Terminating instance {instance_id}");
    client
        .ec2
        .terminate_instances()
        .instance_ids(instance_id)
        .send()
        .await?;
    info!("[AWS] Instance {instance_id} terminated");
    Ok(())
}

pub async fn list_instances(client: &AwsClient, region: &str) -> Result<Vec<InstanceInfo>> {
    debug!("[AWS] Listing instances in region {region}");
    let response = client
        .ec2
        .describe_instances()
        .filters(
            Filter::builder()
                .name(format!("tag:{MANAGED_BY_TAG_KEY}"))
                .values(MANAGED_BY_TAG_VALUE)
                .build(),
        )
        .filters(
            Filter::builder()
                .name("instance-state-name")
                .set_values(Some(
                    LISTED_INSTANCE_STATE_NAMES
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                ))
                .build(),
        )
        .send()
        .await?;

    let instances: Vec<InstanceInfo> = response
        .reservations()
        .iter()
        .flat_map(Reservation::instances)
        .filter_map(|instance| build_instance_info(instance, region))
        .collect();
    debug!(
        "[AWS] Found {} instances in region {region}",
        instances.len()
    );
    Ok(instances)
}

fn build_instance_tags(spawn_id: &str) -> TagSpecification {
    TagSpecification::builder()
        .resource_type(ResourceType::Instance)
        .tags(
            Tag::builder()
                .key(NAME_TAG_KEY)
                .value(SERVER_INSTANCE_NAME)
                .build(),
        )
        .tags(
            Tag::builder()
                .key(MANAGED_BY_TAG_KEY)
                .value(MANAGED_BY_TAG_VALUE)
                .build(),
        )
        .tags(Tag::builder().key(SPAWN_ID_TAG_KEY).value(spawn_id).build())
        .build()
}

async fn get_public_ips(client: &AwsClient, instance_id: &str) -> Result<(String, String)> {
    let response = client
        .ec2
        .describe_instances()
        .instance_ids(instance_id)
        .send()
        .await?;
    let instances: Vec<&Instance> = response
        .reservations()
        .iter()
        .flat_map(Reservation::instances)
        .collect();

    let public_ip_v4 = instances
        .iter()
        .find_map(|instance| instance.public_ip_address())
        .ok_or_else(|| AwsError::MissingPublicIpv4 {
            instance_id: instance_id.to_string(),
        })?
        .to_string();
    let public_ip_v6 = instances
        .iter()
        .find_map(|instance| instance.ipv6_address())
        .ok_or_else(|| AwsError::MissingPublicIpv6 {
            instance_id: instance_id.to_string(),
        })?
        .to_string();
    Ok((public_ip_v4, public_ip_v6))
}

fn build_instance_info(instance: &Instance, region: &str) -> Option<InstanceInfo> {
    let state_name = instance
        .state()
        .and_then(|instance_state| instance_state.name())
        .map_or("unknown", InstanceStateName::as_str);
    let launched_at = instance.launch_time().and_then(|timestamp| {
        DateTime::parse_from_rfc3339(&timestamp.to_string())
            .ok()
            .map(|datetime| datetime.with_timezone(&Utc))
    });

    Some(InstanceInfo {
        id: instance.instance_id()?.to_string(),
        name: find_tag_value(instance, NAME_TAG_KEY),
        region: region.to_string(),
        state: Ec2InstanceState::from_state_name(state_name).into(),
        error_reason: None,
        public_ip_v4: instance.public_ip_address()?.to_string(),
        public_ip_v6: instance.ipv6_address()?.to_string(),
        provider: CloudProviderName::Aws,
        instance_type: instance
            .instance_type()
            .map(|instance_type| instance_type.as_str().to_string())
            .unwrap_or_default(),
        launched_at,
        spawn_id: find_tag_value(instance, SPAWN_ID_TAG_KEY),
    })
}

fn find_tag_value(instance: &Instance, key: &str) -> Option<String> {
    instance
        .tags()
        .iter()
        .find(|tag| tag.key() == Some(key))
        .and_then(|tag| tag.value())
        .map(ToString::to_string)
}
