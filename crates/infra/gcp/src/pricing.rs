use byocvpn_core::cloud_provider::PricingInfo;

const IPV4_HOURLY_RATE: f64 = 0.004;

const EGRESS_RATE_PER_GB: f64 = 0.08;

const STORAGE_GB: f64 = 10.0;

const STORAGE_RATE_PER_GB_MONTH: f64 = 0.04;

const INSTANCE_PRICES: &[(&str, f64)] = &[
    ("e2-micro", 0.0084),
    ("e2-small", 0.0168),
    ("e2-medium", 0.0335),
    ("f1-micro", 0.0076),
    ("g1-small", 0.0257),
    ("n1-standard-1", 0.0475),
    ("n2-standard-2", 0.0971),
];

pub fn get_pricing(instance_type: &str) -> Option<PricingInfo> {
    INSTANCE_PRICES
        .iter()
        .find(|(name, _)| *name == instance_type)
        .map(|(_, hourly_rate)| PricingInfo {
            hourly_rate: *hourly_rate,
            ip_hourly_rate: IPV4_HOURLY_RATE,
            egress_rate_per_gb: EGRESS_RATE_PER_GB,
            storage_gb: STORAGE_GB,
            storage_rate_per_gb_month: STORAGE_RATE_PER_GB_MONTH,
        })
}
