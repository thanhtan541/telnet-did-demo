use base58::{FromBase58, ToBase58};
use ed25519_dalek::{
    ed25519::SignatureBytes, Signature, Signer, SigningKey, Verifier, VerifyingKey,
};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;

use crate::{encode_public_key_to_multibase, generate_document, DidDocument};

// Create request structure
#[derive(Serialize, Deserialize, Clone)]
pub struct CreateRequest {
    #[serde(rename = "type")]
    pub request_type: String,
    pub did: String,
    pub document: DidDocument,
    pub signature: String,
}

// Function to create and sign a create request
pub fn create_signed_request(
    did: &str,
    signer: &SigningKey,
) -> Result<CreateRequest, Box<dyn Error>> {
    let verifying_key = signer.verifying_key();
    let encoded_vk = encode_public_key_to_multibase(&verifying_key)?;
    let document = generate_document(did, Some(encoded_vk)).unwrap();

    let payload = json!({
        "type": "create",
        "did": did,
        "document": document,
    });

    let payload_bytes = serde_json::to_string(&payload)?.into_bytes();
    let signature = signer.sign(&payload_bytes);

    Ok(CreateRequest {
        request_type: "create".to_string(),
        did: did.to_string(),
        document,
        signature: signature.to_bytes().to_base58(),
    })
}

// Function to verify the signature in a create request
fn verify_request(request: &CreateRequest, key: &VerifyingKey) -> Result<bool, String> {
    // Reconstruct payload for verification
    let payload = json!({
        "type": request.request_type,
        "did": request.did,
        "document": request.document,
    });
    let payload_bytes = serde_json::to_string(&payload).unwrap().into_bytes();

    // Decode and verify signature
    let signature_bytes = request.signature.from_base58().unwrap();
    let signature: Signature = Signature::try_from(&signature_bytes[..64]).unwrap();

    Ok(key.verify(&payload_bytes, &signature).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_verify_request() {
        // Generate keypair
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let did = "did:example:123456789abcdefghi";

        // Create signed request
        let request = create_signed_request(did, &signing_key).expect("Failed to create request");

        // Verify the request
        let is_valid = verify_request(&request, &verifying_key).expect("Failed to verify request");
        assert!(is_valid, "Signature verification failed");

        // Test with tampered document
        let mut tampered_request = request.clone();
        tampered_request.document.id = "did:example:tampered".to_string();
        let is_valid_tampered = verify_request(&tampered_request, &verifying_key)
            .expect("Failed to verify tampered request");
        assert!(!is_valid_tampered, "Tampered signature should not verify");
    }
}
