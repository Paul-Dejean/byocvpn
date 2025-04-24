use byocvpn_core::cloud_provider::CloudProvider;

pub async fn terminate_instance(
    aws: &dyn CloudProvider,
    instance_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    aws.terminate_instance(&instance_id)
        .await
        .expect("Failed to terminate instance");
    println!("Terminated instance: {}", instance_id);
    Ok(())
}
