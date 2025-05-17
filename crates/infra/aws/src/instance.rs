use std::time::Duration;

use crate::AwsProvider;

use aws_sdk_ec2::Client as Ec2Client;
use aws_sdk_ec2::client::Waiters;
use aws_sdk_ec2::types::{ResourceType, Tag, TagSpecification};
use base64::{Engine, engine::general_purpose};

use byocvpn_core::cloud_provider::InstanceInfo;

use crate::cloud_init;
use crate::config;
use crate::network;

pub(super) async fn spawn_instance(
    provider: &AwsProvider,
    subnet_id: &str,
    server_private_key: &str,
    client_public_key: &str,
) -> Result<(String, String, String), Box<dyn std::error::Error>> {
    let user_data =
        cloud_init::generate_wireguard_cloud_init(server_private_key, client_public_key);

    println!("{:?}", user_data);
    let encoded_user_data = general_purpose::STANDARD.encode(user_data);

    let ami_id = config::get_al2023_ami(&provider.ssm_client).await?;
    println!("AMI ID: {}", ami_id);

    let group_name = "byocvpn-server";
    let security_group_id =
        network::get_security_group_by_name(&provider.ec2_client, group_name).await?;
    let group_id = match security_group_id {
        Some(id) => id,
        None => {
            let new_group_id =
                network::create_security_group(&provider.ec2_client, group_name, "BYOC VPN server")
                    .await?;
            println!("Created new security group: {}", new_group_id);
            new_group_id
        }
    };

    println!("Security group ID: {}", group_id);

    let tags = TagSpecification::builder()
        .resource_type(ResourceType::Instance)
        .tags(Tag::builder().key("Name").value("byocvpn-server").build())
        .build();
    let resp = provider
        .ec2_client
        .run_instances()
        .subnet_id(subnet_id)
        .image_id(ami_id)
        .security_group_ids(group_id)
        .instance_type(aws_sdk_ec2::types::InstanceType::T2Micro)
        .user_data(encoded_user_data)
        // .key_name("vpn")
        .min_count(1)
        .max_count(1)
        .tag_specifications(tags)
        .send()
        .await?;
    let instance = resp.instances().first().ok_or("No instance found")?;
    let instance_id = instance.instance_id().ok_or("No instance ID")?.to_string();

    provider
        .ec2_client
        .wait_until_instance_running()
        .instance_ids(&instance_id)
        .wait(Duration::from_secs(60))
        .await?;

    let desc = provider
        .ec2_client
        .describe_instances()
        .instance_ids(&instance_id)
        .send()
        .await?;

    let public_ip_v4 = desc
        .reservations()
        .iter()
        .flat_map(|r| r.instances())
        .filter_map(|i| i.public_ip_address())
        .next()
        .ok_or("No public IP address yet")?
        .to_string();

    let public_ip_v6 = desc
        .reservations()
        .iter()
        .flat_map(|r| r.instances())
        .filter_map(|i| i.ipv6_address())
        .next()
        .ok_or("No public IPv6 address yet")?
        .to_string();
    println!("Instance ID: {}", instance_id);
    println!("Public IPv4: {}", public_ip_v4);
    println!("Public IPv6: {}", public_ip_v6);

    Ok((instance_id, public_ip_v4, public_ip_v6))
}

pub async fn terminate_instance(
    ec2_client: &Ec2Client,
    instance_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    ec2_client
        .terminate_instances()
        .instance_ids(instance_id)
        .send()
        .await?;

    Ok(())
}

pub(super) async fn list_instances(
    ec2_client: &Ec2Client,
) -> Result<Vec<InstanceInfo>, Box<dyn std::error::Error>> {
    let resp = ec2_client.describe_instances().send().await?;

    let instances = resp
        .reservations()
        .iter()
        .flat_map(|r| r.instances())
        .filter_map(|i| {
            let id = i.instance_id()?.to_string();

            let state = i
                .state()
                .and_then(|s| s.name().map(|s| s.as_str()))
                .unwrap_or("unknown")
                .to_string();

            if state != "running" {
                return None;
            }
            let name = i
                .tags()
                .iter()
                .find(|t| t.key().unwrap_or_default() == "Name")
                .and_then(|t| t.value().map(|v| v.to_string()));
            let public_ip_v4 = i
                .public_ip_address()
                .map(|ip| ip.to_string())
                .unwrap_or_default();

            let public_ip_v6 = i
                .ipv6_address()
                .map(|ip| ip.to_string())
                .unwrap_or_default();

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
