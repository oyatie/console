macro_rules! assert_admin_source {
    ($source:expr, $origin:expr) => {{
        let s = &$source;
        assert_eq!(s.source_tag, 1);
        assert_eq!(*s.origin_account_id.as_uuid(), uuid($origin));
        assert_eq!(s.origin_command_id, uuid(11));
        assert_eq!(s.origin_receipt_id, uuid(12));
        assert_eq!(*s.administrator_b.as_uuid(), uuid(1));
        assert_eq!(s.group_id, uuid(13));
        assert_eq!(s.group_revision, 2);
        assert_eq!(s.group_incarnation, uuid(14));
        assert_eq!(s.membership_id, uuid(15));
        assert_eq!(s.membership_revision, 3);
        assert_eq!(s.membership_incarnation, uuid(16));
        assert_eq!(s.context_generation, 4);
        assert_eq!(s.root_assignment_id, uuid(17));
        assert_eq!(s.root_assignment_revision, 1);
        assert_eq!(s.root_role_id, uuid(18));
        assert_eq!(s.root_role_revision, 1);
        for (action, id, object) in [(&s.assign_identity, 19, 20), (&s.read_policy, 21, 22)] {
            assert_eq!(*action.org_id().as_uuid(), uuid(2));
            assert_eq!(action.action_type_id(), uuid(id));
            assert_eq!(action.object_type_id(), uuid(object));
            assert_eq!(action.registration_revision(), 1);
            assert_eq!(
                action.manifest_digest(),
                &[
                    0x0d, 0x3d, 0x0c, 0x3b, 0xc0, 0x35, 0x7c, 0x03, 0x94, 0xb0, 0x24, 0x00, 0x29,
                    0x5f, 0x77, 0x23, 0x11, 0x78, 0xcd, 0x5d, 0xc2, 0x2a, 0x66, 0x8a, 0x90, 0x88,
                    0x0f, 0xc9, 0x2a, 0x08, 0x99, 0x35,
                ]
            );
        }
    }};
}
macro_rules! assert_snapshot {
    ($snapshot:expr, $base:expr, $account:expr, $valid:expr) => {{
        let s = &$snapshot;
        let n: u128 = $base;
        assert_eq!(*s.company.as_uuid(), uuid(2));
        assert_eq!(*s.account.as_uuid(), uuid($account));
        assert_eq!(s.human_anchor, uuid(n));
        let q = &s.qualification;
        assert_eq!(q.person_id, uuid(n));
        assert_eq!(q.person_revision, 5);
        assert_eq!(q.person_receipt_id, uuid(n + 1));
        assert_eq!(q.assertion_id, uuid(n + 2));
        assert_eq!(q.assertion_revision, 6);
        assert_eq!(q.assertion_receipt_id, uuid(n + 3));
        assert_eq!(q.current_pointer_revision, 7);
        assert_eq!(q.observation_id, uuid(n + 4));
        assert_eq!(q.observation_revision, 8);
        assert_eq!(q.observation_digest, [(n + 5) as u8; 32]);
        assert_eq!(q.target_confirmation_receipt_id, uuid(n + 6));
        assert_eq!(q.trust_adoption_id, uuid(n + 7));
        assert_eq!(q.trust_adoption_revision, 9);
        assert_eq!(q.trust_adoption_receipt_id, uuid(n + 8));
        assert_eq!(q.conflict_witness_id, uuid(n + 9));
        assert_eq!(q.conflict_witness_revision, 10);
        assert_eq!(q.conflict_witness_digest, [(n + 10) as u8; 32]);
        assert_eq!(s.person_content_digest, [(n + 11) as u8; 32]);
        assert_eq!(s.assertion_content_digest, [(n + 12) as u8; 32]);
        assert_eq!(s.current_pointer_content_digest, [(n + 13) as u8; 32]);
        assert_eq!(s.conflict_receipt_id, uuid(n + 14));
        for (pin, id, revision, digest) in [
            (&s.evidence_availability, n + 15, 11, n + 17),
            (&s.source_impact, n + 18, 12, n + 20),
        ] {
            assert_eq!(pin.source_id, uuid(id));
            assert_eq!(pin.revision, revision);
            assert_eq!(pin.receipt_id, uuid(id + 1));
            assert_eq!(pin.content_digest, [digest as u8; 32]);
        }
        assert_eq!(s.human_anchor, q.person_id);
        if let Some(instant) = s.individual_valid_until {
            assert_eq!(instant.offset(), time::UtcOffset::UTC);
        }
        assert_eq!(
            s.individual_valid_until.map(|x| x.unix_timestamp_nanos()),
            if $valid { Some(EXPIRY * 1_000) } else { None }
        );
    }};
}
macro_rules! assert_literal {
    ($literal:expr) => {{
        let v = $literal;
        let (actor, command) = decode_native_hr_policy_command_v1(PROTOCOL, 1, &v.bytes())
            .unwrap_or_else(|e| panic!("{}: {e}", v.name));
        assert_eq!(*actor.as_uuid(), uuid(1));
        assert_eq!(*command.company.as_uuid(), uuid(2));
        assert_eq!(command.command_id, uuid(3));
        assert_eq!(command.expected_company_epoch, v.epoch);
        assert_eq!(
            command.current_policy_receipt_id,
            if v.epoch == 1 { None } else { Some(uuid(4)) }
        );
        assert_admin_source!(command.admin_source, v.origin_account);
        assert_eq!(actor, command.admin_source.administrator_b);
        assert_eq!(
            command.expected_admin_assignment,
            if v.operation == 1 {
                None
            } else {
                Some((uuid(5), 13))
            }
        );
        assert_eq!(
            command.expected_target_assignment,
            if v.target { Some((uuid(6), 14)) } else { None }
        );
        assert_eq!(command.operation, v.operation);
        assert_eq!(
            command.role_kind,
            if v.operation == 1 { None } else { Some(v.role) }
        );
        assert_eq!(
            command.recipient_account_id.map(|x| *x.as_uuid()),
            if v.operation == 1 {
                None
            } else {
                Some(uuid(v.recipient))
            }
        );
        assert_eq!(
            command.expires_at.map(|x| x.unix_timestamp_nanos()),
            if v.operation == 2 {
                Some(v.grant_expiry * 1_000)
            } else {
                None
            }
        );
        if let Some(instant) = command.expires_at {
            assert_eq!(instant.offset(), time::UtcOffset::UTC);
        }
        assert_eq!(
            command.recipient_source_digest,
            if v.operation == 2 {
                Some([0xa0; 32])
            } else {
                None
            }
        );
        if v.operation == 2 && v.recipient == 7 {
            let (b, c) = command
                .qualification_pair
                .as_ref()
                .expect("complete B/C pair");
            assert_snapshot!(b, 32, 1, v.bvalid);
            assert_snapshot!(c, 80, 7, v.cvalid);
            assert_eq!(b.company, command.company);
            assert_eq!(c.company, command.company);
            assert_eq!(b.account, command.admin_source.administrator_b);
            assert_eq!(Some(c.account), command.recipient_account_id);
            assert_ne!(b.account, c.account);
            assert_ne!(b.human_anchor, c.human_anchor);
        } else {
            assert!(command.qualification_pair.is_none(), "{}", v.name);
        }
    }};
}
