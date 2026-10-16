//! Explicit SQL composite projection and immutable command/receipt verification.
use super::*;
use sha2::{Digest, Sha256};
use sqlx::{Row, ValueRef, postgres::PgRow};

pub(super) struct Opened {
    pub inserted: bool,
    pub effect_digest: [u8; 32],
    pub accepted: AcceptedDirectoryRequestV1,
    pub terminal: Option<DirectoryTerminalV1>,
}
pub(super) enum Opening {
    Prepare,
    Transition(&'static str),
}
macro_rules! result_query { ($tail:literal) => { concat!("SELECT ", "n.inserted,(n.accepted_input).org_id AS i_org_id,(n.accepted_input).command_id AS i_command_id,(n.accepted_input).actor_account_id AS i_actor_account_id,(n.accepted_input).employee_id AS i_employee_id,(n.accepted_input).legal_name AS i_legal_name,(n.accepted_input).employee_number AS i_employee_number,(n.accepted_input).expected_company_epoch AS i_expected_company_epoch,(n.accepted_input).expected_object_type_id AS i_expected_object_type_id,(n.accepted_input).expected_action_type_id AS i_expected_action_type_id,(n.accepted_input).expected_action_revision AS i_expected_action_revision,(n.accepted_input).expected_schema_revision AS i_expected_schema_revision,(n.accepted_input).legal_name_property_id AS i_legal_name_property_id,(n.accepted_input).employee_number_property_id AS i_employee_number_property_id,(n.accepted_input).manifest_digest AS i_manifest_digest,(n.accepted_input).codec_version AS i_codec_version,(n.accepted_input).input_bytes AS i_input_bytes,(n.accepted_input).input_digest AS i_input_digest,(n.accepted_input).intake_receipt_id AS i_intake_receipt_id,(n.accepted_input).accepted_at AS i_accepted_at,(n.accepted_input).execution_not_after AS i_execution_not_after,(n.accepted_input).accepting_session_id AS i_accepting_session_id,(n.accepted_input).acceptance_xid::text AS i_acceptance_xid,(n.accepted_input).acceptance_backend_pid AS i_acceptance_backend_pid,(n.accepted_input).accepting_assignment_id AS i_accepting_assignment_id,(n.accepted_input).accepting_assignment_revision AS i_accepting_assignment_revision,(n.terminal).org_id AS t_org_id,(n.terminal).command_id AS t_command_id,(n.terminal).actor_account_id AS t_actor_account_id,(n.terminal).intake_receipt_id AS t_intake_receipt_id,(n.terminal).input_digest AS t_input_digest,(n.terminal).outcome AS t_outcome,(n.terminal).result_code AS t_result_code,(n.terminal).transition_kind AS t_transition_kind,(n.terminal).execution_session_id AS t_execution_session_id,(n.terminal).effect_xid::text AS t_effect_xid,(n.terminal).effect_backend_pid AS t_effect_backend_pid,(n.terminal).source_company_epoch AS t_source_company_epoch,(n.terminal).source_policy_receipt_id AS t_source_policy_receipt_id,(n.terminal).source_assignment_id AS t_source_assignment_id,(n.terminal).source_assignment_revision AS t_source_assignment_revision,(n.terminal).source_valid_from AS t_source_valid_from,(n.terminal).source_valid_until AS t_source_valid_until,(n.terminal).terminal_at AS t_terminal_at,(n.terminal).employee_id AS t_employee_id,(n.terminal).person_id AS t_person_id,(n.terminal).canonical_command_id AS t_canonical_command_id", $tail) }; }
pub(super) const PREPARE_SQL: &str = result_query!(
    " FROM public.native_people_prepare_v1($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) n LIMIT 2"
);
pub(super) const TERMINAL_SQL: &str =
    result_query!(" FROM public.native_people_terminal_open_v1($1,$2,$3,$4,$5) n LIMIT 2");
const TERMINAL_COLUMNS: &[&str] = &[
    "t_org_id",
    "t_command_id",
    "t_actor_account_id",
    "t_intake_receipt_id",
    "t_input_digest",
    "t_outcome",
    "t_result_code",
    "t_transition_kind",
    "t_execution_session_id",
    "t_effect_xid",
    "t_effect_backend_pid",
    "t_source_company_epoch",
    "t_source_policy_receipt_id",
    "t_source_assignment_id",
    "t_source_assignment_revision",
    "t_source_valid_from",
    "t_source_valid_until",
    "t_terminal_at",
    "t_employee_id",
    "t_person_id",
    "t_canonical_command_id",
];
pub(super) fn decode(
    row: &PgRow,
    source: &source::Source,
    company: OrgId,
    command: Uuid,
    opening: Opening,
    now: OffsetDateTime,
) -> Result<Opened, Error> {
    macro_rules! get {
        ($n:literal,$t:ty) => {
            row.try_get::<$t, _>($n).map_err(sql_error)?
        };
    }
    let inserted = get!("inserted", bool);
    let bytes = get!("i_input_bytes", Vec<u8>);
    if get!("i_codec_version", i16) != DIRECTORY_CODEC_VERSION {
        return Err(Error::Unavailable);
    }
    let (actor, input) =
        NativeDirectoryCommandV1::decode(&bytes).map_err(|_| Error::Unavailable)?;
    let expected = input.expected();
    let digest = Sha256::digest(&bytes).to_vec();
    let mut effect = Sha256::new();
    effect.update(b"console.people.directory-effect\0\0\x01");
    effect.update(&bytes);
    let effect_digest: [u8; 32] = effect.finalize().into();
    if actor != source.binding.account
        || input.company() != company
        || input.command_id() != command
        || get!("i_actor_account_id", Uuid) != *actor.as_uuid()
        || get!("i_org_id", Uuid) != *company.as_uuid()
        || get!("i_command_id", Uuid) != command
        || get!("i_employee_id", Uuid) != input.employee_id()
        || get!("i_legal_name", String) != input.input().legal_name()
        || get!("i_employee_number", String) != input.input().employee_number()
        || positive(get!("i_expected_company_epoch", i64))? != expected.company_epoch
        || get!("i_expected_object_type_id", Uuid) != expected.object_type_id
        || get!("i_expected_action_type_id", Uuid) != expected.action_type_id
        || positive(get!("i_expected_action_revision", i64))? != expected.action_revision
        || positive(get!("i_expected_schema_revision", i64))? != expected.schema_revision
        || get!("i_legal_name_property_id", Uuid) != expected.legal_name_property_id
        || get!("i_employee_number_property_id", Uuid) != expected.employee_number_property_id
        || get!("i_manifest_digest", Vec<u8>) != DIRECTORY_MANIFEST
        || get!("i_input_digest", Vec<u8>) != digest
    {
        return Err(Error::Unavailable);
    }
    let accepted = AcceptedDirectoryRequestV1::new(
        actor,
        input,
        get!("i_intake_receipt_id", Uuid),
        get!("i_accepted_at", OffsetDateTime),
    )
    .map_err(|_| Error::Unavailable)?;
    if accepted.execution_not_after() != get!("i_execution_not_after", OffsetDateTime)
        || accepted.accepted_at() > now
    {
        return Err(Error::Unavailable);
    }
    let family = nonnil(get!("i_accepting_session_id", Uuid))?;
    let xid = xid_number(&get!("i_acceptance_xid", String))?;
    let pid = get!("i_acceptance_backend_pid", i32);
    nonnil(get!("i_accepting_assignment_id", Uuid))?;
    positive(get!("i_accepting_assignment_revision", i64))?;
    if pid < 1 {
        return Err(Error::Unavailable);
    }
    if inserted
        && matches!(opening, Opening::Prepare)
        && (family != source.binding.session_id
            || xid != source.binding.source_xid
            || pid != source.binding.source_backend_pid
            || get!("i_accepting_assignment_id", Uuid)
                != source.authority.current.source().assignment_id
            || get!("i_accepting_assignment_revision", i64)
                != source.authority.current.source().assignment_revision)
    {
        return Err(Error::Unavailable);
    }
    let all_null = TERMINAL_COLUMNS
        .iter()
        .map(|name| {
            row.try_get_raw(*name)
                .map(|v| v.is_null())
                .map_err(sql_error)
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .all(|v| v);
    if all_null {
        if inserted && !matches!(opening, Opening::Prepare) {
            return Err(Error::Unavailable);
        }
        return Ok(Opened {
            effect_digest,
            inserted,
            accepted,
            terminal: None,
        });
    }
    let terminal = DirectoryTerminalV1::new(
        accepted.clone(),
        DirectoryTerminalOutcomeV1::from_storage(
            &get!("t_outcome", String),
            &get!("t_result_code", String),
        )
        .map_err(|_| Error::Unavailable)?,
        get!("t_terminal_at", OffsetDateTime),
    )
    .map_err(|_| Error::Unavailable)?;
    if get!("t_org_id", Uuid) != *company.as_uuid()
        || get!("t_command_id", Uuid) != command
        || get!("t_actor_account_id", Uuid) != *actor.as_uuid()
        || get!("t_intake_receipt_id", Uuid) != accepted.intake_receipt_id()
        || get!("t_input_digest", Vec<u8>) != digest
        || get!("t_employee_id", Option<Uuid>) != terminal.employee_id()
        || get!("t_person_id", Option<Uuid>) != terminal.person_id()
        || get!("t_canonical_command_id", Option<Uuid>) != terminal.canonical_command_id()
        || terminal.terminal_at() > now
    {
        return Err(Error::Unavailable);
    }
    let transition = get!("t_transition_kind", String);
    let valid = match transition.as_str() {
        "EXECUTE" => terminal.outcome() != DirectoryTerminalOutcomeV1::Cancelled,
        "CANCEL" => matches!(
            terminal.outcome(),
            DirectoryTerminalOutcomeV1::Cancelled | DirectoryTerminalOutcomeV1::Expired
        ),
        "STATUS" => terminal.outcome() == DirectoryTerminalOutcomeV1::Expired,
        _ => false,
    };
    let terminal_family = nonnil(get!("t_execution_session_id", Uuid))?;
    let terminal_xid = xid_number(&get!("t_effect_xid", String))?;
    let terminal_pid = get!("t_effect_backend_pid", i32);
    let epoch = positive(get!("t_source_company_epoch", i64))?;
    let policy = nonnil(get!("t_source_policy_receipt_id", Uuid))?;
    let assignment = nonnil(get!("t_source_assignment_id", Uuid))?;
    let revision = positive(get!("t_source_assignment_revision", i64))?;
    let from = get!("t_source_valid_from", OffsetDateTime);
    let until = get!("t_source_valid_until", OffsetDateTime);
    if !valid
        || terminal_pid < 1
        || from.unix_timestamp_nanos() % 1000 != 0
        || until.unix_timestamp_nanos() % 1000 != 0
        || from >= until
        || until - from > Duration::days(30)
        || terminal.terminal_at() < from
        || terminal.terminal_at() >= until
    {
        return Err(Error::Unavailable);
    }
    if inserted {
        let Opening::Transition(operation) = opening else {
            return Err(Error::Unavailable);
        };
        let s = source.authority.current.source();
        if transition != operation
            || terminal_family != source.binding.session_id
            || terminal_xid != source.binding.source_xid
            || terminal_pid != source.binding.source_backend_pid
            || epoch != s.company_epoch as u64
            || policy != s.current_policy_receipt_id
            || assignment != s.assignment_id
            || revision != s.assignment_revision as u64
            || from != s.assignment_valid_from
            || until != s.assignment_valid_until
        {
            return Err(Error::Unavailable);
        }
    }
    Ok(Opened {
        effect_digest,
        inserted,
        accepted,
        terminal: Some(terminal),
    })
}
