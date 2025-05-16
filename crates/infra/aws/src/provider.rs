use async_trait::async_trait;
use aws_sdk_ec2::Client as Ec2Client;
use aws_sdk_ssm::Client as SsmClient;
use byocvpn_core::cloud_provider::CloudProvider;
use byocvpn_core::cloud_provider::InstanceInfo;

use crate::config;

use crate::instance;

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
        // self.create_vpc("10.0.0.0/16", "byocvpn-vpc").await?;
        Ok(())
    }

    async fn enable_region(&self, _region: &str) -> Result<(), Box<dyn std::error::Error>> {
        // self.create_subnet(vpc_id, cidr_block, az, name);

        Ok(())
    }

    async fn spawn_instance(
        &self,
        server_private_key: &str,
        client_public_key: &str,
    ) -> Result<(String, String, String), Box<dyn std::error::Error>> {
        instance::spawn_instance(self, server_private_key, client_public_key).await
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
