use aws_sdk_ec2::Client as Ec2Client;
use aws_sdk_iam::Client as IamClient;
use aws_sdk_pricing::Client as PricingClient;
use aws_sdk_ssm::Client as SsmClient;
use aws_sdk_sts::Client as StsClient;

use crate::{auth::load_sdk_config, credentials::AwsCredentials};

const PRICING_API_REGION: &str = "us-east-1";

pub struct AwsClient {
    pub ec2: Ec2Client,
    pub ssm: SsmClient,
    pub sts: StsClient,
    pub iam: IamClient,
    pub pricing: PricingClient,
}

impl AwsClient {
    pub async fn connect(credentials: &AwsCredentials, region: Option<&str>) -> Self {
        let sdk_config = load_sdk_config(credentials, region).await;
        let pricing_sdk_config = load_sdk_config(credentials, Some(PRICING_API_REGION)).await;
        Self {
            ec2: Ec2Client::new(&sdk_config),
            ssm: SsmClient::new(&sdk_config),
            sts: StsClient::new(&sdk_config),
            iam: IamClient::new(&sdk_config),
            pricing: PricingClient::new(&pricing_sdk_config),
        }
    }
}
