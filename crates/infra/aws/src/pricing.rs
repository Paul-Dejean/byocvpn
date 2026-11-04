use std::collections::HashMap;

use aws_sdk_pricing::{
    Client as PricingClient,
    error::DisplayErrorContext,
    types::{Filter, FilterType},
};
use byocvpn_core::cloud_provider::PricingInfo;
use byocvpn_core::error::{ComputeProvisioningError, Error, Result};
use serde::Deserialize;

pub const PRICING_API_REGION: &str = "us-east-1";

const EC2_SERVICE_CODE: &str = "AmazonEC2";
const VPC_SERVICE_CODE: &str = "AmazonVPC";
const DATA_TRANSFER_SERVICE_CODE: &str = "AWSDataTransfer";
const CURRENCY_CODE: &str = "USD";

const ROOT_VOLUME_GB: f64 = 8.0;
const ROOT_VOLUME_API_NAME: &str = "gp3";
const PUBLIC_IPV4_USAGE_TYPE_FRAGMENT: &str = "PublicIPv4:InUseAddress";

pub async fn fetch_pricing(
    pricing_client: &PricingClient,
    region: &str,
    instance_type: &str,
) -> Result<PricingInfo> {
    let instance_filters = build_filters(&[
        ("regionCode", region),
        ("instanceType", instance_type),
        ("operatingSystem", "Linux"),
        ("tenancy", "Shared"),
        ("preInstalledSw", "NA"),
        ("capacitystatus", "Used"),
        ("licenseModel", "No License required"),
    ])?;
    let mut public_ip_filters =
        build_filters(&[("regionCode", region), ("productFamily", "IP Address")])?;
    public_ip_filters.push(build_filter(
        FilterType::Contains,
        "usagetype",
        PUBLIC_IPV4_USAGE_TYPE_FRAGMENT,
    )?);
    let volume_filters = build_filters(&[
        ("regionCode", region),
        ("productFamily", "Storage"),
        ("volumeApiName", ROOT_VOLUME_API_NAME),
    ])?;
    let egress_filters = build_filters(&[
        ("fromRegionCode", region),
        ("transferType", "AWS Outbound"),
        ("toLocation", "External"),
    ])?;

    let (hourly_rate, ip_hourly_rate, storage_rate_per_gb_month, egress_rate_per_gb) =
        tokio::try_join!(
            fetch_first_paid_rate(
                pricing_client,
                EC2_SERVICE_CODE,
                instance_filters,
                "instance"
            ),
            fetch_first_paid_rate(
                pricing_client,
                VPC_SERVICE_CODE,
                public_ip_filters,
                "public IPv4"
            ),
            fetch_first_paid_rate(
                pricing_client,
                EC2_SERVICE_CODE,
                volume_filters,
                "root volume"
            ),
            fetch_first_paid_rate(
                pricing_client,
                DATA_TRANSFER_SERVICE_CODE,
                egress_filters,
                "internet egress"
            ),
        )
        .map_err(|error| error.with_context(format!("pricing {instance_type} in {region}")))?;

    Ok(PricingInfo {
        hourly_rate,
        ip_hourly_rate,
        egress_rate_per_gb,
        storage_gb: ROOT_VOLUME_GB,
        storage_rate_per_gb_month,
    })
}

async fn fetch_first_paid_rate(
    pricing_client: &PricingClient,
    service_code: &str,
    filters: Vec<Filter>,
    component: &str,
) -> Result<f64> {
    let response = pricing_client
        .get_products()
        .service_code(service_code)
        .set_filters(Some(filters))
        .send()
        .await
        .map_err(|error| {
            build_unavailable_error(component, &DisplayErrorContext(&error).to_string())
        })?;

    response
        .price_list()
        .iter()
        .filter_map(|price_list_json| serde_json::from_str::<PriceListEntry>(price_list_json).ok())
        .flat_map(|entry| entry.terms.on_demand.into_values())
        .flat_map(|offer| offer.price_dimensions.into_values())
        .filter_map(|dimension| {
            let price = dimension
                .price_per_unit
                .get(CURRENCY_CODE)?
                .parse::<f64>()
                .ok()?;
            let begin_range = dimension.begin_range.parse::<f64>().unwrap_or(0.0);
            (price > 0.0).then_some((begin_range, price))
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, price)| price)
        .ok_or_else(|| build_unavailable_error(component, "no on-demand USD price returned"))
}

fn build_filters(fields: &[(&str, &str)]) -> Result<Vec<Filter>> {
    fields
        .iter()
        .map(|(field, value)| build_filter(FilterType::TermMatch, field, value))
        .collect()
}

fn build_filter(filter_type: FilterType, field: &str, value: &str) -> Result<Filter> {
    Filter::builder()
        .r#type(filter_type)
        .field(field)
        .value(value)
        .build()
        .map_err(|error| build_unavailable_error(field, &error.to_string()))
}

fn build_unavailable_error(component: &str, reason: &str) -> Error {
    ComputeProvisioningError::PricingUnavailable {
        region: String::new(),
        instance_type: component.to_string(),
        reason: reason.to_string(),
    }
    .into()
}

#[derive(Deserialize)]
struct PriceListEntry {
    terms: Terms,
}

#[derive(Deserialize)]
struct Terms {
    #[serde(rename = "OnDemand", default)]
    on_demand: HashMap<String, OfferTerm>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OfferTerm {
    #[serde(default)]
    price_dimensions: HashMap<String, PriceDimension>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PriceDimension {
    #[serde(default)]
    begin_range: String,
    #[serde(default)]
    price_per_unit: HashMap<String, String>,
}
