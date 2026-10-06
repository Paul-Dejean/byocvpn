use std::collections::HashMap;

use aws_sdk_iam::Client as IamClient;
use aws_sdk_iam::types::PolicyEvaluationDecisionType;
use aws_sdk_sts::Client as StsClient;
use byocvpn_core::{
    cloud_provider::PermissionStatus,
    error::{Error, Result},
};
use log::*;

use crate::aws_error::map_aws_error;

pub(crate) const REQUIRED_ACTIONS: &[&str] = &[
    "ec2:RunInstances",
    "ec2:TerminateInstances",
    "ec2:DescribeInstances",
    "ec2:CreateVpc",
    "ec2:DescribeVpcs",
    "ec2:CreateSubnet",
    "ec2:DescribeSubnets",
    "ec2:ModifySubnetAttribute",
    "ec2:CreateSecurityGroup",
    "ec2:DescribeSecurityGroups",
    "ec2:AuthorizeSecurityGroupIngress",
    "ec2:RevokeSecurityGroupIngress",
    "ec2:CreateTags",
    "ec2:DescribeAvailabilityZones",
    "ec2:CreateInternetGateway",
    "ec2:AttachInternetGateway",
    "ec2:DescribeInternetGateways",
    "ec2:CreateRoute",
    "ec2:DescribeRouteTables",
    "ec2:DescribeRegions",
    "ssm:GetParameter",
];

const GET_CALLER_IDENTITY_OPERATION: &str = "sts:GetCallerIdentity";
pub(crate) const SIMULATE_POLICY_OPERATION: &str = "iam:SimulatePrincipalPolicy";
const ASSUMED_ROLE_MARKER: &str = ":assumed-role/";

pub(super) async fn verify_permissions(
    sts_client: &StsClient,
    iam_client: &IamClient,
) -> Result<Vec<PermissionStatus>> {
    let principal_arn = get_principal_arn(sts_client).await?;
    info!("permission check: simulating policies for {principal_arn}");
    let decisions = simulate_required_actions(iam_client, &principal_arn).await?;

    let statuses = REQUIRED_ACTIONS
        .iter()
        .map(|action| {
            let granted = decisions.get(*action).copied().unwrap_or(false);
            if granted {
                info!("permission check {action}: allowed");
            } else {
                warn!("permission check {action}: denied");
            }
            PermissionStatus {
                permission: action.to_string(),
                granted,
            }
        })
        .collect();
    Ok(statuses)
}

async fn get_principal_arn(sts_client: &StsClient) -> Result<String> {
    let identity = sts_client
        .get_caller_identity()
        .send()
        .await
        .map_err(|error| map_aws_error(GET_CALLER_IDENTITY_OPERATION, error))?;
    let caller_arn = identity.arn().ok_or_else(|| Error::Unknown {
        operation_name: GET_CALLER_IDENTITY_OPERATION.to_string(),
        detail: "response did not include a caller ARN".to_string(),
    })?;
    Ok(build_policy_source_arn(caller_arn))
}

fn build_policy_source_arn(caller_arn: &str) -> String {
    let Some((arn_prefix, role_path)) = caller_arn.split_once(ASSUMED_ROLE_MARKER) else {
        return caller_arn.to_string();
    };
    let role_name = role_path.split('/').next().unwrap_or(role_path);
    let iam_prefix = arn_prefix.replacen(":sts:", ":iam:", 1);
    format!("{iam_prefix}:role/{role_name}")
}

async fn simulate_required_actions(
    iam_client: &IamClient,
    principal_arn: &str,
) -> Result<HashMap<String, bool>> {
    let action_names: Vec<String> = REQUIRED_ACTIONS
        .iter()
        .map(|action| action.to_string())
        .collect();
    let mut decisions = HashMap::new();
    let mut marker: Option<String> = None;

    loop {
        let response = iam_client
            .simulate_principal_policy()
            .policy_source_arn(principal_arn)
            .set_action_names(Some(action_names.clone()))
            .set_marker(marker)
            .send()
            .await
            .map_err(|error| map_aws_error(SIMULATE_POLICY_OPERATION, error))?;

        for evaluation in response.evaluation_results() {
            let allowed = evaluation.eval_decision() == &PolicyEvaluationDecisionType::Allowed;
            decisions.insert(evaluation.eval_action_name().to_string(), allowed);
        }

        marker = response.marker().map(str::to_string);
        if !response.is_truncated() || marker.is_none() {
            break;
        }
    }

    Ok(decisions)
}

#[cfg(test)]
mod tests {
    use super::build_policy_source_arn;

    #[test]
    fn keeps_user_arn_unchanged() {
        let user_arn = "arn:aws:iam::123456789012:user/byocvpn";
        assert_eq!(build_policy_source_arn(user_arn), user_arn);
    }

    #[test]
    fn converts_assumed_role_session_to_role_arn() {
        let session_arn = "arn:aws:sts::123456789012:assumed-role/Deployer/session-1";
        assert_eq!(
            build_policy_source_arn(session_arn),
            "arn:aws:iam::123456789012:role/Deployer"
        );
    }
}
