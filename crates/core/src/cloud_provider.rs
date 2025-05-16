use async_trait::async_trait;

#[async_trait]
pub trait CloudProvider {
    async fn spawn_instance(
        &self,
        server_private_key: &str,
        client_public_key: &str,
    ) -> Result<(String, String, String), Box<dyn std::error::Error>>;
    async fn terminate_instance(&self, instance_id: &str)
    -> Result<(), Box<dyn std::error::Error>>;
    async fn list_instances(&self) -> Result<Vec<InstanceInfo>, Box<dyn std::error::Error>>;
}

#[derive(Debug)]
pub struct InstanceInfo {
    pub id: String,
    pub name: Option<String>,
    pub state: String,
    pub public_ip_v4: String,
    pub public_ip_v6: String,
}
