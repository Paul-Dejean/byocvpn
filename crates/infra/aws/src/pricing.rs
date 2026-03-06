/// AWS instance pricing.
///
/// Rates are for on-demand Linux instances in us-east-1 (USD).
/// IP pricing applies globally since February 2024.
/// Updated: 2026-03-03.
use byocvpn_core::cloud_provider::PricingInfo;

/// $0.005/hr per public IPv4 address (AWS global, since Feb 2024).
const IPV4_HOURLY_RATE: f64 = 0.005;

/// $0.09/GB outbound data transfer (first 10 TB/month, us-east-1).
const EGRESS_RATE_PER_GB: f64 = 0.09;

const INSTANCE_PRICES: &[(&str, f64)] = &[
    ("t2.micro", 0.0116),
    ("t2.small", 0.023),
    ("t2.medium", 0.0464),
    ("t3.micro", 0.0104),
    ("t3.small", 0.0208),
    ("t3.medium", 0.0416),
    ("t3a.micro", 0.0094),
    ("t3a.small", 0.0188),
];

/// Return pricing for `instance_type`, or `None` if unknown.
pub fn get_pricing(instance_type: &str) -> Option<PricingInfo> {
    INSTANCE_PRICES
        .iter()
        .find(|(name, _)| *name == instance_type)
        .map(|(_, hourly_rate)| PricingInfo {
            hourly_rate: *hourly_rate,
            ip_hourly_rate: IPV4_HOURLY_RATE,
            egress_rate_per_gb: EGRESS_RATE_PER_GB,
        })
}
