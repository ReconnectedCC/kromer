use serde::{Deserialize, Serialize};
use std::fs;
use toml;

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct DetailedMotdResponse {
    pub ok: bool,
    #[serde(flatten)]
    pub motd: DetailedMotd, 
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Motd {
    pub motd: String,
    pub motd_set: String,
    pub debug_mode: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct DetailedMotd {
    pub server_time: String,
    pub motd: String,
    pub set: Option<String>, // Support for backwards compatibility
    pub motd_set: Option<String>,

    pub public_url: String,
    pub public_ws_url: String,
    pub mining_enabled: bool,
    pub transactions_enabled: bool,
    pub debug_mode: bool,

    pub work: i64,
    pub last_block: Option<super::blocks::BlockJson>,
    pub package: PackageInfo,
    pub constants: Constants,
    pub currency: CurrencyInfo,

    pub notice: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub author: String,
    #[serde(rename = "licence")] // Fuck off, Krist
    pub license: String,
    pub repository: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Constants {
    pub wallet_version: i64,
    pub nonce_max_size: i64,
    pub name_cost: i64,
    pub min_work: i64,
    pub max_work: i64,
    pub work_factor: f64,
    pub seconds_per_block: i64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CurrencyInfo {
    pub address_prefix: String,
    pub name_suffix: String,
    pub currency_name: String,
    pub currency_symbol: String,
}

pub const MINING_CONSTANTS: Constants = Constants {
    wallet_version: 16,
    nonce_max_size: 24,
    name_cost: 500,
    min_work: 1,
    max_work: 100000,
    work_factor: 0.025,
    seconds_per_block: 300,
};

pub fn get_currency_info() -> CurrencyInfo {
    CurrencyInfo {
        address_prefix: "k".to_string(),
        name_suffix: "kro".to_string(),
        currency_name: "Kromer".to_string(),
        currency_symbol: "KRO".to_string(),
    }
}

pub fn get_package_info() -> Result<PackageInfo, std::io::Error> {
    let toml_string = fs::read_to_string("Cargo.toml")?;
    let parsed_toml: toml::Value = toml::from_str(&toml_string).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse Cargo TOML: {}", err),
        )
    })?;

    let get_str_value = |key: &str| -> Result<String, std::io::Error> {
        parsed_toml
            .get("package")
            .and_then(|pkg| pkg.get(key))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Missing Cargo.toml key: {}", key),
                )
            })
    };

    // Access info from the parsed TOML data
    let name = get_str_value("name")?;
    let version = get_str_value("version")?;
    let author = get_str_value("author")?;
    let license = get_str_value("license")?;
    let repository = get_str_value("repository")?;

    Ok(PackageInfo {
        name,
        version,
        author,
        license,
        repository,
    })
}
