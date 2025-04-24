use byocvpn_core::cloud_provider::CloudProvider;

pub async fn list_instances(aws: &dyn CloudProvider) -> Result<(), Box<dyn std::error::Error>> {
    let instances = aws
        .list_instances()
        .await
        .expect("Failed to list instances");
    println!("Active instances: {:?}", instances);
    Ok(())
}
