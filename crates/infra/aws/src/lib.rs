use std::time::Duration;

use async_trait::async_trait;
use aws_config::{SdkConfig, meta::region::RegionProviderChain};
use aws_sdk_ec2::types::Ipv6Range;
use aws_sdk_ec2::types::{ResourceType, Tag, TagSpecification};
use aws_sdk_ec2::{
    Client as Ec2Client,
    types::{IpPermission, IpRange},
};
use aws_sdk_ec2::{config::Region, types::Filter};
use aws_sdk_ssm::Client as SsmClient;
use base64::{Engine, engine::general_purpose};
use byocvpn_core::cloud_provider::CloudProvider;
use byocvpn_core::cloud_provider::InstanceInfo;
use tokio::time::sleep;

pub struct AwsProvider {
    ec2_client: Ec2Client,
    ssm_client: SsmClient,
}

impl AwsProvider {
    async fn get_config(region: &Option<String>) -> Result<SdkConfig, Box<dyn std::error::Error>> {
        let region_provider = match region {
            Some(r) => RegionProviderChain::first_try(Region::new(r.clone())).or_default_provider(),
            None => RegionProviderChain::default_provider(),
        };
        let config = aws_config::from_env().region(region_provider).load().await;
        Ok(config)
    }
    pub async fn new(region: &Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Self::get_config(region).await?;
        let ec2_client = aws_sdk_ec2::Client::new(&config);
        let ssm_client = aws_sdk_ssm::Client::new(&config);
        Ok(Self {
            ec2_client,
            ssm_client,
        })
    }

    pub async fn get_al2023_ami(&self) -> Result<String, Box<dyn std::error::Error>> {
        // AL2023 x86_64 SSM parameter name
        let param_name = "/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-x86_64";

        // Fetch the parameter value (AMI ID)
        let result = self
            .ssm_client
            .get_parameter()
            .name(param_name)
            .send()
            .await?;

        let ami_id = result
            .parameter()
            .and_then(|p| p.value())
            .ok_or("AMI ID not found in parameter store")?
            .to_string();

        Ok(ami_id)
    }

    pub async fn create_security_group(
        &self,
        group_name: &str,
        description: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Set region provider

        // 1. Create security group
        let create_resp = self
            .ec2_client
            .create_security_group()
            .group_name(group_name)
            .description(description)
            .send()
            .await?;

        let group_id = create_resp
            .group_id()
            .ok_or("No security group ID returned")?
            .to_string();

        println!("Created security group with ID: {}", group_id);

        // 2. Authorize SSH ingress from anywhere (0.0.0.0/0)
        self.ec2_client
            .authorize_security_group_ingress()
            .group_id(&group_id)
            .ip_permissions(
                IpPermission::builder()
                    .ip_protocol("udp")
                    .from_port(51820)
                    .to_port(51820)
                    .ip_ranges(IpRange::builder().cidr_ip("0.0.0.0/0").build())
                    .ipv6_ranges(Ipv6Range::builder().cidr_ipv6("::/0").build())
                    .build(),
            )
            .send()
            .await?;

        println!("Added SSH ingress rule to security group");

        Ok(group_id)
    }

    pub async fn get_byocvpn_sg_id(
        &self,
        group_name: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let filters = Filter::builder()
            .name("group-name")
            .values(group_name)
            .build();

        let resp = self
            .ec2_client
            .describe_security_groups()
            .filters(filters)
            .send()
            .await?;

        let group_id = resp
            .security_groups()
            .first()
            .and_then(|sg| sg.group_id())
            .map(|s| s.to_string());

        Ok(group_id)
    }

    pub async fn get_console_output(
        &self,
        instance_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = self
            .ec2_client
            .get_console_output()
            .instance_id(instance_id)
            .send()
            .await?;

        if let Some(encoded_output) = output.output() {
            let decoded = general_purpose::STANDARD.decode(encoded_output)?;
            let text = String::from_utf8_lossy(&decoded);
            println!("🧾 Console Output:\n{}", text);
        } else {
            println!("⚠️ No console output available yet. Try again in a few seconds.");
        }

        Ok(())
    }

    fn wireguard_cloud_init(&self, server_private_key: &str, client_public_key: &str) -> String {
        let wg_config = format!(
            r#"[Interface]
PrivateKey={server_private_key}
Address=10.66.66.1/24,fd86:ea04:1111::1/64
ListenPort=51820
PostUp = iptables -A FORWARD -i enX0 -j ACCEPT
PostUp = iptables -t nat -A POSTROUTING -o enX0 -j MASQUERADE
PostUp = ip6tables -A FORWARD -i enX0 -j ACCEPT
PostUp = ip6tables -t nat -A POSTROUTING -o enX0 -j MASQUERADE
PostDown = iptables -D FORWARD -i enX0 -j ACCEPT
PostDown = iptables -t nat -D POSTROUTING -o enX0 -j MASQUERADE
PostDown = ip6tables -D FORWARD -i enX0 -j ACCEPT
PostDown = ip6tables -t nat -D POSTROUTING -o enX0 -j MASQUERADE

[Peer]
PublicKey={client_public_key}
AllowedIPs=10.66.66.2/32,fd86:ea04:1111::2/128
"#
        );

        let user_data = format!(
            r#"#!/bin/bash
dnf install -y wireguard-tools iptables-services

cat>/etc/wireguard/wg0.conf<<'EOF'
{wg_config}
EOF

echo "net.ipv4.ip_forward=1">>/etc/sysctl.conf
echo "net.ipv6.conf.all.forwarding=1">>/etc/sysctl.conf
sysctl -p

systemctl start wg-quick@wg0
systemctl enable wg-quick@wg0
"#
        );
        return user_data;
    }
}

#[async_trait]
impl CloudProvider for AwsProvider {
    async fn spawn_instance(
        &self,
        server_private_key: &str,
        client_public_key: &str,
    ) -> Result<(String, String, String), Box<dyn std::error::Error>> {
        let user_data = self.wireguard_cloud_init(server_private_key, client_public_key);

        println!("{:?}", user_data);
        let encoded_user_data = general_purpose::STANDARD.encode(user_data);

        let ami_id = self.get_al2023_ami().await?;
        println!("AMI ID: {}", ami_id);

        let group_name = "byocvpn-server";
        let security_group_id = self.get_byocvpn_sg_id(group_name).await?;
        let group_id = match security_group_id {
            Some(id) => id,
            None => {
                let new_group_id = self
                    .create_security_group(group_name, "BYOC VPN server")
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
        let resp = self
            .ec2_client
            .run_instances()
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

        for _ in 0..150 {
            let desc = self
                .ec2_client
                .describe_instances()
                .instance_ids(&instance_id)
                .send()
                .await?;

            if let Some(state) = desc
                .reservations()
                .iter()
                .flat_map(|r| r.instances())
                .flat_map(|i| i.state().and_then(|s| s.name().to_owned()))
                .next()
            {
                if state.as_str() == "running" {
                    break;
                }
            }
            sleep(Duration::from_secs(2)).await;
        }

        let desc = self
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

    async fn terminate_instance(
        &self,
        instance_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.ec2_client
            .terminate_instances()
            .instance_ids(instance_id)
            .send()
            .await?;

        Ok(())
    }

    async fn list_instances(&self) -> Result<Vec<InstanceInfo>, Box<dyn std::error::Error>> {
        let resp = self.ec2_client.describe_instances().send().await?;

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
}
