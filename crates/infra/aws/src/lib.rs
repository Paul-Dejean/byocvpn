use async_trait::async_trait;
use aws_sdk_ec2::Client;
use byocvpn_core::cloud_provider::CloudProvider;

pub struct AwsProvider {
    pub client: Client,
}

impl AwsProvider {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_ec2::Client::new(&config);
        Ok(Self { client })
    }
}

#[async_trait]
impl CloudProvider for AwsProvider {
    async fn spawn_instance(&self) -> Result<String, Box<dyn std::error::Error>> {
        let resp = self
            .client
            .run_instances()
            .image_id("ami-0b198a85d03bfa122") // use a proper VPN-ready AMI
            .instance_type(aws_sdk_ec2::types::InstanceType::T2Micro)
            .min_count(1)
            .max_count(1)
            .send()
            .await?;

        let instance_id = resp
            .instances()
            .first()
            .and_then(|i| i.instance_id())
            .map(|id| id.to_string())
            .ok_or("No instance ID returned")?;

        Ok(instance_id)
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
}
