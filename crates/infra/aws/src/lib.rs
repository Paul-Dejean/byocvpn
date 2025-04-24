use std::time::Duration;

use async_trait::async_trait;
use aws_sdk_ec2::Client;
use aws_sdk_ec2::types::{ResourceType, Tag, TagSpecification};
use base64::{Engine, engine::general_purpose};
use byocvpn_core::cloud_provider::CloudProvider;
use byocvpn_core::cloud_provider::InstanceInfo;
use tokio::time::sleep;

pub struct AwsProvider {
    pub client: Client,
}

impl AwsProvider {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_ec2::Client::new(&config);
        Ok(Self { client })
    }

    pub async fn get_console_output(
        &self,
        instance_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = self
            .client
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
Address=10.66.66.1/24
ListenPort=51820
PostUp=iptables -A FORWARD -i enX0 -j ACCEPT
PostUp=iptables -t nat -A POSTROUTING -o enX0 -j MASQUERADE
PostDown=iptables -D FORWARD -i enX0 -j ACCEPT
PostDown=iptables -t nat -D POSTROUTING -o enX0 -j MASQUERADE

[Peer]
PublicKey={client_public_key}
AllowedIPs=10.66.66.2/32
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
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let user_data = self.wireguard_cloud_init(server_private_key, client_public_key);

        println!("{:?}", user_data);
        let encoded_user_data = general_purpose::STANDARD.encode(user_data);

        let tags = TagSpecification::builder()
            .resource_type(ResourceType::Instance)
            .tags(Tag::builder().key("Name").value("byocvpn-server").build())
            .build();
        let resp = self
            .client
            .run_instances()
            .image_id("ami-0b198a85d03bfa122")
            .security_group_ids("sg-08decce6b8f86e674")
            .instance_type(aws_sdk_ec2::types::InstanceType::T2Micro)
            .user_data(encoded_user_data)
            .key_name("vpn")
            .min_count(1)
            .max_count(1)
            .tag_specifications(tags)
            .send()
            .await?;
        let instance = resp.instances().first().ok_or("No instance found")?;
        let instance_id = instance.instance_id().ok_or("No instance ID")?.to_string();

        for _ in 0..150 {
            let desc = self
                .client
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
            .client
            .describe_instances()
            .instance_ids(&instance_id)
            .send()
            .await?;

        let public_ip = desc
            .reservations()
            .iter()
            .flat_map(|r| r.instances())
            .filter_map(|i| i.public_ip_address())
            .next()
            .ok_or("No public IP address yet")?
            .to_string();

        Ok((instance_id, public_ip))
    }

    async fn terminate_instance(
        &self,
        instance_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.client
            .terminate_instances()
            .instance_ids(instance_id)
            .send()
            .await?;

        Ok(())
    }

    async fn list_instances(&self) -> Result<Vec<InstanceInfo>, Box<dyn std::error::Error>> {
        let resp = self.client.describe_instances().send().await?;
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
                let public_ip = i
                    .public_ip_address()
                    .map(|ip| ip.to_string())
                    .unwrap_or_default();

                Some(InstanceInfo {
                    id,
                    name,
                    state,
                    public_ip,
                })
            })
            .collect();
        Ok(instances)
    }
}
