//! Community loop (M7): the changelog parser, survey delivery rules, the
//! seed content and the queries (`store`). Shared types are in
//! `models::community`.

pub mod changelog;
pub mod seeds;
pub mod store;
pub mod surveys;

use crate::models::community::SurveyWording;

/// SHA-256 (hex) of the survey wording's canonical JSON. An approval holds
/// only while the stored `approved_hash` equals this.
pub fn wording_hash(w: &SurveyWording) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(w.canonical_json().as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wording_hash_is_hex_sha256_and_stable() {
        let w = SurveyWording {
            question: "Q?".into(),
            kind: "short_text".into(),
            ..Default::default()
        };
        let h = wording_hash(&w);
        assert_eq!(h.len(), 64);
        assert_eq!(h, wording_hash(&w.clone()));
        let mut w2 = w;
        w2.question = "Q!".into();
        assert_ne!(h, wording_hash(&w2));
    }
}
