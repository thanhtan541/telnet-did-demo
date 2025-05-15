use serde::{Deserialize, Serialize};

// Represents a verification method in the DID Document
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VerificationMethod {
    pub id: String,
    #[serde(rename = "type")]
    pub vc_type: String,
    pub controller: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_hex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_base58: Option<String>,
}

// Represents a service in the DID Document
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Service {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "serviceEndpoint")]
    pub service_endpoint: String,
}

// Represents the DID Document
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DidDocument {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    #[serde(rename = "verificationMethod", skip_serializing_if = "Vec::is_empty")]
    pub verification_method: Vec<VerificationMethod>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub authentication: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<Vec<Service>>,
}

impl DidDocument {
    // Constructor for a minimal DID Document
    pub fn new(did: &str) -> Self {
        DidDocument {
            context: vec!["https://www.w3.org/ns/did/v1".to_string()],
            id: did.to_string(),
            verification_method: vec![],
            authentication: vec![],
            service: None,
        }
    }

    // Add a verification method
    pub fn add_verification_method(&mut self, vm: VerificationMethod) {
        self.verification_method.push(vm);
    }

    // Add an authentication reference
    pub fn add_authentication(&mut self, auth_id: &str) {
        self.authentication.push(auth_id.to_string());
    }

    // Add a service
    pub fn add_service(&mut self, service: Service) {
        if let Some(mut svs) = self.service.take() {
            svs.push(service);
        } else {
            self.service = Some(vec![service]);
        }
    }

    // Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

pub fn generate_document(
    did: &str,
    base58_signing_key: Option<String>,
) -> Result<DidDocument, String> {
    // Create a new DID Document
    let mut did_doc = DidDocument::new(did);

    // Add a verification method
    let ver_method_id_1 = format!("{}#key1", did);
    let verification_method = VerificationMethod {
        id: ver_method_id_1.to_string(),
        vc_type: "Ed25519VerificationKey2020".to_string(),
        controller: did.to_string(),
        public_key_hex: None,
        public_key_base58: base58_signing_key,
    };
    did_doc.add_verification_method(verification_method);

    // Add authentication
    did_doc.add_authentication(&ver_method_id_1);

    // Add a service
    let service = Service {
        id: "did:example:123456789abcdefghi#vcs".to_string(),
        type_: "VerifiableCredentialService".to_string(),
        service_endpoint: "https://example.com/vc/".to_string(),
    };
    did_doc.add_service(service);

    Ok(did_doc)
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::SigningKey;
    use rand_core::OsRng;

    use crate::encode_public_key_to_multibase;

    use super::*;

    #[test]
    fn test_generate_document() {
        let did = "did:example:123456789abcdefghi";
        let doc = generate_document(did, None);

        assert!(doc.is_ok());
    }

    #[test]
    fn test_verify_document() {
        // Generate signing, verifying keypair
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let encoded_vk = encode_public_key_to_multibase(&verifying_key)
            .expect("Failed to encoded verifying key");

        let did = "did:example:123456789abcdefghi";
        let doc = generate_document(did, Some(encoded_vk));

        assert!(doc.is_ok());
    }
}
