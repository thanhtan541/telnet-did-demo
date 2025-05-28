use serde_json::json;
use ssi::claims::vc::v2::JsonCredential;
use ssi::dids::{AnyDidMethod, VerificationMethodDIDResolver};
use ssi::prelude::*;
use ssi::JWK;

#[cfg(test)]
mod tests {

    use ssi::claims::vc::v2::Credential;

    use super::*;

    #[async_std::test]
    async fn bbs_2023() {
        use json_syntax::Value;

        // Keypair
        let jwk = JWK::generate_bls12381g2();
        // Public key
        let did_url = ssi::dids::DIDKey::generate_url(&jwk).unwrap();
        println!("{jwk}");

        let resolver = VerificationMethodDIDResolver::<_, AnyMethod>::new(AnyDidMethod::default());
        let vc: JsonCredential = serde_json::from_value(json!({
            "@context": [
                "https://www.w3.org/ns/credentials/v2",
                {
                    "age": "http://example.org/#age",
                    "single": "http://example.org/#single",
                }
            ],
            "type": [
                "VerifiableCredential"
            ],
            "credentialSubject": {
                "id": "did:key:z6MkhTNL7i2etLerDK8Acz5t528giE5KA4p75T6ka1E1D74r",
                "age": "18",
                "single": "yes",
            },
            "id": "urn:uuid:7a6cafb9-11c3-41a8-98d8-8b5a45c2548f",
            "issuer": did_url.to_string()
        }))
        .unwrap();

        let base_vc = AnySuite::Bbs2023
            .sign(
                vc,
                &resolver,
                SingleSecretSigner::new(jwk).into_local(),
                ProofOptions::from_method(did_url.into_iri().into()),
            )
            .await
            .unwrap();
        println!(
            "Based Verifiable Credential Subjects, {:?}",
            base_vc.credential_subjects()
        );

        let params = VerificationParameters::from_resolver(&resolver);
        let mut selection = ssi::claims::data_integrity::AnySelectionOptions::default();
        selection.selective_pointers = vec![
            "/id".parse().unwrap(),
            "/type".parse().unwrap(),
            "/credentialSubject/age".parse().unwrap(),
            "/issuer".parse().unwrap(),
        ];
        let derived = base_vc
            .select(&params, selection)
            .await
            .unwrap()
            .map(|object| {
                ssi::json_ld::syntax::from_value::<JsonCredential>(Value::Object(object)).unwrap()
            });

        derived.verify(params).await.unwrap().unwrap();
        println!(
            "Dervired Verifiable Credential Subjects {:?}",
            derived.credential_subjects().to_vec()
        );

        assert!(false);
    }
}
