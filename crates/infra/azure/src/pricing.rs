use byocvpn_core::cloud_provider::PricingInfo;

const IP_HOURLY_RATE: f64 = 0.005 * 2.0;

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
