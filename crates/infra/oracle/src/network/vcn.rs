use log::info;

use crate::{
    client::OciClient,
    error::{OracleError, Result},
    models::{CreateVcnRequest, VcnResponse, build_managed_tags},
};

const VCN_DISPLAY_NAME: &str = "byocvpn-vcn";
const VCN_CIDR: &str = "10.0.0.0/16";
const AVAILABLE_LIFECYCLE_STATE: &str = "AVAILABLE";

pub struct Vcn {
    pub id: String,
    pub default_route_table_id: String,
    pub ipv6_prefix: String,
}

impl Vcn {
    pub const fn has_ipv6(&self) -> bool {
        !self.ipv6_prefix.is_empty()
    }

    fn from_response(response: VcnResponse) -> Result<Self> {
        let ipv6_prefix = response
            .ipv6_cidr_blocks
            .and_then(|blocks| blocks.into_iter().next())
            .ok_or(OracleError::MissingResourceField {
                resource: "VCN",
                field: "ipv6CidrBlocks",
            })?;
        Ok(Self {
            id: response.id,
            default_route_table_id: response.default_route_table_id,
            ipv6_prefix,
        })
    }
}

pub async fn ensure_vcn(client: &OciClient) -> Result<Vcn> {
    if let Some(vcn) = get_vcn(client).await? {
        return Ok(vcn);
    }
    create_vcn(client).await
}

async fn get_vcn(client: &OciClient) -> Result<Option<Vcn>> {
    let url = client.build_core_url(&format!(
        "/vcns?compartmentId={}&displayName={VCN_DISPLAY_NAME}",
        client.get_compartment_ocid()
    ));
    let vcns: Vec<VcnResponse> = client.get(&url).await?;
    vcns.into_iter()
        .find(|vcn| vcn.lifecycle_state == AVAILABLE_LIFECYCLE_STATE)
        .map(Vcn::from_response)
        .transpose()
}

async fn create_vcn(client: &OciClient) -> Result<Vcn> {
    let body = CreateVcnRequest {
        compartment_id: client.get_compartment_ocid().to_string(),
        display_name: VCN_DISPLAY_NAME.to_string(),
        cidr_block: VCN_CIDR.to_string(),
        is_ipv6_enabled: true,
        freeform_tags: build_managed_tags(),
    };
    let response: VcnResponse = client.post(&client.build_core_url("/vcns"), &body).await?;
    info!("[OCI] Created VCN {}", response.id);
    Vcn::from_response(response)
}
