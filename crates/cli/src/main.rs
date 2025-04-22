use std::thread;

use byocvpn_aws::AwsProvider;
use byocvpn_core::cloud_provider::CloudProvider;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let aws = AwsProvider::new().await.unwrap();

    let instance_id = aws.spawn_instance().await.unwrap();
    println!("Spawned instance: {}", instance_id);

    thread::sleep(Duration::from_secs(10));

    aws.terminate_instance(&instance_id).await.unwrap();
    println!("Terminated instance: {}", instance_id);
}
