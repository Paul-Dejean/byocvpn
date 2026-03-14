use byocvpn_core::{credentials::CredentialStore, error::Result};
use serde::{Deserialize, Serialize};

use crate::provider::OracleProviderConfig;

fn normalize_pem(raw: &str) -> String {
    let normalised = raw.replace("\r\n", "\n").replace('\r', "\n");

    let begin_pos = match normalised.find("-----BEGIN ") {
        Some(pos) => pos,
        None => return normalised.trim().to_string(),
    };

    let end_prefix = "-----END ";
    let end_start = match normalised.rfind(end_prefix) {
        Some(pos) => pos,
        None => return normalised[begin_pos..].trim().to_string(),
    };

    let rest = &normalised[end_start..];
    let end_line_end = rest
        .find('\n')
        .map(|newline_offset| end_start + newline_offset + 1)
        .unwrap_or(normalised.len());

    normalised[begin_pos..end_line_end].trim_end().to_string()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OracleCredentials {
    pub tenancy_ocid: String,
    pub user_ocid: String,
    pub fingerprint: String,
    pub private_key_pem: String,
    pub region: String,
}

impl OracleCredentials {
    pub fn from_store(store: &CredentialStore) -> Result<Self> {
        let pem_escaped = store.require("ORACLE", "private_key_pem")?;
        Ok(Self {
            tenancy_ocid: store.require("ORACLE", "tenancy_ocid")?,
            user_ocid: store.require("ORACLE", "user_ocid")?,
            fingerprint: store.require("ORACLE", "fingerprint")?,
            private_key_pem: normalize_pem(&pem_escaped.replace("\\n", "\n")),
            region: store.require("ORACLE", "region")?,
        })
    }

    pub fn write_to_store(&self, store: &mut CredentialStore) {
        let cleaned_pem = normalize_pem(&self.private_key_pem);
        let pem_single_line = cleaned_pem.replace('\n', "\\n");
        store.set("ORACLE", "tenancy_ocid", &self.tenancy_ocid);
        store.set("ORACLE", "user_ocid", &self.user_ocid);
        store.set("ORACLE", "fingerprint", &self.fingerprint);
        store.set("ORACLE", "private_key_pem", &pem_single_line);
        store.set("ORACLE", "region", &self.region);
    }
}

impl From<OracleCredentials> for OracleProviderConfig {
    fn from(credentials: OracleCredentials) -> Self {
        Self {
            tenancy_ocid: credentials.tenancy_ocid,
            user_ocid: credentials.user_ocid,
            fingerprint: credentials.fingerprint,
            private_key_pem: credentials.private_key_pem,
            region: credentials.region,
        }
    }
}
