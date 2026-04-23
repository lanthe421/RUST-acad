use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RequestType {
    Success,
    Failure,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PublicTariff {
    pub id: u64,
    pub price: u64,
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PrivateTariff {
    pub client_price: u64,
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Stream {
    pub user_id: String,
    pub is_private: bool,
    pub settings: u64,
    pub shard_url: String,
    pub public_tariff: PublicTariff,
    pub private_tariff: PrivateTariff,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Gift {
    pub id: u64,
    pub price: u64,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Debug {
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Request {
    #[serde(rename = "type")]
    pub kind: RequestType,
    pub stream: Stream,
    pub gifts: Vec<Gift>,
    pub debug: Debug,
}

fn main() {
    let json = include_str!("../request.json");
    println!("{}", json);

    let request: Request = serde_json::from_str(json).expect("Failed to deserialize JSON");

    let yaml = serde_yaml::to_string(&request).expect("Failed to serialize to YAML");
    println!("=== YAML ===\n{yaml}");

    let toml_str = toml::to_string_pretty(&request).expect("Failed to serialize to TOML");
    println!("=== TOML ===\n{toml_str}");
}

#[cfg(test)]
mod tests {
    use super::*;

    const JSON: &str = include_str!("../request.json");

    #[test]
    fn test_deserialize_type_is_success() {
        let req: Request = serde_json::from_str(JSON).unwrap();
        assert!(matches!(req.kind, RequestType::Success));
    }

    #[test]
    fn test_deserialize_stream_fields() {
        let req: Request = serde_json::from_str(JSON).unwrap();
        assert_eq!(req.stream.user_id, "8d234120-0bda-49b2-b7e0-fbd3912f6cbf");
        assert!(!req.stream.is_private);
        assert_eq!(req.stream.settings, 45345);
        assert_eq!(req.stream.shard_url, "https://n3.example.com/sapi");
    }

    #[test]
    fn test_deserialize_public_tariff() {
        let req: Request = serde_json::from_str(JSON).unwrap();
        let t = &req.stream.public_tariff;
        assert_eq!(t.id, 1);
        assert_eq!(t.price, 100);
        assert_eq!(t.duration, Duration::from_secs(3600));
        assert_eq!(t.description, "test public tariff");
    }

    #[test]
    fn test_deserialize_private_tariff() {
        let req: Request = serde_json::from_str(JSON).unwrap();
        let t = &req.stream.private_tariff;
        assert_eq!(t.client_price, 250);
        assert_eq!(t.duration, Duration::from_secs(60));
        assert_eq!(t.description, "test private tariff");
    }

    #[test]
    fn test_deserialize_gifts() {
        let req: Request = serde_json::from_str(JSON).unwrap();
        assert_eq!(req.gifts.len(), 2);
        assert_eq!(req.gifts[0].id, 1);
        assert_eq!(req.gifts[0].price, 2);
        assert_eq!(req.gifts[1].description, "Gift 2");
    }

    #[test]
    fn test_deserialize_debug() {
        let req: Request = serde_json::from_str(JSON).unwrap();
        assert_eq!(req.debug.duration, Duration::from_millis(234));
    }

    #[test]
    fn test_roundtrip_json() {
        let req: Request = serde_json::from_str(JSON).unwrap();
        let serialized = serde_json::to_string(&req).unwrap();
        let req2: Request = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req.stream.user_id, req2.stream.user_id);
        assert_eq!(req.gifts.len(), req2.gifts.len());
    }

    #[test]
    fn test_serialize_yaml() {
        let req: Request = serde_json::from_str(JSON).unwrap();
        let yaml = serde_yaml::to_string(&req).unwrap();
        assert!(yaml.contains("user_id"));
        assert!(yaml.contains("public_tariff"));
    }

    #[test]
    fn test_serialize_toml() {
        let req: Request = serde_json::from_str(JSON).unwrap();
        let toml_str = toml::to_string_pretty(&req).unwrap();
        assert!(toml_str.contains("user_id"));
        assert!(toml_str.contains("public_tariff"));
    }
}
