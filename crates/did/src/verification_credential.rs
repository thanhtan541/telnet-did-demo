use base58::{FromBase58, ToBase58};

use chrono::Utc;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;

// Define the Verifiable Credential structure based on W3C VC Data Model
#[derive(Serialize, Deserialize, Clone)]
struct VerifiableCredential {
    #[serde(rename = "@context")]
    context: Vec<String>,
    id: String,
    #[serde(rename = "type")]
    credential_type: Vec<String>,
    issuer: String,
    #[serde(rename = "issuanceDate")]
    issuance_date: String,
    #[serde(rename = "credentialSubject")]
    credential_subject: CredentialSubject,
    proof: Proof,
}

// Define the CredentialSubject for creditworthiness claims
#[derive(Serialize, Deserialize, Clone)]
struct CredentialSubject {
    id: String,
    #[serde(rename = "creditScore")]
    credit_score: u32,
    #[serde(rename = "scoreRange")]
    score_range: String,
    #[serde(rename = "evaluationDate")]
    evaluation_date: String,
    #[serde(rename = "confidenceLevel")]
    confidence_level: String,
}

// Define the Proof for the digital signature
#[derive(Serialize, Deserialize, Clone)]
struct Proof {
    #[serde(rename = "type")]
    proof_type: String,
    created: String,
    #[serde(rename = "proofPurpose")]
    proof_purpose: String,
    #[serde(rename = "verificationMethod")]
    verification_method: String,
    #[serde(rename = "proofValue")]
    proof_value: Option<String>, // Base58-encoded signature
}

// Custom error type for VC operations
#[derive(Debug)]
struct VCError(String);

impl std::fmt::Display for VCError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "VC Error: {}", self.0)
    }
}

impl Error for VCError {}

// VC generation and verification logic
struct VCCreator {
    issuer_did: String,
    signer: SigningKey,
}

impl VCCreator {
    // Initialize the VC creator with a DID and generate a keypair
    fn new(issuer_did: &str) -> Self {
        let mut csprng = OsRng {};
        let signer = SigningKey::generate(&mut csprng);
        VCCreator {
            issuer_did: issuer_did.to_string(),
            signer,
        }
    }

    // Generate a Verifiable Credential for Alice
    fn generate_vc(
        &self,
        subject_did: &str,
        credit_score: u32,
    ) -> Result<VerifiableCredential, Box<dyn Error>> {
        let now = Utc::now();
        let issuance_date = now.to_rfc3339();
        let evaluation_date = now.date_naive().to_string();

        // Create the credential subject
        let credential_subject = CredentialSubject {
            id: subject_did.to_string(),
            credit_score,
            score_range: "0-850".to_string(),
            evaluation_date,
            confidence_level: "High".to_string(),
        };

        // Create the unsigned VC
        let vc = VerifiableCredential {
            context: vec![
                "https://www.w3.org/2018/credentials/v1".to_string(),
                "https://schema.creditscoringcompany.com/creditworthiness/v1".to_string(),
            ],
            id: format!(
                "http://creditscoringcompany.com/credentials/{}",
                uuid::Uuid::new_v4()
            ),
            credential_type: vec![
                "VerifiableCredential".to_string(),
                "CreditworthinessCredential".to_string(),
            ],
            issuer: self.issuer_did.clone(),
            issuance_date,
            credential_subject,
            proof: Proof {
                proof_type: "Ed25519Signature2020".to_string(),
                created: now.to_rfc3339(),
                proof_purpose: "assertionMethod".to_string(),
                verification_method: format!("{}#key-1", self.issuer_did),
                proof_value: None, // Placeholder, will be replaced
            },
        };

        // Serialize VC to JSON for signing (excluding proof.jws)
        let vc_for_signing = vc.clone();
        let vc_json = serde_json::to_string(&vc_for_signing)?;

        // Sign the JSON string
        let signature = self.signer.sign(vc_json.as_bytes());
        let signature = signature.to_bytes().to_base58();

        // Update the VC with the signature
        let mut signed_vc = vc;
        signed_vc.proof.proof_value = Some(signature);

        Ok(signed_vc)
    }

    // Get the public key for verification
    fn verifying_key(&self) -> VerifyingKey {
        self.signer.verifying_key()
    }
}

// Verify a Verifiable Credential
fn verify_vc(vc: &VerifiableCredential, vr_key: &VerifyingKey) -> Result<bool, Box<dyn Error>> {
    // Create a copy of the VC with proof.jws set to empty for verification
    let mut vc_for_verification = vc.clone();
    vc_for_verification.proof.proof_value = None;
    let vc_json = serde_json::to_string(&vc_for_verification)
        .unwrap()
        .into_bytes();

    // Decode and verify signature
    let signature_bytes = vc.proof.proof_value.clone();
    let signature_bytes = signature_bytes.unwrap().from_base58().unwrap();
    let signature: Signature = Signature::try_from(&signature_bytes[..64]).unwrap();

    Ok(vr_key.verify(&vc_json, &signature).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify_vc() {
        // Initialize the issuer
        let issuer_did = "did:web:creditscoringcompany.com";
        let vc_creator = VCCreator::new(issuer_did);

        // Generate a VC for Alice
        let subject_did = "did:ion:123456789abcdef";
        let credit_score = 750;
        let vc = vc_creator.generate_vc(subject_did, credit_score).unwrap();

        // Verify the VC
        let vr_key = vc_creator.verifying_key();
        let is_valid = verify_vc(&vc, &vr_key).unwrap();
        println!("[test_generate_and_verify_vc] {}", is_valid);
        assert!(is_valid, "VC verification should succeed");

        // Check VC contents
        assert_eq!(vc.issuer, issuer_did);
        assert_eq!(vc.credential_subject.id, subject_did);
        assert_eq!(vc.credential_subject.credit_score, credit_score);
        assert_eq!(vc.credential_subject.score_range, "0-850");
        assert_eq!(vc.credential_subject.confidence_level, "High");
        assert_eq!(vc.proof.proof_type, "Ed25519Signature2020");
    }

    #[test]
    fn test_verify_tampered_vc() {
        let issuer_did = "did:web:creditscoringcompany.com";
        let vc_creator = VCCreator::new(issuer_did);
        let subject_did = "did:ion:123456789abcdef";
        let credit_score = 750;

        // Generate a VC
        let mut vc = vc_creator.generate_vc(subject_did, credit_score).unwrap();

        // Tamper with the credit score
        vc.credential_subject.credit_score = 800;

        // Verify the tampered VC
        let vr_key = vc_creator.verifying_key();
        let is_valid = verify_vc(&vc, &vr_key).unwrap();
        assert!(!is_valid, "Tampered VC verification should fail");
    }

    #[test]
    fn test_verify_invalid_signature() {
        let issuer_did = "did:web:creditscoringcompany.com";
        let vc_creator = VCCreator::new(issuer_did);
        let subject_did = "did:ion:123456789abcdef";
        let credit_score = 750;

        // Generate a VC
        let mut vc = vc_creator.generate_vc(subject_did, credit_score).unwrap();

        // Decode and verify signature
        let signature: Signature = Signature::try_from([1u8; 64]).unwrap();
        // Replace the signature with an invalid one
        vc.proof.proof_value = Some(signature.to_bytes().to_base58());

        // Verify the VC
        let vr_key = vc_creator.verifying_key();
        let result = verify_vc(&vc, &vr_key);
        assert!(
            result.is_ok(),
            "VC with invalid signature should success to process"
        );

        let result = result.unwrap();
        assert!(!result, "VC with invalid signature should return to false");
    }
}
