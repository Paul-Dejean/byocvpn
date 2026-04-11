use byocvpn_core::cloud_provider::PricingInfo;

const INSTANCE_PRICES: &[(&str, f64)] = &[
    ("VM.Standard.A1.Flex", 0.0),
    ("VM.Standard.E4.Flex", 0.025),
    ("VM.Standard.E3.Flex", 0.025),
    ("VM.Standard3.Flex", 0.038),
];

pub fn get_pricing(instance_type: &str) -> Option<PricingInfo> {
    INSTANCE_PRICES
        .iter()
        .find(|(name, _)| *name == instance_type)
        .map(|(_, hourly_rate)| PricingInfo {
            hourly_rate: *hourly_rate,
            ip_hourly_rate: 0.0,
            egress_rate_per_gb: 0.0,
            storage_gb: 50.0,
            storage_rate_per_gb_month: 0.0255,
        })
}
