//! Existing executable application boundary; scripted port, actual Cedar.
//! Mount as child of native_workflow_usecases_tests; no SQL acceptance claim.
use super::*;

#[test]
fn workflow_refuses_foreign_returned_input_before_finish_or_disclosure() {
    for mode in [Mode::Accept, Mode::Status] {
        for variant in 0..3 {
            for same_locator_changed_epoch in [false, true] {
                if same_locator_changed_epoch && mode != Mode::Accept {
                    continue;
                }
                let wrong = NativeCompanyBusinessCommandV1::install(
                    if same_locator_changed_epoch {
                        id(1000)
                    } else {
                        id(9999)
                    },
                    OrgId::from_uuid(id(11)),
                    if same_locator_changed_epoch { 2 } else { 1 },
                )
                .unwrap();
                let mut value = accepted();
                value.input = wrong.into();
                let status = match variant {
                    0 => NativePolicyStatus::AcceptedPending(value),
                    1 => NativePolicyStatus::AcceptedExpired(value),
                    _ => {
                        let mut receipt = terminal();
                        receipt.accepted = value;
                        NativePolicyStatus::Terminal(receipt)
                    }
                };
                let mut plan = Plan::new(mode);
                plan.status = status;
                let store = Store::new(vec![plan]);
                let policy = Policy::new(&store);
                let actual = match mode {
                    Mode::Accept => ready(accept_native_policy_command(
                        &store,
                        &policy,
                        &(),
                        &input(),
                        &TraceContext::generate(),
                    ))
                    .map(|_| ()),
                    Mode::Status => ready(native_policy_command_status(
                        &store,
                        &policy,
                        &(),
                        selector(),
                    ))
                    .map(|_| ()),
                    _ => unreachable!(),
                };
                assert_eq!(actual, Err(NativePolicyWorkflowError::Unavailable));
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
}

#[test]
fn workflow_refuses_foreign_returned_form_selector_before_finish() {
    for mode in [Mode::Current, Mode::Form, Mode::ValidationForm] {
        let mut store = Store::new(vec![Plan::new(mode)]);
        store.expected_selector = NativePolicyCommandRef::new(
            OrgId::from_uuid(id(11)),
            id(9999),
            NativeBusinessOperationV1::Install,
        )
        .unwrap();
        let requested = store.expected_selector;
        let policy = Policy::new(&store);
        let actual = match mode {
            Mode::Current => {
                ready(native_policy_current(&store, &policy, &(), requested)).map(|_| ())
            }
            Mode::Form => ready(native_policy_form(&store, &policy, &(), requested)).map(|_| ()),
            Mode::ValidationForm => ready(native_policy_validation_form(
                &store,
                &policy,
                &(),
                requested,
            ))
            .map(|_| ()),
            _ => unreachable!(),
        };
        assert_eq!(actual, Err(NativePolicyWorkflowError::Unavailable));
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

#[test]
fn workflow_refuses_foreign_returned_execution_before_finish() {
    let mut store = Store::new(vec![Plan::new(Mode::Execute)]);
    store.expected_selector = NativePolicyCommandRef::new(
        OrgId::from_uuid(id(11)),
        id(9999),
        NativeBusinessOperationV1::Install,
    )
    .unwrap();
    let policy = Policy::new(&store);
    let actual = ready(execute_native_policy_command(
        &store,
        &policy,
        &(),
        store.expected_selector,
        &TraceContext::generate(),
    ))
    .map(|_| ());
    assert_eq!(actual, Err(NativePolicyWorkflowError::Unavailable));
    assert_eq!(
        store.events(),
        [
            "open:Execute",
            "decision",
            "operation:Execute",
            "release:Execute:false"
        ]
    );
    store.drained();
}
