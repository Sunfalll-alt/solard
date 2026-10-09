use anyhow::{anyhow, Result};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct GeoLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub city: String,
    pub country: String,
    pub timezone: String,
}

pub struct GeoLocator;

impl GeoLocator {
    /// Detects user's geographic coordinates using ip-api.com via curl
    pub fn auto_detect() -> Result<GeoLocation> {
        log::info!("Attempting geolocation auto-detection via IP...");

        let output = Command::new("curl")
            .args(["-s", "--connect-timeout", "5", "http://ip-api.com/json"])
            .output()
            .map_err(|e| anyhow!("Failed to run curl: {}. Please ensure curl is installed.", e))?;

        if !output.status.success() {
            return Err(anyhow!("Network request failed with status {:?}", output.status));
        }

        let body = String::from_utf8_lossy(&output.stdout);
        Self::parse_ip_api_json(&body)
    }

    fn parse_ip_api_json(json: &str) -> Result<GeoLocation> {
        let lat = Self::extract_float(json, "\"lat\":")
            .ok_or_else(|| anyhow!("Could not find 'lat' in response"))?;
        let lon = Self::extract_float(json, "\"lon\":")
            .ok_or_else(|| anyhow!("Could not find 'lon' in response"))?;
        let city = Self::extract_string(json, "\"city\":").unwrap_or_else(|| "Unknown".to_string());
        let country = Self::extract_string(json, "\"country\":").unwrap_or_else(|| "Unknown".to_string());
        let timezone = Self::extract_string(json, "\"timezone\":").unwrap_or_else(|| "UTC".to_string());

        Ok(GeoLocation {
            latitude: lat,
            longitude: lon,
            city,
            country,
            timezone,
        })
    }

    fn extract_float(json: &str, key: &str) -> Option<f64> {
        let start = json.find(key)? + key.len();
        let rest = &json[start..].trim_start();
        let end = rest.find(|c: char| c == ',' || c == '}' || c.is_whitespace())?;
        rest[..end].trim().parse::<f64>().ok()
    }

    fn extract_string(json: &str, key: &str) -> Option<String> {
        let start = json.find(key)? + key.len();
        let rest = &json[start..].trim_start();
        if !rest.starts_with('"') {
            return None;
        }
        let quote_start = 1;
        let quote_end = rest[quote_start..].find('"')? + quote_start;
        Some(rest[quote_start..quote_end].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json() {
        let sample = r#"{"status":"success","country":"Russia","city":"Saint Petersburg","lat":59.9343,"lon":30.3351,"timezone":"Europe/Moscow"}"#;
        let res = GeoLocator::parse_ip_api_json(sample).unwrap();
        assert_eq!(res.city, "Saint Petersburg");
        assert_eq!(res.country, "Russia");
        assert!((res.latitude - 59.9343).abs() < 0.001);
        assert!((res.longitude - 30.3351).abs() < 0.001);
    }
}
