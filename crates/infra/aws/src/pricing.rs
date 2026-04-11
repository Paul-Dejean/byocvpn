use byocvpn_core::cloud_provider::PricingInfo;

const IPV4_HOURLY_RATE: f64 = 0.005;

const EGRESS_RATE_PER_GB: f64 = 0.09;

const STORAGE_GB: f64 = 8.0;

const STORAGE_RATE_PER_GB_MONTH: f64 = 0.10;

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
