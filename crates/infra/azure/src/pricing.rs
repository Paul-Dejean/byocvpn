/// Azure VM pricing.
///
/// Rates are for pay-as-you-go Linux VMs in West Europe (USD).
/// Two public IPs (pip4 + pip6) are created per instance; both are billed.
/// Updated: 2026-03-03.
use byocvpn_core::cloud_provider::PricingInfo;

/// $0.005/hr per public IP (Standard SKU, Azure global).
/// Two IPs are created per instance (IPv4 + IPv6), so effective rate is 2×.
const IP_HOURLY_RATE: f64 = 0.005 * 2.0;

/// $0.087/GB outbound data transfer (first 10 GB/month free, then tiered).
const EGRESS_RATE_PER_GB: f64 = 0.087;

const INSTANCE_PRICES: &[(&str, f64)] = &[
    ("Standard_B1s", 0.0104),
    ("Standard_B1ms", 0.0207),
    ("Standard_B2s", 0.0416),
    ("Standard_B2ms", 0.0832),
    ("Standard_D2s_v3", 0.096),
    ("Standard_D2s_v4", 0.096),
    ("Standard_D2s_v5", 0.096),
    ("Standard_F2s_v2", 0.085),
];

/// Return pricing for `instance_type`, or `None` if unknown.
pub fn get_pricing(instance_type: &str) -> Option<PricingInfo> {
    INSTANCE_PRICES
        .iter()
        .find(|(name, _)| *name == instance_type)
        .map(|(_, hourly_rate)| PricingInfo {
            hourly_rate: *hourly_rate,
            ip_hourly_rate: IP_HOURLY_RATE,
            egress_rate_per_gb: EGRESS_RATE_PER_GB,
        })
}
