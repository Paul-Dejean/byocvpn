use byocvpn_core::cloud_provider::PricingInfo;
use byocvpn_core::error::{ComputeProvisioningError, Error, Result};
use serde::Deserialize;

use crate::client::GcpClient;

const BILLING_CATALOG_BASE_URL: &str = "https://cloudbilling.googleapis.com/v1";
const COMPUTE_ENGINE_SERVICE_NAME: &str = "services/6F81-5844-456A";
const SKU_PAGE_SIZE: u32 = 5000;
const GLOBAL_REGION: &str = "global";
const ON_DEMAND_USAGE_TYPE: &str = "OnDemand";
const CURRENCY_CODE: &str = "USD";
const NANOS_PER_UNIT: f64 = 1_000_000_000.0;

const BOOT_DISK_GB: f64 = 10.0;
const CORE_DESCRIPTION_PREFIX: &str = "E2 Instance Core running in";
const RAM_DESCRIPTION_PREFIX: &str = "E2 Instance Ram running in";
const EXTERNAL_IP_DESCRIPTION_PREFIX: &str = "External IP Charge on a Standard VM";
const BOOT_DISK_DESCRIPTION_PREFIX: &str = "Storage PD Capacity";
const EGRESS_DESCRIPTION_PREFIX: &str = "Network Internet Egress from";

const SHARED_CORE_MACHINE_SHAPES: &[(&str, MachineShape)] = &[
    (
        "e2-micro",
        MachineShape {
            vcpu_fraction: 0.25,
            memory_gb: 1.0,
        },
    ),
    (
        "e2-small",
        MachineShape {
            vcpu_fraction: 0.5,
            memory_gb: 2.0,
        },
    ),
    (
        "e2-medium",
        MachineShape {
            vcpu_fraction: 1.0,
            memory_gb: 4.0,
        },
    ),
];

struct MachineShape {
    vcpu_fraction: f64,
    memory_gb: f64,
}

pub async fn fetch_pricing(
    client: &GcpClient,
    region: &str,
    machine_type: &str,
) -> Result<PricingInfo> {
    let shape = SHARED_CORE_MACHINE_SHAPES
        .iter()
        .find(|(name, _)| *name == machine_type)
        .map(|(_, shape)| shape)
        .ok_or_else(|| {
            build_unavailable_error(region, machine_type, "machine type has no known shape")
        })?;

    let skus = list_compute_engine_skus(client).await?;
    let regional_skus: Vec<&Sku> = skus
        .iter()
        .filter(|sku| sku.category.usage_type == ON_DEMAND_USAGE_TYPE)
        .filter(|sku| {
            sku.service_regions
                .iter()
                .any(|service_region| service_region == region || service_region == GLOBAL_REGION)
        })
        .collect();
    let egress_destination = select_egress_destination(region);

    let core_rate = find_first_paid_rate(&regional_skus, |sku| {
        sku.category.resource_group == "CPU" && sku.description.starts_with(CORE_DESCRIPTION_PREFIX)
    })
    .ok_or_else(|| build_unavailable_error(region, machine_type, "no E2 core price"))?;
    let ram_rate = find_first_paid_rate(&regional_skus, |sku| {
        sku.category.resource_group == "RAM" && sku.description.starts_with(RAM_DESCRIPTION_PREFIX)
    })
    .ok_or_else(|| build_unavailable_error(region, machine_type, "no E2 memory price"))?;
    let ip_hourly_rate = find_first_paid_rate(&regional_skus, |sku| {
        sku.description.starts_with(EXTERNAL_IP_DESCRIPTION_PREFIX)
    })
    .ok_or_else(|| build_unavailable_error(region, machine_type, "no external IP price"))?;
    let storage_rate_per_gb_month = find_first_paid_rate(&regional_skus, |sku| {
        sku.category.resource_group == "PDStandard"
            && sku.description.starts_with(BOOT_DISK_DESCRIPTION_PREFIX)
    })
    .ok_or_else(|| build_unavailable_error(region, machine_type, "no boot disk price"))?;
    let egress_rate_per_gb = find_first_paid_rate(&regional_skus, |sku| {
        sku.category.resource_group == "PremiumInternetEgress"
            && sku.description.starts_with(EGRESS_DESCRIPTION_PREFIX)
            && sku.description.ends_with(egress_destination)
    })
    .ok_or_else(|| build_unavailable_error(region, machine_type, "no internet egress price"))?;

    Ok(PricingInfo {
        hourly_rate: core_rate.mul_add(shape.vcpu_fraction, ram_rate * shape.memory_gb),
        ip_hourly_rate,
        egress_rate_per_gb,
        storage_gb: BOOT_DISK_GB,
        storage_rate_per_gb_month,
    })
}

fn select_egress_destination(region: &str) -> &'static str {
    if region.starts_with("europe-") || region.starts_with("me-") || region.starts_with("africa-") {
        "to EMEA"
    } else if region.starts_with("asia-") || region.starts_with("australia-") {
        "to APAC"
    } else {
        "to Americas"
    }
}

fn find_first_paid_rate(skus: &[&Sku], matches: impl Fn(&Sku) -> bool) -> Option<f64> {
    skus.iter()
        .filter(|sku| matches(sku))
        .flat_map(|sku| &sku.pricing_info)
        .flat_map(|pricing_info| &pricing_info.pricing_expression.tiered_rates)
        .filter(|tiered_rate| tiered_rate.unit_price.currency_code == CURRENCY_CODE)
        .filter_map(|tiered_rate| {
            let price = tiered_rate.unit_price.get_amount();
            (price > 0.0).then_some((tiered_rate.start_usage_amount, price))
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, price)| price)
}

async fn list_compute_engine_skus(client: &GcpClient) -> Result<Vec<Sku>> {
    let mut skus = Vec::new();
    let mut page_token = String::new();
    loop {
        let url = format!(
            "{BILLING_CATALOG_BASE_URL}/{COMPUTE_ENGINE_SERVICE_NAME}/skus?pageSize={SKU_PAGE_SIZE}&pageToken={page_token}"
        );
        let page: SkuListResponse = client
            .get(&url)
            .await
            .map_err(|error| error.with_context("listing Compute Engine SKUs"))?;
        skus.extend(page.skus);
        match page.next_page_token {
            Some(next_page_token) if !next_page_token.is_empty() => page_token = next_page_token,
            _ => return Ok(skus),
        }
    }
}

fn build_unavailable_error(region: &str, machine_type: &str, reason: &str) -> Error {
    ComputeProvisioningError::PricingUnavailable {
        region: region.to_string(),
        instance_type: machine_type.to_string(),
        reason: reason.to_string(),
    }
    .into()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SkuListResponse {
    #[serde(default)]
    skus: Vec<Sku>,
    next_page_token: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Sku {
    #[serde(default)]
    description: String,
    category: SkuCategory,
    #[serde(default)]
    service_regions: Vec<String>,
    #[serde(default)]
    pricing_info: Vec<SkuPricingInfo>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SkuCategory {
    #[serde(default)]
    resource_group: String,
    #[serde(default)]
    usage_type: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SkuPricingInfo {
    pricing_expression: PricingExpression,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PricingExpression {
    #[serde(default)]
    tiered_rates: Vec<TieredRate>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TieredRate {
    #[serde(default)]
    start_usage_amount: f64,
    unit_price: Money,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Money {
    #[serde(default)]
    currency_code: String,
    #[serde(default)]
    units: String,
    #[serde(default)]
    nanos: i32,
}

impl Money {
    fn get_amount(&self) -> f64 {
        let units = self.units.parse::<f64>().unwrap_or(0.0);
        units + f64::from(self.nanos) / NANOS_PER_UNIT
    }
}
