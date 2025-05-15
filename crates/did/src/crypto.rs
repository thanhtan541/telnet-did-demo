use ed25519_dalek::VerifyingKey;
use multibase;
use std::error::Error;

pub fn encode_public_key_to_multibase(public_key: &VerifyingKey) -> Result<String, Box<dyn Error>> {
    let public_key_bytes: [u8; 32] = public_key.to_bytes();

    let mut multicodec_key: Vec<u8> = vec![0xed, 0x01];
    multicodec_key.extend_from_slice(&public_key_bytes);

    let multibase_key = multibase::encode(multibase::Base::Base58Btc, &multicodec_key);

    Ok(multibase_key)
}

pub fn decode_multibase_to_public_key(multibase_key: &str) -> Result<VerifyingKey, Box<dyn Error>> {
    let (base, decoded_bytes) = multibase::decode(multibase_key)?;
    if base != multibase::Base::Base58Btc {
        return Err("Expected base58btc encoding".into());
    }

    if decoded_bytes.len() != 34 || decoded_bytes[0] != 0xed || decoded_bytes[1] != 0x01 {
        return Err("Invalid multicodec prefix or length".into());
    }

    let public_key_bytes: [u8; 32] = decoded_bytes[2..34]
        .try_into()
        .map_err(|_| "Invalid public key length")?;
    let public_key = VerifyingKey::from_bytes(&public_key_bytes)?;

    Ok(public_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    // Test encoding a public key to publicKeyMultibase
    #[test]
    fn test_encode_public_key() {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        let verifying_key: VerifyingKey = signing_key.verifying_key();

        let result = encode_public_key_to_multibase(&verifying_key).unwrap();

        // Check that the result starts with 'z' (base58btc)
        assert!(
            result.starts_with('z'),
            "Multibase string should start with 'z'"
        );

        // Decode back to verify
        let decoded_bytes = multibase::decode(&result).unwrap().1;
        assert_eq!(
            decoded_bytes.len(),
            34,
            "Decoded bytes should be 34 bytes long"
        );
        assert_eq!(decoded_bytes[0], 0xed, "First byte should be 0xed");
        assert_eq!(decoded_bytes[1], 0x01, "Second byte should be 0x01");
        assert_eq!(
            &decoded_bytes[2..34],
            &verifying_key.to_bytes(),
            "Decoded public key should match original"
        );
    }

    // Test decoding a publicKeyMultibase string
    #[test]
    fn test_decode_public_key() {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        let verifying_key: VerifyingKey = signing_key.verifying_key();

        // Encode to multibase
        let multibase_key = encode_public_key_to_multibase(&verifying_key).unwrap();

        // Decode back
        let decoded_public_key = decode_multibase_to_public_key(&multibase_key).unwrap();

        // Verify the decoded public key matches the original
        assert_eq!(
            decoded_public_key.to_bytes(),
            verifying_key.to_bytes(),
            "Decoded public key should match original"
        );
    }

    // Test round-trip: encode -> decode -> compare
    #[test]
    fn test_round_trip() {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        let verifying_key: VerifyingKey = signing_key.verifying_key();

        let multibase_key = encode_public_key_to_multibase(&verifying_key).unwrap();
        let decoded_public_key = decode_multibase_to_public_key(&multibase_key).unwrap();

        assert_eq!(
            decoded_public_key.to_bytes(),
            verifying_key.to_bytes(),
            "Round-trip public key should match original"
        );
    }

    // Test invalid multibase string
    #[test]
    fn test_decode_invalid_multibase() {
        let invalid_multibase = "f12345"; // Invalid base (f = base64)
        let result = decode_multibase_to_public_key(invalid_multibase);
        assert!(result.is_err(), "Decoding invalid multibase should fail");
    }

    // Test invalid multicodec prefix
    #[test]
    fn test_decode_invalid_multicodec() {
        let mut csprng = OsRng;
        let keypair: SigningKey = SigningKey::generate(&mut csprng);
        let public_key_bytes = keypair.verifying_key().to_bytes();

        // Use wrong multicodec prefix (e.g., 0x00 instead of 0xed01)
        let mut invalid_multicodec: Vec<u8> = vec![0x00, 0x00];
        invalid_multicodec.extend_from_slice(&public_key_bytes);
        let invalid_multibase = multibase::encode(multibase::Base::Base58Btc, &invalid_multicodec);

        let result = decode_multibase_to_public_key(&invalid_multibase);
        assert!(result.is_err(), "Decoding invalid multicodec should fail");
    }
}
