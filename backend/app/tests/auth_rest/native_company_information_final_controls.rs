//! Controls exercise the actual relay grammar; none contacts a database or prints rows.
use super::wire::{
    Fault, Grammar,
    codec::{self, Kind},
};
use serde_json::json;
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;
fn ids() -> [Uuid; 4] {
    [1, 2, 3, 4].map(Uuid::from_u128)
}
fn group() -> Uuid {
    Uuid::from_u128(5)
}
fn string(s: &str) -> Vec<u8> {
    let mut b = s.as_bytes().to_vec();
    b.push(0);
    b
}
fn roster(kind: Kind) -> &'static [(&'static str, u32)] {
    if kind == Kind::Manager {
        codec::MANAGER_SCHEMA
    } else {
        codec::AUTH_SCHEMA
    }
}
fn prepare(g: &mut Grammar, name: &str, kind: Kind) {
    let sql = if kind == Kind::Manager {
        codec::MANAGER
    } else {
        codec::AUTH
    };
    let n = if kind == Kind::Manager { 5u16 } else { 2u16 };
    let mut p = string(name);
    p.extend(string(sql));
    p.extend(n.to_be_bytes());
    for _ in 0..n {
        p.extend(2950u32.to_be_bytes());
    }
    assert!(g.upload(b'P', &p).is_ok());
    let mut d = vec![b'S'];
    d.extend(string(name));
    assert!(g.upload(b'D', &d).is_ok());
    assert!(g.download(b'1', &[]).is_ok());
    let mut t = n.to_be_bytes().to_vec();
    for _ in 0..n {
        t.extend(2950u32.to_be_bytes());
    }
    assert!(g.download(b't', &t).is_ok());
    let columns = roster(kind);
    let mut t = (columns.len() as u16).to_be_bytes().to_vec();
    for (name, oid) in columns {
        t.extend(string(name));
        t.extend(0u32.to_be_bytes());
        t.extend(0u16.to_be_bytes());
        t.extend(oid.to_be_bytes());
        t.extend(u16::MAX.to_be_bytes());
        t.extend(u32::MAX.to_be_bytes());
        t.extend(0u16.to_be_bytes());
    }
    assert!(g.download(b'T', &t).is_ok());
}
fn bind(kind: Kind, final_call: bool, format: u16) -> Vec<u8> {
    let mut out = string("p");
    out.extend(string(if kind == Kind::Manager { "m" } else { "a" }));
    out.extend(1u16.to_be_bytes());
    out.extend(format.to_be_bytes());
    let mut params = ids()
        .into_iter()
        .take(if kind == Kind::Manager { 4 } else { 2 })
        .map(|id| Some(codec::uuid_bytes(id, format)))
        .collect::<Vec<_>>();
    if kind == Kind::Manager {
        params.push(final_call.then(|| codec::uuid_bytes(group(), format)));
    }
    out.extend(codec::write_cells(&params));
    out.extend(1u16.to_be_bytes());
    out.extend(format.to_be_bytes());
    out
}
fn execute(g: &mut Grammar, kind: Kind, final_call: bool, format: u16) -> Vec<u8> {
    let request = bind(kind, final_call, format);
    let forwarded = g.upload(b'B', &request).unwrap();
    assert!(g.upload(b'E', &execution()).is_ok());
    let mut close = vec![b'P'];
    close.extend(string("p"));
    assert!(g.upload(b'C', &close).is_ok());
    assert!(g.upload(b'S', &[]).is_ok());
    forwarded
}
fn execution() -> Vec<u8> {
    let mut e = string("p");
    e.extend(0u32.to_be_bytes());
    e
}
fn row(kind: Kind, final_call: bool, format: u16) -> Vec<u8> {
    let mut cells = Vec::new();
    let original = ids();
    for (name, oid) in roster(kind) {
        let value = match *oid {
            2950 => {
                if (*name == "org_id" && kind == Kind::Auth)
                    || (*name == "current_policy_receipt_id" && kind == Kind::Manager)
                {
                    None
                } else {
                    let id = match *name {
                        "actor_account_id" | "user_id" => original[0],
                        "session_id" => original[1],
                        "org_id" => original[2],
                        "command_id" => original[3],
                        "current_group_id" => group(),
                        _ => Uuid::from_u128(6),
                    };
                    Some(codec::uuid_bytes(id, format))
                }
            }
            20 | 23 => {
                let n = if *name == "source_backend_pid" {
                    12345
                } else {
                    1
                };
                Some(if format == 0 {
                    n.to_string().into_bytes()
                } else if *oid == 23 {
                    (n as i32).to_be_bytes().to_vec()
                } else {
                    (n as i64).to_be_bytes().to_vec()
                })
            }
            25 => {
                let text = match *name {
                    "security_state" => "ACTIVE",
                    "protocol" => "ACCOUNT_V1",
                    "assurance" => "PASSKEY_PRIMARY",
                    "source_xid" => "100",
                    _ => "opaque",
                };
                Some(text.as_bytes().to_vec())
            }
            1184 => {
                if *name == "revoked_at" {
                    None
                } else {
                    let n =
                        800_000_000_000_000i64 + i64::from(final_call && *name == "observed_at");
                    Some(if format == 1 {
                        n.to_be_bytes().to_vec()
                    } else {
                        (OffsetDateTime::from_unix_timestamp(946684800).unwrap()
                            + Duration::microseconds(n))
                        .format(&Rfc3339)
                        .unwrap()
                        .into_bytes()
                    })
                }
            }
            3802 => {
                let v = if *name == "registered_clauses" {
                    json!([])
                } else {
                    json!({"request":{"codec_version":4,"operation":"Grant","account_id":original[0],"session_id":original[1],"org_id":original[2],"command_id":original[3]},"account":{"id":original[0]},"family":{"id":original[1]},"company_head":{"org_id":original[2],"epoch":1,"current_policy_receipt_id":null},"company":{"created_at":"2020-01-01T00:00:00Z","updated_at":"2020-01-01T00:00:00Z"}})
                };
                let mut b = serde_json::to_vec(&v).unwrap();
                if format == 1 {
                    b.insert(0, 1);
                }
                Some(b)
            }
            _ => panic!("control roster changed"),
        };
        cells.push(value);
    }
    codec::write_cells(&cells)
}
fn finish(g: &mut Grammar, kind: Kind, final_call: bool, format: u16) {
    let sent = execute(g, kind, final_call, format);
    if g.fault == Fault::Group && kind == Kind::Manager && final_call {
        let original = bind(kind, final_call, format);
        let old = codec::uuid_bytes(group(), format);
        let at = original.windows(old.len()).position(|b| b == old).unwrap();
        let mut bytes = *group().as_bytes();
        bytes[0] ^= 0x80;
        let replacement = codec::uuid_bytes(Uuid::from_bytes(bytes), format);
        let mut expected = original.clone();
        expected[at..at + old.len()].copy_from_slice(&replacement);
        assert!(
            sent == expected && sent != original,
            "Group fault changed bytes beyond fifth UUID"
        );
        let mut e = vec![b'C'];
        e.extend(string("40001"));
        e.push(b'M');
        e.extend(string("company_information.lock_plan_changed"));
        e.push(0);
        assert!(g.download(b'E', &e).is_ok());
        assert!(g.download(b'Z', b"E").is_ok());
        return;
    }
    assert!(sent == bind(kind, final_call, format));
    let actual = row(kind, final_call, format);
    let response = g.download(b'D', &actual).unwrap();
    let dropped = final_call
        && ((kind == Kind::Manager && g.fault == Fault::SourceZero)
            || (kind == Kind::Auth && g.fault == Fault::AuthZero));
    let drift = final_call && kind == Kind::Manager && g.fault == Fault::Material;
    if dropped {
        assert!(response.is_none());
    } else if drift {
        let changed = codec::row(&response.unwrap()).unwrap();
        let original = codec::row(&actual).unwrap();
        assert!(
            changed[..19] == original[..19] && changed[19] != original[19],
            "drift changed an output besides complete material"
        );
    } else {
        assert!(
            response == Some(actual),
            "transparent relay changed actual result bytes"
        );
    }
    let complete = g.download(b'C', b"SELECT 1\0").unwrap();
    assert!(
        complete
            == Some(if dropped {
                b"SELECT 0\0".to_vec()
            } else {
                b"SELECT 1\0".to_vec()
            })
    );
    assert!(g.download(b'Z', b"T").is_ok());
}
fn attempt(g: &mut Grammar, format: u16) {
    finish(g, Kind::Manager, false, format);
    finish(g, Kind::Auth, false, format);
    finish(g, Kind::Manager, true, format);
    if g.fault != Fault::Group {
        finish(g, Kind::Auth, true, format);
    }
    let command = if g.fault == Fault::Transparent {
        "COMMIT"
    } else {
        "ROLLBACK"
    };
    assert!(g.upload(b'Q', &string(command)).is_ok());
    assert!(g.download(b'C', &string(command)).is_ok());
    assert!(g.download(b'Z', b"I").is_ok());
}
fn grammar() -> Grammar {
    let mut g = Grammar::new(ids(), group(), 1, None);
    let mut k = 12345i32.to_be_bytes().to_vec();
    k.extend([0; 4]);
    assert!(g.download(b'K', &k).is_ok());
    prepare(&mut g, "m", Kind::Manager);
    prepare(&mut g, "a", Kind::Auth);
    g
}

#[test]
fn transparent_and_four_faults_retain_cached_statement_result_formats() {
    for format in [0, 1] {
        for fault in [
            Fault::Material,
            Fault::SourceZero,
            Fault::Group,
            Fault::AuthZero,
        ] {
            let mut g = grammar();
            attempt(&mut g, format);
            assert!(g.proven());
            g.arm(fault).unwrap();
            attempt(&mut g, format);
            assert!(g.proven());
        }
    }
}
#[test]
fn duplicate_actual_rows_and_uncorrelated_portal_are_rejected() {
    let mut g = grammar();
    execute(&mut g, Kind::Manager, false, 1);
    let row = row(Kind::Manager, false, 1);
    assert!(g.download(b'D', &row).is_ok());
    assert!(g.download(b'D', &row).is_err());
    let mut g = grammar();
    let mut e = string("missing");
    e.extend(0u32.to_be_bytes());
    assert!(g.upload(b'E', &e).is_err());
}
#[test]
fn absent_final_source_or_original_auth_cannot_qualify() {
    let mut g = grammar();
    finish(&mut g, Kind::Manager, false, 1);
    finish(&mut g, Kind::Auth, false, 1);
    assert!(!g.proven());
    finish(&mut g, Kind::Manager, true, 1);
    assert!(!g.proven());
    assert!(g.upload(b'Q', b"COMMIT\0").is_ok());
    assert!(g.download(b'C', b"COMMIT\0").is_ok());
    assert!(g.download(b'Z', b"I").is_ok());
    assert!(!g.proven());
}
#[test]
fn changed_original_namespace_null_final_group_and_extra_source_are_rejected() {
    let mut g = grammar();
    finish(&mut g, Kind::Manager, false, 1);
    assert!(g.upload(b'B', &bind(Kind::Manager, false, 1)).is_err());
    let mut wrong = bind(Kind::Auth, false, 1);
    let needle = ids()[1].as_bytes().to_vec();
    let at = wrong.windows(16).position(|b| b == needle).unwrap();
    wrong[at] ^= 0x80;
    assert!(g.upload(b'B', &wrong).is_err());
    let mut g = grammar();
    attempt(&mut g, 1);
    assert!(g.upload(b'B', &bind(Kind::Manager, true, 1)).is_err());
}
#[test]
fn ambiguous_sql_trailing_cells_and_jsonb_version_are_rejected() {
    assert!(codec::kind(format!("{}; SELECT 1", codec::MANAGER).as_bytes()).is_err());
    let mut raw = row(Kind::Manager, false, 1);
    raw.push(0);
    assert!(codec::row(&raw).is_err());
    let mut cells = codec::row(&row(Kind::Manager, false, 1)).unwrap();
    cells[19].as_mut().unwrap()[0] = 2;
    let columns = codec::MANAGER_SCHEMA
        .iter()
        .map(|(name, oid)| codec::Column {
            name: (*name).to_owned(),
            oid: *oid,
        })
        .collect::<Vec<_>>();
    assert!(codec::values(&cells, &columns, &[1; 20]).is_err());
    assert!(codec::row(&[0, 65]).is_err());
}
#[test]
fn altered_xid_pid_material_or_schema_are_detected() {
    for index in [17, 18, 19] {
        let mut g = grammar();
        finish(&mut g, Kind::Manager, false, 1);
        finish(&mut g, Kind::Auth, false, 1);
        execute(&mut g, Kind::Manager, true, 1);
        let mut cells = codec::row(&row(Kind::Manager, true, 1)).unwrap();
        cells[index] = Some(if index == 19 {
            vec![1, b'{', b'}']
        } else if index == 18 {
            12346i32.to_be_bytes().to_vec()
        } else {
            b"101".to_vec()
        });
        assert!(
            g.download(b'D', &codec::write_cells(&cells)).is_err(),
            "corrupted complete actual source escaped correlation"
        );
    }
    let mut g = Grammar::new(ids(), group(), 1, None);
    let mut p = string("m");
    p.extend(string(codec::MANAGER));
    p.extend(5u16.to_be_bytes());
    for _ in 0..5 {
        p.extend(25u32.to_be_bytes());
    }
    assert!(g.upload(b'P', &p).is_err());
}

#[test]
fn exact_manager_projection_accepts_whitespace_only_and_pins_limit_and_xid_text() {
    let spaced = codec::MANAGER.replace(' ', "\n\t  ");
    assert!(matches!(codec::kind(spaced.as_bytes()), Ok(Kind::Manager)));
    for wrong in [
        codec::MANAGER.replace(" LIMIT 2", ""),
        codec::MANAGER.replace("LIMIT 2", "LIMIT 1"),
        codec::MANAGER.replace("source_xid::text AS source_xid", "source_xid"),
        codec::MANAGER.replace("source_material", "source_material->'request'"),
        codec::MANAGER.to_ascii_uppercase(),
        "SELECT * FROM public.identity_company_information_manager_current_v1($1,$2,$3,$4,$5) LIMIT 2".to_owned(),
    ] {
        assert!(codec::kind(wrong.as_bytes()).is_err(), "unreviewed target SQL accepted");
    }
    let columns = codec::MANAGER_SCHEMA
        .iter()
        .map(|(name, oid)| codec::Column {
            name: (*name).to_owned(),
            oid: *oid,
        })
        .collect::<Vec<_>>();
    for format in [0, 1] {
        let cells = codec::row(&row(Kind::Manager, false, format)).unwrap();
        assert!(codec::values(&cells, &columns, &vec![format; 20]).unwrap()["source_xid"] == 100);
        for wrong in [
            b"0".to_vec(),
            b"-1".to_vec(),
            b"+100".to_vec(),
            b"1e2".to_vec(),
            u64::MAX.to_be_bytes().to_vec(),
            b"18446744073709551616".to_vec(),
        ] {
            let mut cells = cells.clone();
            cells[17] = Some(wrong);
            assert!(codec::values(&cells, &columns, &vec![format; 20]).is_err());
        }
    }
}

#[test]
fn group_source_precedes_initial_auth_and_final_source_precedes_final_auth() {
    let mut g = grammar();
    assert!(g.upload(b'B', &bind(Kind::Auth, false, 1)).is_ok());
    assert!(
        g.upload(b'E', &execution()).is_err(),
        "Auth acquired before Group source"
    );
    let mut g = grammar();
    finish(&mut g, Kind::Manager, false, 1);
    assert!(g.upload(b'B', &bind(Kind::Manager, true, 1)).is_ok());
    assert!(
        g.upload(b'E', &execution()).is_err(),
        "final source skipped initial Auth"
    );
    let mut g = grammar();
    finish(&mut g, Kind::Manager, false, 1);
    finish(&mut g, Kind::Auth, false, 1);
    assert!(g.upload(b'B', &bind(Kind::Auth, true, 1)).is_ok());
    assert!(
        g.upload(b'E', &execution()).is_err(),
        "final Auth preceded final source"
    );
    let mut g = grammar();
    finish(&mut g, Kind::Manager, false, 1);
    let mut stale = string("stale");
    stale.extend(&bind(Kind::Auth, false, 1)[2..]);
    assert!(g.upload(b'B', &stale).is_ok());
    let mut e = string("stale");
    e.extend(0u32.to_be_bytes());
    assert!(g.upload(b'E', &e).is_ok());
    assert!(g.upload(b'S', &[]).is_ok());
    assert!(g.download(b'D', &row(Kind::Auth, false, 1)).is_ok());
    assert!(g.download(b'C', b"SELECT 1\0").is_ok());
    assert!(g.download(b'Z', b"T").is_ok());
    finish(&mut g, Kind::Manager, true, 1);
    // A cached statement is reusable, but this retained portal is still Auth1.
    assert!(g.upload(b'E', &e).is_err(), "stale bound portal accepted");
}

#[test]
fn missing_wrong_or_duplicate_commit_acknowledgements_are_rejected() {
    let ready = || {
        let mut g = grammar();
        finish(&mut g, Kind::Manager, false, 1);
        finish(&mut g, Kind::Auth, false, 1);
        finish(&mut g, Kind::Manager, true, 1);
        finish(&mut g, Kind::Auth, true, 1);
        assert!(g.upload(b'Q', b"COMMIT\0").is_ok());
        g
    };
    let mut g = ready();
    assert!(
        g.download(b'Z', b"I").is_err(),
        "missing COMMIT acknowledgement accepted"
    );
    let mut g = ready();
    assert!(
        g.download(b'C', b"ROLLBACK\0").is_err(),
        "wrong COMMIT acknowledgement accepted"
    );
    let mut g = ready();
    assert!(g.download(b'C', b"COMMIT\0").is_ok());
    assert!(
        g.download(b'C', b"COMMIT\0").is_err(),
        "duplicate COMMIT acknowledgement accepted"
    );
}

#[test]
fn epoch_one_receipt_null_is_exactly_bound_and_other_nulls_are_rejected() {
    let mut g = grammar();
    attempt(&mut g, 1);
    assert!(g.proven(), "real epoch1 NULL receipt was rejected");
    for index in 0..20 {
        if index == 6 {
            continue;
        }
        let mut g = grammar();
        execute(&mut g, Kind::Manager, false, 1);
        let mut cells = codec::row(&row(Kind::Manager, false, 1)).unwrap();
        cells[index] = None;
        assert!(
            g.download(b'D', &codec::write_cells(&cells)).is_err(),
            "nullable acceptance broadened"
        );
    }
    let mut g = grammar();
    execute(&mut g, Kind::Manager, false, 1);
    let mut cells = codec::row(&row(Kind::Manager, false, 1)).unwrap();
    cells[6] = Some(codec::uuid_bytes(Uuid::from_u128(6), 1));
    assert!(
        g.download(b'D', &codec::write_cells(&cells)).is_err(),
        "epoch1 receipt changed"
    );
    for wrong in [
        json!({"org_id": ids()[2], "epoch": 2, "current_policy_receipt_id": null}),
        json!({"org_id": ids()[2], "epoch": 1}),
        json!({"org_id": ids()[2], "epoch": 1, "current_policy_receipt_id": Uuid::from_u128(6)}),
    ] {
        let mut g = grammar();
        execute(&mut g, Kind::Manager, false, 1);
        let mut cells = codec::row(&row(Kind::Manager, false, 1)).unwrap();
        let mut material: serde_json::Value =
            serde_json::from_slice(&cells[19].as_ref().unwrap()[1..]).unwrap();
        material["company_head"] = wrong;
        let mut encoded = vec![1];
        encoded.extend(serde_json::to_vec(&material).unwrap());
        cells[19] = Some(encoded);
        assert!(
            g.download(b'D', &codec::write_cells(&cells)).is_err(),
            "source head escaped authority binding"
        );
    }
}
