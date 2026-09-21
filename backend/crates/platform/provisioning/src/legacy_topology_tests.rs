// Fixed independent vectors approved before the product codec existed.
const GOLDEN: [(&str, &str); 9] = [
    (
        "636f6e736f6c652e706c6174666f726d2e746f706f6c6f6779000001000000000000000000000000000000010000000000000000000000000000006501000000000461636d6500000006ed9a8cec82ac",
        "a30063cfdbf6c7eff5ef19c7c8820d59657b8e9ecf2fff19eb38a7d055d78a44",
    ),
    (
        "636f6e736f6c652e706c6174666f726d2e746f706f6c6f6779000001000000000000000000000000000000010000000000000000000000000000006602000000000567726f757000000006eab7b8eba3b9",
        "20c96b8e933e7dc63a016dc5aafaf056b11ad3bf8864546ef9e5e6c7de7fadc8",
    ),
    (
        "636f6e736f6c652e706c6174666f726d2e746f706f6c6f6779000001000000000000000000000000000000010000000000000000000000000000006703010000000000000000000000000000000a0000000000000000000000000000001400000000000000010000000000000000000000000000000a00010000000aec838820ed9a8cec82ac0102",
        "bcc68acb4c6d25e4deac02b0816dac7110b613bd1dc9a4fe8bbeeb9edb5fee07",
    ),
    (
        "636f6e736f6c652e706c6174666f726d2e746f706f6c6f6779000001000000000000000000000000000000010000000000000000000000000000006804020000000000000000000000000000000a0000000000000000000000000000001400000000000000010000000000000000000000000000000b0000000000000000000000000000001500000000000000020000000000000000000000000000000b0000000000000000000000000000001e",
        "7c06a4ef74ee235bdc92743baea4450992ed93ea7d624a9940dbf2ba98422889",
    ),
    (
        "636f6e736f6c652e706c6174666f726d2e746f706f6c6f6779000001000000000000000000000000000000010000000000000000000000000000006905010000000000000000000000000000000a0000000000000000000000000000001400000000000000010000000000000000000000000000000a0000000000000000000000000000001e",
        "071360a293ee9785d23072233468ea52e7dae155cf8547e7aa22eafc8f4b8e6d",
    ),
    (
        "636f6e736f6c652e706c6174666f726d2e746f706f6c6f6779000001000000000000000000000000000000010000000000000000000000000000006a06010000000000000000000000000000000a0000000000000000000000000000001400000000000000010000000000000000000000000000000a0000000000000000000000000000001e00000009ed998deab8b8eb8f99010000000b30313031323334353637380306060201",
        "048ce7a3ffd37a2dce96f6709dc72e89d013da38a657a987356ab919b0fbcfc6",
    ),
    (
        "636f6e736f6c652e706c6174666f726d2e746f706f6c6f6779000001000000000000000000000000000000010000000000000000000000000000006b07020000000000000000000000000000000a0000000000000000000000000000001400000000000000010000000000000000000000000000000b0000000000000000000000000000001500000000000000020000000000000000000000000000000a0000000000000000000000000000002802",
        "9798f986e041fde0d76baff052e5cf4088499dac2694791d4f35c8d05ae71e0d",
    ),
    (
        "636f6e736f6c652e706c6174666f726d2e746f706f6c6f6779000001000000000000000000000000000000010000000000000000000000000000006c08010000000000000000000000000000000a0000000000000000000000000000001400000000000000010000000000000000000000000000001e",
        "0b004a95dd032ba9dfe76b1a5e990b96a1e509a5bc3d024b2ac55db364f79c16",
    ),
    (
        "636f6e736f6c652e706c6174666f726d2e746f706f6c6f6779000001000000000000000000000000000000010000000000000000000000000000006d09010000000000000000000000000000000a0000000000000000000000000000001400000000000000010000000000000000000000000000001e03",
        "796ee2ea3244691e087c920d64001ee31365d0e0b09aa9f6b66767f0517ac865",
    ),
];

use super::*;
use sha2::{Digest, Sha256};

fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}
fn head(n: u128) -> ExpectedGroupHead {
    ExpectedGroupHead::new(id(n), id(n + 10), (n - 9) as u64).unwrap()
}
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn cases() -> Vec<(Vec<ExpectedGroupHead>, LegacyTopologyIntent)> {
    use LegacyTopologyIntent::*;
    vec![
        (
            vec![],
            CreateCompany {
                slug: "acme".into(),
                name: "회사".into(),
            },
        ),
        (
            vec![],
            CreateGroup {
                slug: "group".into(),
                name: "그룹".into(),
            },
        ),
        (
            vec![head(10)],
            UpdateGroup {
                group: id(10),
                slug: None,
                name: Some("새 회사".into()),
                status: Some(CompanyStatus::Suspended),
            },
        ),
        (
            vec![head(10), head(11)],
            AssignCompany {
                group: id(11),
                company: id(30),
            },
        ),
        (
            vec![head(10)],
            RemoveCompanyFromGroup {
                group: id(10),
                company: id(30),
            },
        ),
        (
            vec![head(10)],
            CreateLegacyGroupAccount {
                group: id(10),
                company: id(30),
                display_name: "홍길동".into(),
                phone: Some("01012345678".into()),
                tenant_roles: vec![TenantRole::Member, TenantRole::Member, TenantRole::Admin],
                group_role: GroupRole::Admin,
            },
        ),
        (
            vec![head(10), head(11)],
            RevokeLegacyGroupRole {
                group: id(10),
                user: id(40),
                group_role: GroupRole::Viewer,
            },
        ),
        (vec![head(10)], RemoveEmptyCompany { company: id(30) }),
        (
            vec![head(10)],
            SetCompanyStatus {
                company: id(30),
                status: CompanyStatus::Archived,
            },
        ),
    ]
}

#[test]
fn topology_nine_frozen_vectors_preserve_intent_identity_and_exact_bytes() {
    for (index, (heads, intent)) in cases().into_iter().enumerate() {
        let command = LegacyTopologyCommandV1::new(
            UserId::from_uuid(id(1)),
            id(101 + index as u128),
            heads.clone(),
            intent.clone(),
        )
        .unwrap();
        let bytes = unhex(GOLDEN[index].0);
        assert_eq!(command.encode(), bytes);
        assert_eq!(command.digest().as_slice(), unhex(GOLDEN[index].1));
        assert_eq!(
            command.digest().as_slice(),
            Sha256::digest(&bytes).as_slice()
        );
        let decoded = LegacyTopologyCommandV1::decode(&bytes).unwrap();
        assert_eq!(decoded.actor(), UserId::from_uuid(id(1)));
        assert_eq!(decoded.command_id(), id(101 + index as u128));
        assert_eq!(decoded.expected_groups(), heads);
        assert_eq!(decoded.intent(), &intent);
        assert_eq!(decoded.encode(), bytes);
        assert_eq!(decoded.digest(), command.digest());
    }
}

#[test]
fn topology_rejects_truncated_trailing_oversized_unknown_and_nil_frames() {
    for (hex, _) in GOLDEN {
        let bytes = unhex(hex);
        for end in 0..bytes.len() {
            assert!(
                LegacyTopologyCommandV1::decode(&bytes[..end]).is_err(),
                "accepted truncated frame at {end}"
            );
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(LegacyTopologyCommandV1::decode(&trailing).is_err());
        for offset in 0..28 {
            let mut broken = bytes.clone();
            broken[offset] ^= 0xff;
            assert!(LegacyTopologyCommandV1::decode(&broken).is_err());
        }
        for offset in [28, 44] {
            let mut broken = bytes.clone();
            broken[offset..offset + 16].fill(0);
            assert!(LegacyTopologyCommandV1::decode(&broken).is_err());
        }
        for tag in [0, 10, 255] {
            let mut broken = bytes.clone();
            broken[60] = tag;
            assert!(LegacyTopologyCommandV1::decode(&broken).is_err());
        }
        let mut broken = bytes.clone();
        broken[61] = 3;
        assert!(LegacyTopologyCommandV1::decode(&broken).is_err());
    }
    assert!(LegacyTopologyCommandV1::decode(&vec![0; 4097]).is_err());
}

#[test]
fn topology_normalizes_only_new_text_and_never_repairs_stored_bytes() {
    use LegacyTopologyIntent::*;
    let command = LegacyTopologyCommandV1::new(
        UserId::from_uuid(id(1)),
        id(101),
        vec![],
        CreateCompany {
            slug: "  acme\u{2003}".into(),
            name: "\t회사\n".into(),
        },
    )
    .unwrap();
    assert_eq!(command.encode(), unhex(GOLDEN[0].0));
    let update = LegacyTopologyCommandV1::new(
        UserId::from_uuid(id(1)),
        id(103),
        vec![head(10)],
        UpdateGroup {
            group: id(10),
            slug: Some("\u{2003}".into()),
            name: Some(" 새 회사 ".into()),
            status: Some(CompanyStatus::Suspended),
        },
    )
    .unwrap();
    assert_eq!(update.encode(), unhex(GOLDEN[2].0));
    let create = |name: &str| {
        LegacyTopologyCommandV1::new(
            UserId::from_uuid(id(1)),
            id(101),
            vec![],
            CreateCompany {
                slug: "acme".into(),
                name: name.into(),
            },
        )
    };
    for name in ["", " \t", "가\0나", "가\n나", "가\u{85}나"] {
        assert!(create(name).is_err());
    }
    assert!(create(&"가".repeat(85)).is_ok());
    assert!(create(&"가".repeat(86)).is_err());
    let mut bytes = unhex(GOLDEN[0].0);
    // A noncanonical stored slug must fail; decoding must not trim it.
    bytes[66] = b' ';
    assert!(LegacyTopologyCommandV1::decode(&bytes).is_err());
    let mut bytes = unhex(GOLDEN[0].0);
    bytes[74..80].copy_from_slice(b"      ");
    assert!(LegacyTopologyCommandV1::decode(&bytes).is_err());
    for length in [0u32, 41, u32::MAX] {
        let mut bytes = unhex(GOLDEN[0].0);
        bytes[62..66].copy_from_slice(&length.to_be_bytes());
        assert!(LegacyTopologyCommandV1::decode(&bytes).is_err());
    }
    let mut bytes = unhex(GOLDEN[0].0);
    bytes[74] = 0xff;
    assert!(LegacyTopologyCommandV1::decode(&bytes).is_err());
}

#[test]
fn topology_heads_are_bounded_sorted_distinct_and_bound_to_requested_group() {
    use LegacyTopologyIntent::*;
    let command = |heads, intent| {
        LegacyTopologyCommandV1::new(UserId::from_uuid(id(1)), id(101), heads, intent)
    };
    for (group, incarnation, revision) in [
        (Uuid::nil(), id(20), 1),
        (id(10), Uuid::nil(), 1),
        (id(10), id(20), 0),
        (id(10), id(20), i64::MAX as u64 + 1),
    ] {
        assert!(ExpectedGroupHead::new(group, incarnation, revision).is_err());
    }
    assert!(ExpectedGroupHead::new(id(10), id(20), i64::MAX as u64).is_ok());
    for heads in [vec![], vec![head(11)], vec![head(10), head(11)]] {
        assert!(
            command(
                heads,
                RemoveCompanyFromGroup {
                    group: id(10),
                    company: id(30)
                }
            )
            .is_err()
        );
    }
    for heads in [
        vec![head(11), head(10)],
        vec![head(10), head(10)],
        vec![head(10), head(11), head(12)],
        vec![head(11)],
    ] {
        assert!(
            command(
                heads,
                AssignCompany {
                    group: id(10),
                    company: id(30)
                }
            )
            .is_err()
        );
    }
    assert!(
        command(
            vec![head(10)],
            AssignCompany {
                group: id(10),
                company: id(30)
            }
        )
        .is_ok()
    );
    assert!(
        command(
            vec![head(10)],
            RevokeLegacyGroupRole {
                group: id(10),
                user: id(40),
                group_role: GroupRole::Finance
            }
        )
        .is_ok()
    );
    for heads in [vec![], vec![head(10), head(11)]] {
        assert!(command(heads.clone(), RemoveEmptyCompany { company: id(30) }).is_err());
        assert!(
            command(
                heads,
                SetCompanyStatus {
                    company: id(30),
                    status: CompanyStatus::Active
                }
            )
            .is_err()
        );
    }
    for index in [3, 6] {
        let mut bytes = unhex(GOLDEN[index].0);
        let first = bytes[62..102].to_vec();
        let second = bytes[102..142].to_vec();
        bytes[62..102].copy_from_slice(&second);
        bytes[102..142].copy_from_slice(&first);
        assert!(LegacyTopologyCommandV1::decode(&bytes).is_err());
    }
    for offset in [62, 78, 102] {
        let mut bytes = unhex(GOLDEN[4].0);
        bytes[offset..offset + 16].fill(0);
        assert!(LegacyTopologyCommandV1::decode(&bytes).is_err());
    }
    for revision in [0u64, i64::MAX as u64 + 1] {
        let mut bytes = unhex(GOLDEN[4].0);
        bytes[94..102].copy_from_slice(&revision.to_be_bytes());
        assert!(LegacyTopologyCommandV1::decode(&bytes).is_err());
    }
}

#[test]
fn topology_closed_status_role_tags_optional_tags_and_utf8_bounds() {
    use LegacyTopologyIntent::*;
    let command = |intent| {
        LegacyTopologyCommandV1::new(UserId::from_uuid(id(1)), id(101), vec![head(10)], intent)
    };
    for status in [
        CompanyStatus::Active,
        CompanyStatus::Suspended,
        CompanyStatus::Archived,
    ] {
        let c = command(SetCompanyStatus {
            company: id(30),
            status,
        })
        .unwrap();
        assert_eq!(
            LegacyTopologyCommandV1::decode(c.encode())
                .unwrap()
                .intent(),
            c.intent()
        );
    }
    let roles = vec![
        TenantRole::SuperAdmin,
        TenantRole::Admin,
        TenantRole::Mechanic,
        TenantRole::Receptionist,
        TenantRole::Executive,
        TenantRole::Member,
    ];
    for group_role in [GroupRole::Admin, GroupRole::Viewer, GroupRole::Finance] {
        let c = command(CreateLegacyGroupAccount {
            group: id(10),
            company: id(30),
            display_name: "name".into(),
            phone: Some(" \t".into()),
            tenant_roles: roles.clone(),
            group_role,
        })
        .unwrap();
        assert_eq!(
            LegacyTopologyCommandV1::decode(c.encode())
                .unwrap()
                .intent(),
            c.intent()
        );
        assert!(matches!(
            c.intent(),
            CreateLegacyGroupAccount { phone: None, .. }
        ));
    }
    for count in [0, 64, 65] {
        let result = command(CreateLegacyGroupAccount {
            group: id(10),
            company: id(30),
            display_name: "name".into(),
            phone: Some("가".repeat(21)),
            tenant_roles: vec![TenantRole::Member; count],
            group_role: GroupRole::Admin,
        });
        assert_eq!(result.is_ok(), count == 64);
    }
    for phone in ["가".repeat(22), "a\nb".into()] {
        assert!(
            command(CreateLegacyGroupAccount {
                group: id(10),
                company: id(30),
                display_name: "name".into(),
                phone: Some(phone),
                tenant_roles: vec![TenantRole::Member],
                group_role: GroupRole::Admin
            })
            .is_err()
        );
    }
    assert!(
        command(UpdateGroup {
            group: id(10),
            slug: None,
            name: Some(" ".into()),
            status: None
        })
        .is_err()
    );
    for (index, offset, tag) in [
        (2, 118, 2),
        (2, 119, 2),
        (2, 134, 2),
        (2, 135, 4),
        (5, 147, 2),
        (5, 163, 0),
        (5, 163, 65),
        (5, 164, 7),
        (5, 167, 4),
        (6, 174, 0),
        (8, 118, 4),
    ] {
        let mut bytes = unhex(GOLDEN[index].0);
        bytes[offset] = tag;
        assert!(
            LegacyTopologyCommandV1::decode(&bytes).is_err(),
            "invalid kind {index} offset {offset} tag {tag}"
        );
    }
}
