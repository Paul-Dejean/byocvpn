use byocvpn_core::cloud_provider::PricingInfo;
use reqwest::Client as HttpClient;
use serde::Deserialize;
use tokio::time::Duration;

use crate::{
    error::{AzureError, Result},
    instance::OS_DISK_SIZE_GB,
};

const RETAIL_PRICES_URL: &str = "https://prices.azure.com/api/retail/prices";
const RETAIL_PRICES_API_VERSION: &str = "2023-01-01-preview";
const REQUEST_TIMEOUT_SECS: u64 = 15;

const PUBLIC_IP_COUNT_PER_VM: f64 = 2.0;
const PUBLIC_IP_METER_NAME: &str = "Standard IPv4 Static Public IP";
const OS_DISK_PRODUCT_NAME: &str = "Premium SSD Managed Disks";
const OS_DISK_SKU_NAME: &str = "P4 LRS";
const OS_DISK_METER_NAME: &str = "P4 LRS Disk";
const EGRESS_PRODUCT_NAME: &str = "Rtn Preference: MGN";
const EGRESS_METER_NAME: &str = "Standard Data Transfer Out";
const HOURLY_UNIT_OF_MEASURE: &str = "1 Hour";

pub async fn fetch_pricing(region: &str, vm_size: &str) -> Result<PricingInfo> {
    let http = HttpClient::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()?;

    let vm_filter = format!(
        "serviceName eq 'Virtual Machines' and armRegionName eq '{region}' and armSkuName eq '{vm_size}' and priceType eq 'Consumption'"
    );
    let public_ip_filter = format!(
        "serviceName eq 'Virtual Network' and armRegionName eq '{region}' and meterName eq '{PUBLIC_IP_METER_NAME}' and priceType eq 'Consumption'"
    );
    let os_disk_filter = format!(
        "serviceName eq 'Storage' and armRegionName eq '{region}' and productName eq '{OS_DISK_PRODUCT_NAME}' and skuName eq '{OS_DISK_SKU_NAME}' and meterName eq '{OS_DISK_METER_NAME}' and priceType eq 'Consumption'"
    );
    let egress_filter = format!(
        "serviceName eq 'Bandwidth' and armRegionName eq '{region}' and productName eq '{EGRESS_PRODUCT_NAME}' and meterName eq '{EGRESS_METER_NAME}' and priceType eq 'Consumption'"
    );
    let (vm_items, public_ip_items, os_disk_items, egress_items) = tokio::try_join!(
        query_retail_prices(&http, &vm_filter),
        query_retail_prices(&http, &public_ip_filter),
        query_retail_prices(&http, &os_disk_filter),
        query_retail_prices(&http, &egress_filter),
    )?;

    let build_unavailable_error = |component: &'static str| AzureError::PriceUnavailable {
        region: region.to_string(),
        vm_size: vm_size.to_string(),
        component,
    };
    let hourly_rate = select_linux_hourly_rate(&vm_items)
        .ok_or_else(|| build_unavailable_error("Linux consumption rate"))?;
    let public_ip_hourly_rate = public_ip_items
        .first()
        .map(|item| item.retail_price)
        .ok_or_else(|| build_unavailable_error("public IP"))?;
    let os_disk_monthly_rate = os_disk_items
        .first()
        .map(|item| item.retail_price)
        .ok_or_else(|| build_unavailable_error("OS disk"))?;
    let egress_rate_per_gb = select_first_paid_tier(&egress_items)
        .ok_or_else(|| build_unavailable_error("internet egress"))?;

    let storage_gb = f64::from(OS_DISK_SIZE_GB);
    Ok(PricingInfo {
        hourly_rate,
        ip_hourly_rate: public_ip_hourly_rate * PUBLIC_IP_COUNT_PER_VM,
        egress_rate_per_gb,
        storage_gb,
        storage_rate_per_gb_month: os_disk_monthly_rate / storage_gb,
    })
}

fn select_linux_hourly_rate(items: &[RetailPriceItem]) -> Option<f64> {
    items
        .iter()
        .filter(|item| {
            !item.product_name.contains("Windows")
                && !item.sku_name.contains("Spot")
                && !item.sku_name.contains("Low Priority")
                && item.unit_of_measure == HOURLY_UNIT_OF_MEASURE
        })
        .map(|item| item.retail_price)
        .reduce(f64::min)
}

fn select_first_paid_tier(items: &[RetailPriceItem]) -> Option<f64> {
    items
        .iter()
        .filter(|item| item.retail_price > 0.0)
        .min_by(|left, right| left.tier_minimum_units.total_cmp(&right.tier_minimum_units))
        .map(|item| item.retail_price)
}

async fn query_retail_prices(http: &HttpClient, filter: &str) -> Result<Vec<RetailPriceItem>> {
    let response = http
        .get(RETAIL_PRICES_URL)
        .query(&[
            ("api-version", RETAIL_PRICES_API_VERSION),
            ("$filter", filter),
        ])
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        return Err(AzureError::RetailPricesRejected {
            status: status.as_u16(),
        });
    }

    let page: RetailPricesPage = response.json().await?;
    Ok(page.items)
}

#[derive(Deserialize)]
struct RetailPricesPage {
    #[serde(rename = "Items", default)]
    items: Vec<RetailPriceItem>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RetailPriceItem {
    retail_price: f64,
    product_name: String,
    sku_name: String,
    unit_of_measure: String,
    #[serde(default)]
    tier_minimum_units: f64,
}
