use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct VerifiablePresentation {
    holder_did: String,
    message: String,
    signature: String,
}
