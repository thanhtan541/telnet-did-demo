use serde::{Deserialize, Serialize};
use std::fmt;

pub struct Keypair {}

/// Represents a Decentralized Identifier (DID) as per W3C DID v1.0 specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DID {
    /// The complete DID string, e.g., "did:example:123456789abcdefghi".
    id: String,
    /// The DID method, e.g., "example".
    method: String,
    /// The method-specific identifier, e.g., "123456789abcdefghi".
    method_specific_id: String,
}

impl DID {
    /// Creates a new DID instance by parsing a DID string.
    ///
    /// # Arguments
    /// * `did` - A string representing the DID (e.g., "did:example:123456789abcdefghi").
    ///
    /// # Returns
    /// * `Result<DID, String>` - Ok with parsed DID or Err with error message.
    pub fn new(did: &str) -> Result<Self, String> {
        // Split the DID into components: did:method:method_specific_id
        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() < 3 || parts[0] != "did" {
            return Err(format!("Invalid DID format: {}", did));
        }

        let method = parts[1].to_string();
        let method_specific_id = parts[2..].join(":");

        // Validate method name (alphanumeric, lowercase, 1-50 chars)
        if !method
            .chars()
            .all(|c| c.is_alphanumeric() && c.is_lowercase())
            || method.len() > 50
        {
            return Err(format!("Invalid method name: {}", method));
        }

        // Validate method-specific ID (basic check for non-empty)
        if method_specific_id.is_empty() {
            return Err("Method-specific ID cannot be empty".to_string());
        }

        Ok(DID {
            id: did.to_string(),
            method,
            method_specific_id,
        })
    }

    /// Returns the DID string.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the DID method.
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Returns the method-specific identifier.
    pub fn method_specific_id(&self) -> &str {
        &self.method_specific_id
    }
}

impl fmt::Display for DID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_did() {
        let did_str = "did:example:123456789abcdefghi";
        let did = DID::new(did_str).unwrap();
        assert_eq!(did.id(), did_str);
        assert_eq!(did.method(), "example");
        assert_eq!(did.method_specific_id(), "123456789abcdefghi");
        assert_eq!(did.to_string(), did_str);
    }

    #[test]
    fn test_invalid_did_format() {
        let did_str = "example:123456789abcdefghi";
        let result = DID::new(did_str);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!("Invalid DID format: {}", did_str)
        );
    }

    #[test]
    fn test_invalid_method_name() {
        let did_str = "did:EXAMPLE:123456789abcdefghi";
        let result = DID::new(did_str);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid method name: EXAMPLE");
    }

    #[test]
    fn test_empty_method_specific_id() {
        let did_str = "did:example:";
        let result = DID::new(did_str);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Method-specific ID cannot be empty");
    }
}
