//! Six exact generic owner framing tests extracted from the frozen native-s2 packet.
//! Literal operands do not construct authenticated context or sealed capabilities.
//! The seven native framing tests remain in the frozen source, outside this lane.
use crate::owner28::{decode_owner_submission,encode_owner_submission,NormalizedOwnerSubmission};
use std::io::Cursor;
use sha2::{Digest,Sha256};
fn digest(bytes:&[u8])->String{hex::encode(Sha256::digest(bytes))}

#[test]
fn rust_owner_direct_wage_matches_independent_literal_bytes() {
    let expected=include_bytes!("codec-goldens/owner-direct-wage.bin");
    assert_eq!(expected.len(),1872);
    assert_eq!(digest(expected),"8d27899205f60348204015c80d3dfa2c09db8caca3241426abc6c2ce0665170b");
    let fixture:NormalizedOwnerSubmission=serde_json::from_slice(include_bytes!("codec-goldens/owner-direct-wage.json")).unwrap();
    let mut actual=Vec::new();let summary=encode_owner_submission(&fixture,&mut actual).unwrap();
    assert_eq!(actual.as_slice(),expected);
    assert_eq!(summary.command_fingerprint,"27a9fe6ee36df58348552d381dfeadfef06b06c551fe41b062aedc7cdb7fcc09");
    assert_eq!(decode_owner_submission(&mut Cursor::new(expected)).unwrap().command,fixture);
}

#[test]
fn rust_owner_attempt_create_run_matches_independent_literal_bytes() {
    let expected=include_bytes!("codec-goldens/owner-attempt-create-run.bin");
    assert_eq!(expected.len(),2360);
    assert_eq!(digest(expected),"e962a57c7ed5f5e397fd607390e58561d3b40373e5bb091ab6298f2c967383df");
    let fixture:NormalizedOwnerSubmission=serde_json::from_slice(include_bytes!("codec-goldens/owner-attempt-create-run.json")).unwrap();
    let mut actual=Vec::new();let summary=encode_owner_submission(&fixture,&mut actual).unwrap();
    assert_eq!(actual.as_slice(),expected);
    assert_eq!(summary.command_fingerprint,"b2ebcacabd4e8e0c289d9738c7e83ebf74faf10f534c3545111853fe29d51011");
    assert_eq!(decode_owner_submission(&mut Cursor::new(expected)).unwrap().command,fixture);
}

#[test]
fn rust_owner_gated_calculate_matches_independent_literal_bytes() {
    let expected=include_bytes!("codec-goldens/owner-gated-calculate.bin");
    assert_eq!(expected.len(),1948);
    assert_eq!(digest(expected),"a08be2ac3ff8f5ad657bbe4a5fb8494956269af5e35949af4ff3ff7e721afcf0");
    let fixture:NormalizedOwnerSubmission=serde_json::from_slice(include_bytes!("codec-goldens/owner-gated-calculate.json")).unwrap();
    let mut actual=Vec::new();let summary=encode_owner_submission(&fixture,&mut actual).unwrap();
    assert_eq!(actual.as_slice(),expected);
    assert_eq!(summary.command_fingerprint,"55ac174164f9e6e4a14b1e482db54f56ef1065eba4307953ee1c46da40fe1a8a");
    assert_eq!(decode_owner_submission(&mut Cursor::new(expected)).unwrap().command,fixture);
}

#[test]
fn rust_owner_publication_matches_independent_literal_bytes() {
    let expected=include_bytes!("codec-goldens/owner-publication.bin");
    assert_eq!(expected.len(),2857);
    assert_eq!(digest(expected),"9489ebfea27dfc175711bb65425dff04628d7a2fdff2296a0084b77aa81ef6ad");
    let fixture:NormalizedOwnerSubmission=serde_json::from_slice(include_bytes!("codec-goldens/owner-publication.json")).unwrap();
    let mut actual=Vec::new();let summary=encode_owner_submission(&fixture,&mut actual).unwrap();
    assert_eq!(actual.as_slice(),expected);
    assert_eq!(summary.command_fingerprint,"f57c46cc27e9ca2e80469729194fcade41d4de7a04ea47699ef72cef0555eaa7");
    assert_eq!(decode_owner_submission(&mut Cursor::new(expected)).unwrap().command,fixture);
}

#[test]
fn rust_owner_case_response_matches_independent_literal_bytes() {
    let expected=include_bytes!("codec-goldens/owner-case-response.bin");
    assert_eq!(expected.len(),2138);
    assert_eq!(digest(expected),"df2fcb0667d45d82075f504c5d36b07cb2be7639dcbf45142fc81e05a44326d5");
    let fixture:NormalizedOwnerSubmission=serde_json::from_slice(include_bytes!("codec-goldens/owner-case-response.json")).unwrap();
    let mut actual=Vec::new();let summary=encode_owner_submission(&fixture,&mut actual).unwrap();
    assert_eq!(actual.as_slice(),expected);
    assert_eq!(summary.command_fingerprint,"9b17c48e47e0ebfb6b06764815ef9badf713bf2e34a9b4b6ff17e8db0c1944ac");
    assert_eq!(decode_owner_submission(&mut Cursor::new(expected)).unwrap().command,fixture);
}

#[test]
fn rust_owner_decoder_rejects_public_numeric_tokens_duplicate_keys_and_future_gate_digest() {
    // Each mutation uses retained canonical bytes from a positive literal. Reject
    // before JSONB/Value duplicate-key loss or current owner admission.
    let positive=include_bytes!("codec-goldens/owner-gated-calculate.bin");
    let body=std::str::from_utf8(&positive[34..]).unwrap();
    for changed in [
        body.replacen("\"protocol\":\"CCF1\"","\"protocol\":\"CCF1\",\"protocol\":\"CCF1\"",1),
        body.replacen("\"run_revision\":\"1\"","\"run_revision\":1",1),
        body.replacen("\"gate_ref\":{","\"gate_ref\":{\"binding_digest\":\"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\",",1),
    ] {
        assert_ne!(changed,body,"fixture mutation must change actual bytes");
        let mut frame=b"console.owner28.submission.v1\0".to_vec();
        frame.extend_from_slice(&(changed.len() as u32).to_be_bytes());frame.extend_from_slice(changed.as_bytes());
        assert!(decode_owner_submission(&mut Cursor::new(frame)).is_err());
    }
}
