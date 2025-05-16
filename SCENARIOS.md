# Prescriptions (healthcare)

Alice want to purchase medicine from her prescription

---

# Driver's license (Verifiable Credential)

## Alice has request her driver's license issued from Department of Transporation

#### Parties
- Issuer: Department of Transporation - DOT
- Subject: Alice, who receives the credential
- Holder: Alice, who requests and stores the credential
- Verifier: Department of Transporation

#### Alice's DID document
```JSON
{
  "@context": "https://www.w3.org/ns/did/v1",
  "id": "did:example:alice123",
  "verificationMethod": [
    {
      "id": "did:example:alice123#key-1",
      "type": "Ed25519VerificationKey2020",
      "controller": "did:example:alice123",
      "publicKeyMultibase": "z6Mkn..."
    }
  ],
  "authentication": ["did:example:alice123#key-1"]
}
```

#### DOT's DID document
```JSON
{
  "@context": "https://www.w3.org/ns/did/v1",
  "id": "did:example:dmv123456789",
  "verificationMethod": [
    {
      "id": "did:example:dmv123456789#key-1",
      "type": "Ed25519VerificationKey2020",
      "controller": "did:example:dmv123456789",
      "publicKeyMultibase": "z6Mkq..."
    }
  ],
  "assertionMethod": ["did:example:dmv123456789#key-1"]
}
```

#### Alice's Request for Driver's license
```JSON
{
  "type": "CredentialRequest",
  "to": "did:example:dmv123456789",
  "from": "did:example:alice123",
  "credentialType": "DriversLicenseCredential",
  "proof": {
    "type": "Ed25519Signature2020",
    "created": "2025-01-01T00:00:01Z",
    "verificationMethod": "did:example:alice123#key-1",
    "jws": "eyJhbGciOiJFZDI1NTE5Iiw... (base64url-encoded signature)"
  }
}
```

#### DOT verify request based on signature
```
Ed25519_Verify(alice_public_key, signature, SHA-256(canonicalized_message)) == true
```

#### DOT create Driver's license for Alice
```JSON
{
  "@context": [
    "https://www.w3.org/2018/credentials/v1",
    "https://example.org/contexts/drivers-license/v1"
  ],
  "id": "urn:uuid:12345678-1234-5678-1234-567812345678",
  "type": ["VerifiableCredential", "DriversLicenseCredential"],
  "issuer": {
    "id": "did:example:dmv123456789"
  },
  "issuanceDate": "2025-01-01T00:00:00Z",
  "expirationDate": "2030-01-01T00:00:00Z",
  "credentialSubject": {
    "id": "did:example:alice123",
    "driversLicense": {
      "licenseNumber": "D12345678",
      "name": "Alice Smith",
      "dateOfBirth": "1990-05-15",
      "licenseClass": "C"
    }
  },
  "credentialStatus": {
    "id": "https://dmv.example.org/revocation-list#123",
    "type": "RevocationList2020Status",
    "revocationListIndex": "123",
    "revocationListCredential": "https://dmv.example.org/revocation-list"
  },
  "proof": {
    "type": "Ed25519Signature2020",
    "created": "2025-01-01T00:00:01Z",
    "proofPurpose": "assertionMethod",
    "verificationMethod": "did:example:dmv123456789#key-1",
    "jws": "eyJhbGciOiJFZDI1NTE5Iiw... (base64url-encoded signature)"
  }
}
```

---

## Alice has her driver's license issued from Department of Transporation and used it to rent a car

#### Parties
- Issuer: Department of Transporation - DOT
- Subject: Alice, who receives the credential
- Holder: Alice, who stores and presents the credential
- Verifier: A car rental company that check the credential

#### Verifiable Credential Example (JSON-LD)
```json
{
  "@context": [
    "https://www.w3.org/2018/credentials/v1",
    "https://example.org/contexts/drivers-license/v1"
  ],
  "id": "urn:uuid:12345678-1234-5678-1234-567812345678",
  "type": ["VerifiableCredential", "DriversLicenseCredential"],
  "issuer": {
    "id": "did:example:dmv123456789"
  },
  "issuanceDate": "2025-01-01T00:00:00Z",
  "expirationDate": "2030-01-01T00:00:00Z",
  "credentialSubject": {
    "id": "did:example:alice123",
    "driversLicense": {
      "licenseNumber": "D12345678",
      "name": "Alice Smith",
      "dateOfBirth": "1990-05-15",
      "licenseClass": "C",
      "restrictions": "None"
    }
  },
  "proof": {
    "type": "Ed25519Signature2020",
    "created": "2025-01-01T00:00:01Z",
    "proofPurpose": "assertionMethod",
    "verificationMethod": "did:example:dmv123456789#key-1",
    "jws": "eyJhbGciOiJFZDI1NTE5Iiw... (base64-encoded signature)"
  }
}
```

## Steps:
1. Issuance:
- The DMV creates the credential, signs it with its private key, and sends it to Alice (via a digital wallet, for example).
2. Storage:
- Alice stores the credential in her digital wallet, which is associated with her DID (did:example:alice123).
3. Presentation:
- When renting a car, Alice presents the credential to the car rental company. She might use selective disclosure to share only her license class and proof of being over 21, without revealing her full date of birth or license number.
4. Verification:
- The car rental company:
  - Resolves the DMV's DID (did:example:dmv123456789) to retrieve the public key from the DID Document.
  - Verifies the signature in the proof section to ensure the credential hasn't been tampered with.
  - Checks the issuanceDate and expirationDate to confirm validity.
  - Confirms Alice's DID matches the credentialSubject to ensure she is the rightful holder.

---
# ShieldNetID (Identify)
Alice joined ShieldNetId

## Steps
Alice create join ShieldNetID and get her bronze badge
Her badge has information about credit score
Should she get a VC from Shieldnet

---
