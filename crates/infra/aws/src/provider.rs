use async_trait::async_trait;
use aws_sdk_ec2::Client as Ec2Client;
use aws_sdk_ssm::Client as SsmClient;
use byocvpn_core::cloud_provider::{CloudProvider, InstanceInfo};

use crate::{config, instance, network};

pub struct AwsProvider {
    pub ec2_client: Ec2Client,
    pub ssm_client: SsmClient,
}

impl AwsProvider {
    pub async fn new(region: &Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let config = config::get_config(region).await?;
        let ec2_client = aws_sdk_ec2::Client::new(&config);
        let ssm_client = aws_sdk_ssm::Client::new(&config);
        Ok(Self {
            ec2_client,
            ssm_client,
        })
    }
}

#[async_trait]
impl CloudProvider for AwsProvider {
    async fn setup(&self) -> Result<(), Box<dyn std::error::Error>> {
        let existing_vpc_id = network::get_vpc_by_name(&self.ec2_client, "byocvpn-vpc")
            .await
            .unwrap();
        if existing_vpc_id.is_some() {
            println!("Existing VPC found, skipping creation.");
            return Ok(());
        }
        let vpc_id = network::create_vpc(&self.ec2_client, "10.0.0.0/16", "byocvpn-vpc").await?;
        let igw_id = network::create_and_attach_igw(&self.ec2_client, &vpc_id).await?;
        let main_route_table_id = network::find_main_route_table(&self.ec2_client, &vpc_id).await?;
        network::tag_resource_with_name(
            &self.ec2_client,
            &main_route_table_id,
            "byocvpn-main-route-table",
        )
        .await?;
        network::tag_resource_with_name(&self.ec2_client, &igw_id, "byocvpn-igw").await?;
        network::add_igw_routes_to_table(&self.ec2_client, &main_route_table_id, &igw_id).await?;

        Ok(())
    }

    async fn enable_region(&self, _region: &str) -> Result<(), Box<dyn std::error::Error>> {
        let vpc_id = network::get_vpc_by_name(&self.ec2_client, "byocvpn-vpc")
            .await
            .unwrap()
            .unwrap();

        let vpc_ipv6_cidr = network::get_vpc_ipv6_block(&self.ec2_client, &vpc_id)
            .await
            .unwrap();

        let azs = network::list_availability_zones(&self.ec2_client).await?;

        let subnets = network::get_subnets_in_vpc(&self.ec2_client, &vpc_id).await?;

        for (i, az) in azs.iter().enumerate() {
            let subnet_name = format!("byocvpn-subnet-{az}");

            // Check if this subnet already exists
            let already_exists = subnets.iter().any(|subnet| {
                subnet
                    .tags()
                    .iter()
                    .any(|tag| tag.key() == Some("Name") && tag.value() == Some(&subnet_name))
            });

            if already_exists {
                println!("Subnet {subnet_name} already exists, skipping.");
                continue;
            }
            let cidr = format!("10.0.{}.0/24", i); // safely spaced /20s
            let ipv6_cidr = network::carve_ipv6_subnet(&vpc_ipv6_cidr, i as u8).unwrap();

            let subnet_id = network::create_subnet(
                &self.ec2_client,
                &vpc_id,
                &cidr,
                &ipv6_cidr,
                az,
                &subnet_name,
            )
            .await?;

            network::enable_auto_ip_assign(&self.ec2_client, &subnet_id).await?;
        }

        let new_group_id = network::create_security_group(
            &self.ec2_client,
            &vpc_id,
            "byocvpn-security-group",
            "BYOC VPN server",
        )
        .await?;
        println!("Created new security group: {}", new_group_id);

        Ok(())
    }

    async fn spawn_instance(
        &self,
        server_private_key: &str,
        client_public_key: &str,
    ) -> Result<(String, String, String), Box<dyn std::error::Error>> {
        let vpc_id = network::get_vpc_by_name(&self.ec2_client, "byocvpn-vpc")
            .await
            .unwrap()
            .unwrap();

        let subnets = network::get_subnets_in_vpc(&self.ec2_client, &vpc_id).await?;

        let subnet_id = subnets[0].subnet_id.clone().unwrap();
        instance::spawn_instance(self, &subnet_id, server_private_key, client_public_key).await
    }

    async fn terminate_instance(
        &self,
        instance_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        instance::terminate_instance(&self.ec2_client, instance_id).await
    }

    async fn list_instances(&self) -> Result<Vec<InstanceInfo>, Box<dyn std::error::Error>> {
        instance::list_instances(&self.ec2_client).await
    }
}
