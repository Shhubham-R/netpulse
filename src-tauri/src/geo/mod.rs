pub mod dns_resolver;

use dashmap::DashMap;
use maxminddb::Reader;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoResult {
    pub country_code: String,
    pub country_name: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
}

pub struct GeoIpLookup {
    reader: Option<Reader<Vec<u8>>>,
    cache: Arc<DashMap<IpAddr, GeoResult>>,
}

impl GeoIpLookup {
    pub fn new(data_dir: PathBuf) -> Self {
        let mmdb_path = data_dir.join("GeoLite2-City.mmdb");
        let reader = if mmdb_path.exists() {
            match Reader::open_readfile(&mmdb_path) {
                Ok(r) => {
                    log::info!("Loaded GeoLite2-City.mmdb from {}", mmdb_path.display());
                    Some(r)
                }
                Err(e) => {
                    log::warn!("Failed to load GeoLite2 database: {e}");
                    None
                }
            }
        } else {
            log::info!(
                "GeoLite2-City.mmdb not found at {}. GeoIP lookups will return unknown.",
                mmdb_path.display()
            );
            None
        };

        Self {
            reader,
            cache: Arc::new(DashMap::new()),
        }
    }

    pub fn lookup(&self, ip: IpAddr) -> GeoResult {
        // Check cache first
        if let Some(cached) = self.cache.get(&ip) {
            return cached.clone();
        }

        let result = self.do_lookup(ip);
        self.cache.insert(ip, result.clone());
        result
    }

    fn do_lookup(&self, ip: IpAddr) -> GeoResult {
        let reader = match &self.reader {
            Some(r) => r,
            None => return GeoResult::unknown(),
        };

        let city: maxminddb::geoip2::City = match reader.lookup(ip) {
            Ok(c) => c,
            Err(_) => return GeoResult::unknown(),
        };

        let country_code = city
            .country
            .as_ref()
            .and_then(|c| c.iso_code)
            .unwrap_or("??")
            .to_string();

        let country_name = city
            .country
            .as_ref()
            .and_then(|c| c.names.as_ref())
            .and_then(|n| n.get("en"))
            .copied()
            .unwrap_or("Unknown")
            .to_string();

        let city_name = city
            .city
            .as_ref()
            .and_then(|c| c.names.as_ref())
            .and_then(|n| n.get("en"))
            .copied()
            .unwrap_or("")
            .to_string();

        let (latitude, longitude) = city
            .location
            .as_ref()
            .map(|l| (l.latitude.unwrap_or(0.0), l.longitude.unwrap_or(0.0)))
            .unwrap_or((0.0, 0.0));

        GeoResult {
            country_code,
            country_name,
            city: city_name,
            latitude,
            longitude,
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.reader.is_some()
    }
}

impl GeoResult {
    pub fn unknown() -> Self {
        Self {
            country_code: "??".to_string(),
            country_name: "Unknown".to_string(),
            city: String::new(),
            latitude: 0.0,
            longitude: 0.0,
        }
    }
}
