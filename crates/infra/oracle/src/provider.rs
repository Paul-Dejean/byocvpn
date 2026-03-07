use async_trait::async_trait;
use byocvpn_core::{
    cloud_provider::{
        CloudProvider, CloudProviderName, InstanceInfo, SpawnInstanceParams, SpawnStep,
        TerminateInstanceParams,
    },
    commands::setup::Region,
    error::{NetworkProvisioningError, Result},
};
use serde_json::Value;

use crate::{auth::OciCredentials, client::OciClient, instance, network};

/// Configuration required to create an `OracleProvider`.
pub struct OracleProviderConfig {
    /// OCI tenancy OCID.
    pub tenancy_ocid: String,
    /// OCI user OCID.
    pub user_ocid: String,
    /// RSA key fingerprint as shown in the OCI console.
    pub fingerprint: String,
    /// PEM-encoded RSA private key (PKCS#8).
    pub private_key_pem: String,
    /// Home region identifier (e.g. `us-ashburn-1`).
    pub region: String,
}

/// Oracle Cloud Infrastructure implementation of `CloudProvider`.
pub struct OracleProvider {
    config: OracleProviderConfig,
}

impl OracleProvider {
    pub fn new(config: OracleProviderConfig) -> Self {
        Self { config }
    }

    fn make_client(&self, region: Option<&str>) -> OciClient {
        let credentials = OciCredentials {
            tenancy_ocid: self.config.tenancy_ocid.clone(),
            user_ocid: self.config.user_ocid.clone(),
            fingerprint: self.config.fingerprint.clone(),
            private_key_pem: self.config.private_key_pem.clone(),
            region: region.unwrap_or(&self.config.region).to_string(),
        };
        OciClient::new(credentials)
    }

    /// The tenancy OCID doubles as the root compartment OCID in OCI.
    fn compartment_ocid(&self) -> &str {
        &self.config.tenancy_ocid
    }
}

#[async_trait]
impl CloudProvider for OracleProvider {
    fn get_provider_name(&self) -> CloudProviderName {
        CloudProviderName::Oracle
    }

    fn spawn_steps(&self, _region: &str) -> Vec<SpawnStep> {
        vec![
            SpawnStep { id: "setup_vcn".into(), label: "Creating home VCN".into() },
            SpawnStep { id: "setup_igw".into(), label: "Creating internet gateway".into() },
            SpawnStep { id: "region_subscribe".into(), label: "Subscribing to region \u{2014} this may take several minutes".into() },
            SpawnStep { id: "region_vcn".into(), label: "Creating regional VCN".into() },
            SpawnStep { id: "region_igw".into(), label: "Creating regional internet gateway".into() },
            SpawnStep { id: "region_security_list".into(), label: "Creating security list".into() },
            SpawnStep { id: "region_subnet".into(), label: "Creating subnet".into() },
            SpawnStep { id: "launch".into(), label: "Launching Ampere A1 instance".into() },
            SpawnStep { id: "wireguard_ready".into(), label: "Waiting for WireGuard to start".into() },
        ]
    }

    async fn run_spawn_step(&self, step_id: &str, region: &str) -> Result<()> {
        match step_id {
            "setup_vcn" => {
                let client = self.make_client(None);
                let compartment = self.compartment_ocid();
                if network::get_vcn_by_name(&client, compartment).await?.is_none() {
                    network::create_vcn(&client, compartment).await?;
                }
                Ok(())
            }
            "setup_igw" => {
                let client = self.make_client(None);
                let compartment = self.compartment_ocid();
                let (vcn_id, route_table_id, ipv6_prefix) =
                    network::get_vcn_by_name(&client, compartment)
                        .await?
                        .ok_or(NetworkProvisioningError::VpcNotFound {
                            vpc_name: "byocvpn-vcn".to_string(),
                        })?;
                let igw_id =
                    network::get_or_create_internet_gateway(&client, compartment, &vcn_id).await?;
                network::add_default_route_to_table(
                    &client,
                    &route_table_id,
                    &igw_id,
                    &ipv6_prefix,
                )
                .await?;
                Ok(())
            }
            "region_subscribe" => {
                let home_client = self.make_client(None);
                network::ensure_region_subscribed(&home_client, self.compartment_ocid(), region)
                    .await
            }
            "region_vcn" => {
                let client = self.make_client(Some(region));
                let compartment = self.compartment_ocid();
                if network::get_vcn_by_name(&client, compartment).await?.is_none() {
                    network::create_vcn(&client, compartment).await?;
                }
                Ok(())
            }
            "region_igw" => {
                let client = self.make_client(Some(region));
                let compartment = self.compartment_ocid();
                let (vcn_id, route_table_id, ipv6_prefix) =
                    network::get_vcn_by_name(&client, compartment)
                        .await?
                        .ok_or(NetworkProvisioningError::VpcNotFound {
                            vpc_name: "byocvpn-vcn".to_string(),
                        })?;
                let igw_id =
                    network::get_or_create_internet_gateway(&client, compartment, &vcn_id).await?;
                network::add_default_route_to_table(
                    &client,
                    &route_table_id,
                    &igw_id,
                    &ipv6_prefix,
                )
                .await?;
                Ok(())
            }
            "region_security_list" => {
                let client = self.make_client(Some(region));
                let compartment = self.compartment_ocid();
                let (vcn_id, _, ipv6_prefix) =
                    network::get_vcn_by_name(&client, compartment)
                        .await?
                        .ok_or(NetworkProvisioningError::VpcNotFound {
                            vpc_name: "byocvpn-vcn".to_string(),
                        })?;
                network::get_or_create_security_list(
                    &client,
                    compartment,
                    &vcn_id,
                    &ipv6_prefix,
                )
                .await?;
                Ok(())
            }
            "region_subnet" => {
                let client = self.make_client(Some(region));
                let compartment = self.compartment_ocid();
                let (vcn_id, route_table_id, ipv6_prefix) =
                    network::get_vcn_by_name(&client, compartment)
                        .await?
                        .ok_or(NetworkProvisioningError::VpcNotFound {
                            vpc_name: "byocvpn-vcn".to_string(),
                        })?;
                let security_list_id = network::get_or_create_security_list(
                    &client,
                    compartment,
                    &vcn_id,
                    &ipv6_prefix,
                )
                .await?;
                let subnet_id =
                    match network::get_subnet_by_name(&client, compartment, &vcn_id).await? {
                        Some((existing_id, _)) => existing_id,
                        None => {
                            network::create_subnet(
                                &client,
                                compartment,
                                &vcn_id,
                                &security_list_id,
                                &route_table_id,
                                &ipv6_prefix,
                            )
                            .await?
                        }
                    };
                network::ensure_subnet_security_list(&client, &subnet_id, &security_list_id)
                    .await?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn verify_permissions(&self) -> Result<Value> {
        Ok(serde_json::json!({ "status": "not_implemented" }))
    }

    async fn setup(&self) -> Result<()> {
        self.run_spawn_step("setup_vcn", "").await?;
        self.run_spawn_step("setup_igw", "").await?;
        Ok(())
    }

    async fn enable_region(&self, region: &str) -> Result<()> {
        let compartment_ocid = self.compartment_ocid();

        // Subscribe to the region if not already subscribed. This is a no-op for
        // regions the tenancy already has access to, and blocks until READY for
        // newly subscribed ones (typically 2–5 minutes).
        let home_client = self.make_client(None);
        network::ensure_region_subscribed(&home_client, compartment_ocid, region).await?;

        let client = self.make_client(Some(region));

        // Ensure a VCN exists in this region (OCI VCNs are region-scoped).
        // If the existing VCN has no IPv6 and cannot be upgraded in-place (OCI
        // returns 404 in some regions for addIpv6VcnCidr), we tear it down and
        // recreate it with isIpv6Enabled: true.  This is safe here because
        // enable_region always runs before any instances are spawned.
        let (vcn_id, route_table_id, ipv6_prefix) = {
            let (existing_vcn_id, existing_rt_id, raw_prefix) =
                match network::get_vcn_by_name(&client, compartment_ocid).await? {
                    Some(existing) => {
                        println!("Existing VCN found in {}, skipping VCN creation.", region);
                        existing
                    }
                    None => {
                        println!("No VCN in {}, creating...", region);
                        let ids = network::create_vcn(&client, compartment_ocid).await?;
                        println!("VCN created in {}.", region);
                        ids
                    }
                };

            if raw_prefix.is_empty() {
                // Try to add an Oracle-provided IPv6 prefix to the existing VCN.
                let upgraded =
                    network::ensure_vcn_ipv6(&client, &existing_vcn_id, &raw_prefix).await;
                if !upgraded.is_empty() {
                    println!("IPv6 CIDR added to VCN in {}: {}", region, upgraded);
                    (existing_vcn_id, existing_rt_id, upgraded)
                } else {
                    // In-place upgrade failed — recreate the VCN with IPv6 from the start.
                    println!(
                        "IPv6 upgrade failed for VCN in {}. Tearing down and recreating with IPv6...",
                        region
                    );
                    network::teardown_vcn_resources(
                        &client,
                        compartment_ocid,
                        &existing_vcn_id,
                        &existing_rt_id,
                    )
                    .await?;
                    let ids = network::create_vcn(&client, compartment_ocid).await?;
                    println!("VCN recreated with IPv6 in {}.", region);
                    ids
                }
            } else {
                (existing_vcn_id, existing_rt_id, raw_prefix)
            }
        };

        // Always ensure the IGW and default route exist — these are idempotent and
        // must run even when the VCN already existed (e.g. created in a previous
        // attempt that failed before the route was added).
        let igw_id =
            network::get_or_create_internet_gateway(&client, compartment_ocid, &vcn_id).await?;
        network::add_default_route_to_table(&client, &route_table_id, &igw_id, &ipv6_prefix)
            .await?;
        println!("IGW + default route ensured in {}.", region);

        let security_list_id =
            network::get_or_create_security_list(&client, compartment_ocid, &vcn_id, &ipv6_prefix)
                .await?;

        let subnet_id =
            match network::get_subnet_by_name(&client, compartment_ocid, &vcn_id).await? {
                Some((existing_id, _)) => {
                    println!(
                        "Subnet already exists in {}, ensuring security list.",
                        region
                    );
                    existing_id
                }
                None => {
                    let id = network::create_subnet(
                        &client,
                        compartment_ocid,
                        &vcn_id,
                        &security_list_id,
                        &route_table_id,
                        &ipv6_prefix,
                    )
                    .await?;
                    println!("Created subnet in {}", region);
                    id
                }
            };

        // Always attach our security list — fixes subnets from earlier partial runs
        // that were created before the security list rules were configured.
        network::ensure_subnet_security_list(&client, &subnet_id, &security_list_id).await?;
        println!("Security list attached to subnet in {}.", region);

        Ok(())
    }

    async fn spawn_instance(&self, params: &SpawnInstanceParams) -> Result<InstanceInfo> {
        let client = self.make_client(Some(params.region));
        let compartment_ocid = self.compartment_ocid();

        let (vcn_id, _, _) = network::get_vcn_by_name(&client, compartment_ocid)
            .await?
            .ok_or_else(|| NetworkProvisioningError::VpcNotFound {
                vpc_name: "byocvpn-vcn".to_string(),
            })?;

        let (subnet_ocid, subnet_has_ipv6) =
            network::get_subnet_by_name(&client, compartment_ocid, &vcn_id)
                .await?
                .ok_or_else(|| NetworkProvisioningError::SubnetMissingIdentifier {})?;

        let image_ocid = network::get_ubuntu_image(&client, compartment_ocid).await?;

        instance::spawn_instance(
            &client,
            compartment_ocid,
            &subnet_ocid,
            subnet_has_ipv6,
            &image_ocid,
            params.region,
            params,
        )
        .await
    }

    async fn terminate_instance(&self, params: &TerminateInstanceParams) -> Result<()> {
        let client = self.make_client(Some(params.region));
        instance::terminate_instance(&client, params.instance_id).await
    }

    async fn list_instances(&self, region: Option<&str>) -> Result<Vec<InstanceInfo>> {
        if let Some(region_name) = region {
            let client = self.make_client(Some(region_name));
            return instance::list_instances(&client, self.compartment_ocid(), region_name).await;
        }

        // Use subscribed regions only — querying unsubscribed regions returns 401.
        let client = self.make_client(None);
        let region_list = network::list_regions(&client, self.compartment_ocid()).await?;
        let regions: Vec<Region> = region_list
            .into_iter()
            .map(|(name, country)| Region { name, country })
            .collect();
        let results = futures::future::join_all(regions.iter().map(|r| async move {
            let client = self.make_client(Some(&r.name));
            match instance::list_instances(&client, self.compartment_ocid(), &r.name).await {
                Ok(instances) => instances,
                Err(e) => {
                    eprintln!("Skipping OCI region {}: {}", r.name, e);
                    vec![]
                }
            }
        }))
        .await;
        Ok(results.into_iter().flatten().collect())
    }

    async fn get_regions(&self) -> Result<Vec<Region>> {
        let client = self.make_client(None);
        let region_list = network::list_all_regions(&client).await?;
        Ok(region_list
            .into_iter()
            .map(|(name, country)| Region { name, country })
            .collect())
    }
}
