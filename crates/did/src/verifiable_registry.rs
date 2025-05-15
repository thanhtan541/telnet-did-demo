use std::collections::HashMap;

use crate::DidDocument;

// Main storage structure for DID documents
pub struct DidStorage {
    documents: HashMap<String, DidDocument>,
}

impl DidStorage {
    // Create a new empty DID storage
    pub fn new() -> Self {
        DidStorage {
            documents: HashMap::new(),
        }
    }

    // Store a DID document
    pub fn store(&mut self, did: String, document: DidDocument) -> Result<(), String> {
        if did != document.id {
            return Err("DID and document ID must match".to_string());
        }
        self.documents.insert(did, document);
        Ok(())
    }

    // Retrieve a DID document
    pub fn get(&self, did: &str) -> Option<&DidDocument> {
        self.documents.get(did)
    }

    // Update an existing DID document
    pub fn update(&mut self, did: &str, document: DidDocument) -> Result<(), String> {
        if did != document.id {
            return Err("DID and document ID must match".to_string());
        }
        if !self.documents.contains_key(did) {
            return Err("DID not found".to_string());
        }
        self.documents.insert(did.to_string(), document);
        Ok(())
    }

    // Delete a DID document
    pub fn delete(&mut self, did: &str) -> Option<DidDocument> {
        self.documents.remove(did)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Service, VerificationMethod};

    use super::*;

    fn create_test_document(did: &str) -> DidDocument {
        // Create a new DID Document
        let mut did_doc = DidDocument::new(did);

        // Add a verification method
        let verification_method = VerificationMethod {
            id: "did:example:123456789abcdefghi#keys-1".to_string(),
            vc_type: "Ed25519VerificationKey2018".to_string(),
            controller: did.to_string(),
            public_key_hex: None,
            public_key_base58: Some("H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string()),
        };
        did_doc.add_verification_method(verification_method);

        // Add authentication
        did_doc.add_authentication("did:example:123456789abcdefghi#keys-1");

        // Add a service
        let service = Service {
            id: "did:example:123456789abcdefghi#vcs".to_string(),
            type_: "VerifiableCredentialService".to_string(),
            service_endpoint: "https://example.com/vc/".to_string(),
        };
        did_doc.add_service(service);

        did_doc
    }

    #[test]
    fn test_store_and_get() {
        let mut storage = DidStorage::new();
        let did = "did:example:123";
        let doc = create_test_document(did);

        // Test successful store
        assert!(storage.store(did.to_string(), doc.clone()).is_ok());

        // Test retrieval
        let retrieved = storage.get(did);
        assert!(retrieved.is_some());

        assert_eq!(
            &retrieved.unwrap().to_json().unwrap(),
            &doc.to_json().unwrap()
        );
    }

    #[test]
    fn test_store_invalid_did() {
        let mut storage = DidStorage::new();
        let did = "did:example:123";
        let mut doc = create_test_document(did);
        doc.id = "did:example:456".to_string();

        // Test storing with mismatched DID
        let result = storage.store(did.to_string(), doc);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "DID and document ID must match");
    }

    #[test]
    fn test_update() {
        let mut storage = DidStorage::new();
        let did = "did:example:123";
        let doc = create_test_document(did);

        // Store initial document
        storage.store(did.to_string(), doc.clone()).unwrap();

        // Create updated document
        let updated_doc = {
            let mut doc = create_test_document(did);
            doc.service = None;
            doc
        };

        // Test successful update
        assert!(
            storage.update(did, updated_doc.clone()).is_ok(),
            "Failed to update new doc"
        );

        // Verify update
        let retrieved = storage.get(did).unwrap();
        assert_eq!(
            &retrieved.to_json().unwrap(),
            &updated_doc.to_json().unwrap()
        );
    }

    #[test]
    fn test_update_nonexistent() {
        let mut storage = DidStorage::new();
        let did = "did:example:123";
        let doc = create_test_document(did);

        // Test updating non-existent DID
        let result = storage.update(did, doc);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "DID not found");
    }

    #[test]
    fn test_delete() {
        let mut storage = DidStorage::new();
        let did = "did:example:123";
        let doc = create_test_document(did);

        // Store document
        storage.store(did.to_string(), doc.clone()).unwrap();

        // Test successful deletion
        let deleted = storage.delete(did);
        assert!(deleted.is_some());
        assert_eq!(
            &deleted.unwrap().to_json().unwrap(),
            &doc.to_json().unwrap()
        );

        // Verify document is gone
        assert!(storage.get(did).is_none());
    }

    #[test]
    fn test_delete_nonexistent() {
        let mut storage = DidStorage::new();
        let did = "did:example:123";

        // Test deleting non-existent DID
        let deleted = storage.delete(did);
        assert!(deleted.is_none());
    }

    #[test]
    fn test_empty_storage() {
        let storage = DidStorage::new();
        assert!(storage.get("did:example:123").is_none());
    }
}
