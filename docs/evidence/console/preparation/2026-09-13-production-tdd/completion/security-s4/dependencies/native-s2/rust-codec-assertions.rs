//! Actual proposed Rust codec assertion bodies, intended as in-owner module tests.
//! No SQLx application dependency, no fake owner. Missing native28 module is an
//! implementation prerequisite, not claimed meaningful product RED.
use super::native28::{decode_input_stream,encode_input_stream,decode_calculation_stream,encode_calculation_stream,InputFrame,CalculationFrame};
use console_ontology_application::owner28::{decode_owner_submission,encode_owner_submission,NormalizedOwnerSubmission};
use std::io::Cursor;
use sha2::{Digest,Sha256};
fn digest(bytes:&[u8])->String{hex::encode(Sha256::digest(bytes))}

#[test]
fn rust_input_blocked_two_matches_independent_literal_bytes() {
    let expected=include_bytes!("codec-goldens/input-blocked-two.bin");
    assert_eq!(expected.len(),4090);
    assert_eq!(digest(expected),"a426f26631b5d388919c3205295253fefc8cc2cde5e5a74fd4bbe98b7efb9b69");
    let fixture:InputFrame=serde_json::from_slice(include_bytes!("codec-goldens/input-blocked-two.json")).unwrap();
    let mut actual=Vec::new();let summary=encode_input_stream(&fixture,&mut actual).unwrap();
    assert_eq!(actual.as_slice(),expected);
    assert_eq!(summary.manifest18_digest,"d87be74fb748af576671108b06e1bdc1252d0d3e0796c003d087dbfaab173a69");
    let decoded=decode_input_stream(&mut Cursor::new(expected)).unwrap();
    assert_eq!(decoded.frame,fixture);
}

#[test]
fn rust_input_resolved_one_matches_independent_literal_bytes() {
    let expected=include_bytes!("codec-goldens/input-resolved-one.bin");
    assert_eq!(expected.len(),4364);
    assert_eq!(digest(expected),"4a93f3889dfe4eb2a2617a83452202665a0e6afd9edaf998ea6bf8084cf009c4");
    let fixture:InputFrame=serde_json::from_slice(include_bytes!("codec-goldens/input-resolved-one.json")).unwrap();
    let mut actual=Vec::new();let summary=encode_input_stream(&fixture,&mut actual).unwrap();
    assert_eq!(actual.as_slice(),expected);
    assert_eq!(summary.manifest18_digest,"9f7b6c4dde388f570f6170d122bd0af48b9cff018af9e4d11598624ca0a9b140");
    let decoded=decode_input_stream(&mut Cursor::new(expected)).unwrap();
    assert_eq!(decoded.frame,fixture);
}

#[test]
fn rust_calculation_blocked_one_matches_independent_literal_bytes() {
    let expected=include_bytes!("codec-goldens/calculation-blocked-one.bin");
    assert_eq!(expected.len(),1431);
    assert_eq!(digest(expected),"00438949c40484663b4ffbc74fa529dcc03c708bcbf6f3411eeb5809f9cc0a8f");
    let input=decode_input_stream(&mut Cursor::new(include_bytes!("codec-goldens/input-resolved-one.bin"))).unwrap();
    let fixture:CalculationFrame=serde_json::from_slice(include_bytes!("codec-goldens/calculation-blocked-one.json")).unwrap();
    let mut actual=Vec::new();let summary=encode_calculation_stream(&fixture,&input.frame,&mut actual).unwrap();
    assert_eq!(actual.as_slice(),expected);
    assert_eq!(summary.manifest18_digest,"1512c930e4e88f47d9636e13521e87b9ac7fa5991841836bfc65008861dcbc76");
    let decoded=decode_calculation_stream(&mut Cursor::new(expected),&input.frame).unwrap();
    assert_eq!(decoded.frame,fixture);
}

#[test]
fn rust_calculation_success_one_matches_independent_literal_bytes() {
    let expected=include_bytes!("codec-goldens/calculation-success-one.bin");
    assert_eq!(expected.len(),1435);
    assert_eq!(digest(expected),"33bad30007efe3a43e07cc05ad34d9c19af72092b87b0227009e857ac27ec156");
    let input=decode_input_stream(&mut Cursor::new(include_bytes!("codec-goldens/input-resolved-one.bin"))).unwrap();
    let fixture:CalculationFrame=serde_json::from_slice(include_bytes!("codec-goldens/calculation-success-one.json")).unwrap();
    let mut actual=Vec::new();let summary=encode_calculation_stream(&fixture,&input.frame,&mut actual).unwrap();
    assert_eq!(actual.as_slice(),expected);
    assert_eq!(summary.manifest18_digest,"d356b586c87e51ec1c10fe6088cb5a687baaca4d4502073e0d6353f8768aa4a9");
    let decoded=decode_calculation_stream(&mut Cursor::new(expected),&input.frame).unwrap();
    assert_eq!(decoded.frame,fixture);
}

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
fn rust_input_decoder_rejects_truncation_every_byte_boundary() {
    let complete=include_bytes!("codec-goldens/input-resolved-one.bin");
    for end in 0..complete.len() {
        assert!(decode_input_stream(&mut Cursor::new(&complete[..end])).is_err(),"accepted truncation at {end}");
    }
}
#[test]
fn rust_native_decoder_rejects_trailing_bytes_and_bad_frame_lengths() {
    let mut trailing=include_bytes!("codec-goldens/input-resolved-one.bin").to_vec();trailing.push(0);
    assert!(decode_input_stream(&mut Cursor::new(trailing)).is_err());
    let mut oversized=include_bytes!("codec-goldens/input-resolved-one.bin").to_vec();
    oversized[41..45].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(decode_input_stream(&mut Cursor::new(oversized)).is_err());
}
#[test]
fn rust_calc_decoder_requires_exact_input_custody_not_semantics() {
    let mut input=decode_input_stream(&mut Cursor::new(include_bytes!("codec-goldens/input-resolved-one.bin"))).unwrap();
    let original=input.frame.pairs[0].1.semantic_digest.clone();
    input.frame.pairs[0].1.semantic_digest="f".repeat(64);
    assert_ne!(input.frame.pairs[0].1.semantic_digest,original);
    assert!(decode_calculation_stream(&mut Cursor::new(include_bytes!("codec-goldens/calculation-success-one.bin")),&input.frame).is_err());
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
