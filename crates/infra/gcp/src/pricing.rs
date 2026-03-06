/// GCP Compute Engine pricing.
///
/// Rates are for on-demand Linux VMs (us-central1, USD).
/// Ephemeral public IPv4 addresses on standard tier are billed since 2024.
/// Updated: 2026-03-03.
use byocvpn_core::cloud_provider::PricingInfo;

/// $0.004/hr for in-use ephemeral IPv4 on standard tier (GCP global, since 2024).
const IPV4_HOURLY_RATE: f64 = 0.004;

/// $0.08/GB internet egress (first 1 GB/month free, then tiered; us-central1).
const EGRESS_RATE_PER_GB: f64 = 0.08;

const INSTANCE_PRICES: &[(&str, f64)] = &[
    ("e2-micro", 0.0084),
    ("e2-small", 0.0168),
    ("e2-medium", 0.0335),
    ("f1-micro", 0.0076),
    ("g1-small", 0.0257),
    ("n1-standard-1", 0.0475),
    ("n2-standard-2", 0.0971),
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
