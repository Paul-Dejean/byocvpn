use byocvpn_core::{
    cloud_provider::PricingInfo,
    error::{ComputeProvisioningError, Error, NetworkProvisioningError, Result},
};
use reqwest::Client as HttpClient;
use serde::Deserialize;
use tokio::time::Duration;

use crate::instance::{INSTANCE_MEMORY_GB, INSTANCE_OCPUS};

const PRICE_LIST_URL: &str = "https://apexapps.oracle.com/pls/apex/cetools/api/v1/products/";
const REQUEST_TIMEOUT_SECS: u64 = 15;
const CURRENCY_CODE: &str = "USD";
const PAY_AS_YOU_GO_MODEL: &str = "PAY_AS_YOU_GO";

const PUBLIC_IP_HOURLY_RATE: f64 = 0.0;
const BOOT_VOLUME_GB: f64 = 50.0;
const BALANCED_PERFORMANCE_UNITS_PER_GB: f64 = 10.0;

const BLOCK_VOLUME_STORAGE_PART: &str = "B91961";
const BLOCK_VOLUME_PERFORMANCE_PART: &str = "B91962";
const EGRESS_AMERICAS_EUROPE_PART: &str = "B88327";
const EGRESS_APAC_SOUTH_AMERICA_PART: &str = "B93455";
const EGRESS_MIDDLE_EAST_AFRICA_PART: &str = "B93456";

const SHAPE_PARTS: &[(&str, ShapeParts)] = &[
    (
        "VM.Standard.A1.Flex",
        ShapeParts {
            ocpu: "B93297",
            memory: "B93298",
        },
    ),
    (
        "VM.Standard.A2.Flex",
        ShapeParts {
            ocpu: "B109529",
            memory: "B109530",
        },
    ),
    (
        "VM.Standard.E4.Flex",
        ShapeParts {
            ocpu: "B93113",
            memory: "B93114",
        },
    ),
    (
        "VM.Standard.E5.Flex",
        ShapeParts {
            ocpu: "B97384",
            memory: "B97385",
        },
    ),
];

struct ShapeParts {
    ocpu: &'static str,
    memory: &'static str,
}

pub async fn fetch_pricing(region: &str, shape: &str) -> Result<PricingInfo> {
    let shape_parts = SHAPE_PARTS
        .iter()
        .find(|(name, _)| *name == shape)
        .map(|(_, parts)| parts)
        .ok_or_else(|| build_unavailable_error(region, shape, "shape has no known part numbers"))?;

    let http = HttpClient::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .map_err(|error| NetworkProvisioningError::CloudOperationFailed {
            reason: format!("failed to build HTTP client: {error}"),
        })?;

    let (ocpu_rate, memory_rate, storage_rate, performance_rate, egress_rate) = tokio::try_join!(
        fetch_pay_as_you_go_rate(&http, shape_parts.ocpu),
        fetch_pay_as_you_go_rate(&http, shape_parts.memory),
        fetch_pay_as_you_go_rate(&http, BLOCK_VOLUME_STORAGE_PART),
        fetch_pay_as_you_go_rate(&http, BLOCK_VOLUME_PERFORMANCE_PART),
        fetch_pay_as_you_go_rate(&http, select_egress_part(region)),
    )?;

    Ok(PricingInfo {
        hourly_rate: ocpu_rate.mul_add(
            f64::from(INSTANCE_OCPUS),
            memory_rate * f64::from(INSTANCE_MEMORY_GB),
        ),
        ip_hourly_rate: PUBLIC_IP_HOURLY_RATE,
        egress_rate_per_gb: egress_rate,
        storage_gb: BOOT_VOLUME_GB,
        storage_rate_per_gb_month: performance_rate
            .mul_add(BALANCED_PERFORMANCE_UNITS_PER_GB, storage_rate),
    })
}

fn select_egress_part(region: &str) -> &'static str {
    if region.starts_with("ap-") || region.starts_with("sa-") {
        EGRESS_APAC_SOUTH_AMERICA_PART
    } else if region.starts_with("me-") || region.starts_with("af-") || region.starts_with("il-") {
        EGRESS_MIDDLE_EAST_AFRICA_PART
    } else {
        EGRESS_AMERICAS_EUROPE_PART
    }
}

async fn fetch_pay_as_you_go_rate(http: &HttpClient, part_number: &str) -> Result<f64> {
    let response = http
        .get(PRICE_LIST_URL)
        .query(&[("partNumber", part_number)])
        .send()
        .await
        .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
            reason: format!("Oracle price list request for {part_number} failed: {error}"),
        })?;

    let status = response.status();
    if !status.is_success() {
        return Err(NetworkProvisioningError::NetworkQueryFailed {
            reason: format!("Oracle price list returned HTTP {status} for {part_number}"),
        }
        .into());
    }

    let page: PriceListPage =
        response
            .json()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!(
                    "Oracle price list response for {part_number} is not valid JSON: {error}"
                ),
            })?;

    page.items
        .iter()
        .flat_map(|product| &product.currency_code_localizations)
        .filter(|localization| localization.currency_code == CURRENCY_CODE)
        .flat_map(|localization| &localization.prices)
        .filter(|price| price.model == PAY_AS_YOU_GO_MODEL)
        .map(|price| price.value)
        .reduce(f64::max)
        .ok_or_else(|| {
            ComputeProvisioningError::PricingUnavailable {
                region: String::new(),
                instance_type: part_number.to_string(),
                reason: "no pay-as-you-go USD price in the price list".to_string(),
            }
            .into()
        })
}

fn build_unavailable_error(region: &str, shape: &str, reason: &str) -> Error {
    ComputeProvisioningError::PricingUnavailable {
        region: region.to_string(),
        instance_type: shape.to_string(),
        reason: reason.to_string(),
    }
    .into()
}

#[derive(Deserialize)]
struct PriceListPage {
    #[serde(default)]
    items: Vec<PriceListProduct>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PriceListProduct {
    #[serde(default)]
    currency_code_localizations: Vec<CurrencyLocalization>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurrencyLocalization {
    currency_code: String,
    #[serde(default)]
    prices: Vec<PriceListPrice>,
}

#[derive(Deserialize)]
struct PriceListPrice {
    model: String,
    value: f64,
}
