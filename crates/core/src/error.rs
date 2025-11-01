use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse IP address: {0}")]
    IpAddressParse(#[from] std::net::AddrParseError),
}

pub type Result<T> = std::result::Result<T, Error>;
