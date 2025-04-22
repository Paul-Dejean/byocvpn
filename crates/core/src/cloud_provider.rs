use async_trait::async_trait;

#[async_trait]
pub trait CloudProvider {
    async fn spawn_instance(&self) -> Result<String, Box<dyn std::error::Error>>;
    async fn terminate_instance(&self, instance_id: &str)
    -> Result<(), Box<dyn std::error::Error>>;
}
