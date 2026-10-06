//! Proposed child of native_workflow_usecases_tests. Actual usecase + Cedar;
//! scripted persistence/finish fixture is not PostgreSQL or authorization custody proof.
use super::*;
use console_identity_application::company_policy::people_business::{
    DirectoryActionV1, NativePeoplePolicyCommandV1,
};

fn people_commands() -> Vec<NativePeoplePolicyCommandV1> {
    let mut commands =
        vec![NativePeoplePolicyCommandV1::install(id(1000), OrgId::from_uuid(id(11)), 1).unwrap()];
    let witness =
        console_identity_application::company_policy::business::PolicyAssignmentExpectationV1 {
            role_revision: 1,
            assignment_id: id(2000),
            assignment_revision: 1,
        };
    for action in [DirectoryActionV1::Read, DirectoryActionV1::Create] {
        commands.push(
            NativePeoplePolicyCommandV1::grant(
                id(1000),
                OrgId::from_uuid(id(11)),
                1,
                action,
                AccountId::from_uuid(id(34)).unwrap(),
                None,
                at() + Duration::days(1),
            )
            .unwrap(),
        );
        commands.push(
            NativePeoplePolicyCommandV1::revoke(
                id(1000),
                OrgId::from_uuid(id(11)),
                1,
                action,
                witness,
            )
            .unwrap(),
        );
    }
    commands
}
fn people_plan(mode: Mode, command: &NativePeoplePolicyCommandV1) -> Plan {
    let mut plan = Plan::new(mode);
    let selected = NativePolicyCommandRef::from_people_command(command);
    plan.resolved_selector = Some(selected);
    plan.returned_selector = Some(selected);
    let mut accepted = accepted();
    accepted.input = command.clone().into();
    plan.status = NativePolicyStatus::AcceptedPending(accepted.clone());
    let mut receipt = terminal();
    receipt.accepted = accepted;
    // A rejection needs no invented installed/granted object; this fixture tests
    // identity and disclosure, not the database transition's correctness.
    receipt.epoch_after = receipt.epoch_before;
    receipt.outcome = NativePolicyOutcome::Rejected(NativePolicyRejection::RevisionConflict);
    plan.terminal = Some(receipt);
    plan
}
fn people_store(plans: Vec<Plan>, selected: NativePolicyCommandRef) -> Store {
    let mut store = Store::new(plans);
    store.expected_selector = selected;
    store
}
fn invoke(
    mode: Mode,
    store: &Store,
    policy: &Policy,
    selected: NativePolicyCommandRef,
    command: &NativePeoplePolicyCommandV1,
) -> Result<(), NativePolicyWorkflowError> {
    match mode {
        Mode::Current => ready(native_policy_current(store, policy, &(), selected)).map(|v| assert_eq!(v.selector, NativePolicyCommandRef::from_people_command(command))),
        Mode::Form => ready(native_policy_form(store, policy, &(), selected)).map(|v| {assert_eq!(v.proof, 77);assert_eq!(v.view.selector, NativePolicyCommandRef::from_people_command(command));}),
        Mode::ValidationForm => ready(native_policy_validation_form(store, policy, &(), selected)).map(|v| {assert_eq!(v.proof, 77);assert_eq!(v.view.selector, NativePolicyCommandRef::from_people_command(command));}),
        Mode::Accept => ready(accept_native_policy_command(store, policy, &(), command, &TraceContext::generate())).map(|_| ()),
        Mode::Execute => ready(execute_native_policy_command(store, policy, &(), selected, &TraceContext::generate())).map(|v| assert_eq!(v.terminal.accepted.input, NativePolicyCommand::People(command.clone()))),
        Mode::Status => ready(native_policy_command_status(store, policy, &(), selected)).map(|v| assert!(matches!(v, NativePolicyStatus::AcceptedPending(a) if a.input == NativePolicyCommand::People(command.clone())))),
    }
}
const MODES: [Mode; 6] = [
    Mode::Current,
    Mode::Form,
    Mode::ValidationForm,
    Mode::Accept,
    Mode::Execute,
    Mode::Status,
];

#[test]
fn people_workflow_uses_people_manifest_at_initial_and_final_decision_in_every_mode() {
    for command in people_commands() {
        let selected = NativePolicyCommandRef::from_people_command(&command);
        assert_eq!(
            selected.manifest_digest(),
            &console_identity_application::company_policy::people_business::MANIFEST
        );
        assert_ne!(
            selected,
            NativePolicyCommandRef::new(
                selected.company(),
                selected.command_id(),
                selected.operation()
            )
            .unwrap()
        );
        for mode in MODES {
            let store = people_store(vec![people_plan(mode, &command)], selected);
            let policy = Policy::new(&store);
            assert_eq!(invoke(mode, &store, &policy, selected, &command), Ok(()));
            assert_eq!(store.events(), history(mode));
            assert_eq!(
                store.history.lock().unwrap().input_bytes,
                if mode == Mode::Accept {
                    vec![command.encode(binding().account)]
                } else {
                    vec![]
                }
            );
            store.drained();
        }
    }
}

#[test]
fn people_workflow_final_denial_withholds_all_provisional_results() {
    let command = people_commands().remove(3); // Create grant, not a read default.
    let selected = NativePolicyCommandRef::from_people_command(&command);
    for mode in MODES {
        let store = people_store(vec![people_plan(mode, &command)], selected);
        let policy = Policy::new(&store);
        *policy.overrides.lock().unwrap() = vec![
            Ok(CompanyPolicyDecision::Allow),
            Ok(CompanyPolicyDecision::Deny),
        ]
        .into();
        assert_eq!(
            invoke(mode, &store, &policy, selected, &command),
            Err(NativePolicyWorkflowError::NotFound)
        );
        assert_eq!(
            store.events(),
            [
                format!("open:{mode:?}"),
                "decision".into(),
                format!("operation:{mode:?}"),
                format!("finish:{mode:?}"),
                "decision".into(),
                format!("release:{mode:?}:false")
            ]
        );
        assert!(policy.overrides.lock().unwrap().is_empty());
        store.drained();
    }
}

#[test]
fn people_workflow_rejects_locked_scope_family_action_and_identity_substitution_before_operation() {
    let commands = people_commands();
    let command = &commands[3];
    let selected = NativePolicyCommandRef::from_people_command(command);
    let wrong = [
        NativePolicyCommandRef::new(
            selected.company(),
            selected.command_id(),
            selected.operation(),
        )
        .unwrap(),
        NativePolicyCommandRef::from_people_command(&commands[1]), // same IDs/op, Read instead of Create
        NativePolicyCommandRef::new_people(
            selected.company(),
            id(9999),
            selected.operation(),
            Some(DirectoryActionV1::Create),
        )
        .unwrap(),
        NativePolicyCommandRef::new_people(
            OrgId::from_uuid(id(9998)),
            selected.command_id(),
            selected.operation(),
            Some(DirectoryActionV1::Create),
        )
        .unwrap(),
        NativePolicyCommandRef::from_people_command(&commands[4]), // Revoke instead of Grant
    ];
    for mode in MODES {
        for resolved in wrong {
            let mut plan = people_plan(mode, command);
            plan.resolved_selector = Some(resolved);
            let store = people_store(vec![plan], selected);
            let policy = Policy::new(&store);
            assert_eq!(
                invoke(mode, &store, &policy, selected, command),
                Err(NativePolicyWorkflowError::Unavailable)
            );
            assert_eq!(
                store.events(),
                [format!("open:{mode:?}"), format!("release:{mode:?}:false")]
            );
            store.drained();
        }
    }
}

#[test]
fn people_workflow_recovery_resolves_original_create_action_and_unresolved_forms_refuse() {
    let command = people_commands().remove(3);
    let exact = NativePolicyCommandRef::from_people_command(&command);
    let family_only = NativePolicyCommandRef::new_people(
        exact.company(),
        exact.command_id(),
        exact.operation(),
        None,
    )
    .unwrap();
    assert_ne!(family_only, exact);
    for mode in [Mode::Status, Mode::Execute] {
        let store = people_store(vec![people_plan(mode, &command)], family_only);
        let policy = Policy::new(&store);
        assert_eq!(invoke(mode, &store, &policy, family_only, &command), Ok(()));
        assert_eq!(store.events(), history(mode));
        assert!(store.history.lock().unwrap().input_bytes.is_empty());
        store.drained();
        let mut plan = people_plan(mode, &command);
        plan.resolved_selector = Some(family_only);
        let store = people_store(vec![plan], family_only);
        let policy = Policy::new(&store);
        assert_eq!(
            invoke(mode, &store, &policy, family_only, &command),
            Err(NativePolicyWorkflowError::Unavailable)
        );
        assert_eq!(
            store.events(),
            [format!("open:{mode:?}"), format!("release:{mode:?}:false")]
        );
        store.drained();
    }
    for mode in [Mode::Current, Mode::Form, Mode::ValidationForm] {
        let store = people_store(vec![], family_only);
        let policy = Policy::new(&store);
        assert_eq!(
            invoke(mode, &store, &policy, family_only, &command),
            Err(NativePolicyWorkflowError::InvalidInput)
        );
        assert!(store.events().is_empty());
        store.drained();
    }
    for action in [DirectoryActionV1::Read, DirectoryActionV1::Create] {
        assert!(
            NativePolicyCommandRef::new_people(
                exact.company(),
                exact.command_id(),
                NativeBusinessOperationV1::Install,
                Some(action)
            )
            .is_err()
        );
    }
}

#[test]
fn people_workflow_wrong_returned_family_or_action_never_finishes_or_discloses() {
    let commands = people_commands();
    let command = &commands[3];
    let selected = NativePolicyCommandRef::from_people_command(command);
    let payroll = NativeCompanyBusinessCommandV1::grant(
        selected.command_id(),
        selected.company(),
        1,
        AccountId::from_uuid(id(34)).unwrap(),
        None,
        at() + Duration::days(1),
    )
    .unwrap();
    for foreign in [
        NativePolicyCommand::Payroll(payroll),
        NativePolicyCommand::People(commands[1].clone()),
    ] {
        for mode in [Mode::Accept, Mode::Status, Mode::Execute] {
            let mut plan = people_plan(mode, command);
            let mut value = accepted();
            value.input = foreign.clone();
            plan.status = NativePolicyStatus::AcceptedPending(value.clone());
            plan.terminal.as_mut().unwrap().accepted = value;
            let store = people_store(vec![plan], selected);
            let policy = Policy::new(&store);
            assert_eq!(
                invoke(mode, &store, &policy, selected, command),
                Err(NativePolicyWorkflowError::Unavailable)
            );
            assert_eq!(
                store.events(),
                [
                    format!("open:{mode:?}"),
                    "decision".into(),
                    format!("operation:{mode:?}"),
                    format!("release:{mode:?}:false")
                ]
            );
            store.drained();
        }
    }
}

#[test]
fn people_submit_keeps_original_union_bytes_and_releases_a_before_same_family_b() {
    let command = people_commands().remove(3);
    let selected = NativePolicyCommandRef::from_people_command(&command);
    let store = people_store(
        vec![
            people_plan(Mode::Accept, &command),
            people_plan(Mode::Execute, &command),
        ],
        selected,
    );
    let policy = Policy::new(&store);
    let union = NativePolicyCommand::People(command.clone());
    let result = ready(submit_native_policy_command(
        &store,
        &policy,
        &(),
        &union,
        &TraceContext::generate(),
    ))
    .unwrap();
    assert_eq!(result.terminal.accepted.input, union);
    assert_eq!(
        store.history.lock().unwrap().input_bytes,
        vec![command.encode(binding().account)]
    );
    let mut events = history(Mode::Accept);
    events.extend(history(Mode::Execute));
    assert_eq!(store.events(), events);
    store.drained();
}

// Pure codec3 scope fence from the adopted Organization V12 design appendix,
// SHA256 c26751c6a49b759de50ee9e9dd42c7255b2403d1c874673f7b51b91d23dd3086.
// These exact literals add no catalog installation or active Organization owner.
#[test]
fn org_unit_codec3_is_unavailable_before_every_policy_store_entry() {
    let literals = [
        (
            "install",
            "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
        ),
        (
            "Read_grant_new",
            "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
        ),
        (
            "Read_grant_replace",
            "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
        ),
        (
            "Read_revoke",
            "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
        ),
        (
            "CreateSite_grant_new",
            "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
        ),
        (
            "CreateSite_grant_replace",
            "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
        ),
        (
            "CreateSite_revoke",
            "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
        ),
        (
            "CorrectSiteName_grant_new",
            "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
        ),
        (
            "CorrectSiteName_grant_replace",
            "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
        ),
        (
            "CorrectSiteName_revoke",
            "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
        ),
    ];
    assert_eq!(literals.len(), 10);
    let mut checked = 0;
    for (name, literal) in literals {
        let bytes = hex::decode(literal).unwrap();
        let (_, command) = NativePolicyCommand::decode(3, &bytes)
            .expect("ORG_UNIT_CODEC3_NO_STORE: exact reviewed literal rejected");
        let selected = NativePolicyCommandRef::from_command(&command);
        assert_eq!(selected.codec_version(), 3);
        assert_eq!(selected.manifest_digest(), command.manifest_digest());
        for mode in MODES {
            let mut store = Store::new(vec![]);
            store.expected_selector = selected;
            let policy = Policy::new(&store);
            let trace = TraceContext::generate();
            let result = match mode {
                Mode::Current => {
                    ready(native_policy_current(&store, &policy, &(), selected)).map(|_| ())
                }
                Mode::Form => ready(native_policy_form(&store, &policy, &(), selected)).map(|_| ()),
                Mode::ValidationForm => ready(native_policy_validation_form(
                    &store,
                    &policy,
                    &(),
                    selected,
                ))
                .map(|_| ()),
                Mode::Accept => ready(accept_native_policy_command(
                    &store,
                    &policy,
                    &(),
                    &command,
                    &trace,
                ))
                .map(|_| ()),
                Mode::Execute => ready(execute_native_policy_command(
                    &store,
                    &policy,
                    &(),
                    selected,
                    &trace,
                ))
                .map(|_| ()),
                Mode::Status => {
                    ready(native_policy_command_status(&store, &policy, &(), selected)).map(|_| ())
                }
            };
            assert_eq!(
                result,
                Err(NativePolicyWorkflowError::Unavailable),
                "codec-only {name} reached {mode:?}"
            );
            assert!(
                store.events().is_empty(),
                "codec-only {name} reached {mode:?} store or policy"
            );
            assert!(store.history.lock().unwrap().input_bytes.is_empty());
            store.drained();
            checked += 1;
        }
        let mut store = Store::new(vec![]);
        store.expected_selector = selected;
        let policy = Policy::new(&store);
        let result = ready(submit_native_policy_command(
            &store,
            &policy,
            &(),
            &command,
            &TraceContext::generate(),
        ));
        assert_eq!(
            result.map(|_| ()),
            Err(NativePolicyWorkflowError::Unavailable),
            "codec-only {name} reached submit"
        );
        assert!(
            store.events().is_empty(),
            "codec-only {name} reached submit store or policy"
        );
        assert!(store.history.lock().unwrap().input_bytes.is_empty());
        store.drained();
        checked += 1;
    }
    assert_eq!(checked, 70);
}
