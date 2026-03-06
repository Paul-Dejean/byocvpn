/// Oracle Cloud Infrastructure (OCI) pricing.
///
/// VM.Standard.A1.Flex is part of the OCI Always Free tier:
/// up to 4 OCPUs and 24 GB RAM per tenancy at no cost.
/// Outbound data transfer is also free up to 10 TB/month.
/// Updated: 2026-03-03.
use byocvpn_core::cloud_provider::PricingInfo;

const INSTANCE_PRICES: &[(&str, f64)] = &[
    // Always Free: VM.Standard.A1.Flex (Ampere Altra, ARM)
    ("VM.Standard.A1.Flex", 0.0),
    // Paid shapes (for reference if users switch away from Always Free)
    ("VM.Standard.E4.Flex", 0.025),
    ("VM.Standard.E3.Flex", 0.025),
    ("VM.Standard3.Flex", 0.038),
];

/// Return pricing for `instance_type`, or `None` if unknown.
pub fn get_pricing(instance_type: &str) -> Option<PricingInfo> {
    INSTANCE_PRICES
        .iter()
        .find(|(name, _)| *name == instance_type)
        .map(|(_, hourly_rate)| PricingInfo {
            hourly_rate: *hourly_rate,
            // OCI does not charge separately for public IPs on Always Free shapes.
            ip_hourly_rate: 0.0,
            // Outbound transfer is free up to 10 TB/month on OCI.
            egress_rate_per_gb: 0.0,
        })
}
