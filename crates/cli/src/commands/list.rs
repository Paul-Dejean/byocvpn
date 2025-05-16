use byocvpn_core::cloud_provider::CloudProvider;

pub async fn list_instances(aws: &dyn CloudProvider) -> Result<(), Box<dyn std::error::Error>> {
    match aws.list_instances().await {
        Ok(instances) => {
            println!("Active instances: {:?}", instances);
        }
        Err(e) => {
            eprintln!("Failed to list instances: {:?}", e.source());
            // You could also log `e.source()` for deeper info if needed
        }
    }
    Ok(())
}
