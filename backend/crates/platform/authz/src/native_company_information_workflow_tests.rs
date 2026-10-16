//! Shared application inactive-family fence only; scripted store is not PostgreSQL.
//! Frozen Company-information V2 design SHA256:
//! 999c457cae8cbae8640c6dc55e0819e0e2ee0cbba36e0ec67ed895b630255c8a.
use super::*;

#[test]
fn company_information_codec4_is_unavailable_before_every_policy_store_entry() {
    let literals = [
        (
            "grant_exact_microseconds",
            NativeBusinessOperationV1::Grant,
            "636f6e736f6c652e636f6d70616e792e696e666f726d6174696f6e2d706f6c69637900000400000000000000000000000000000001000000000000000000000000000000020000000000000000000000000000000300000000000000070d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935020014ed9a8cec82ac20eca095ebb3b420ed9995ec9db8000000000000000000000000000000040000000000000000000000000000004e000000000000000100065d34cbd9e00100065d353723b20702000000000000000000000000000000020000000000000000000000000000000b0000000000000000000000000000000600000000000000010d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a08993500000000000000000000000000000002020000000000000000000000000000000200000000000000000000000000000006000000000000000000000000000000150000000000000001000000000000000000000000000000020000000000000000000000000000000600000000000000000000000000000016000000000000000100065d34cbd9e0010100065d353723b20700000000000000000000000000000000020000000000000000000000000000000c0000000000000000000000000000000600000000000000010d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a08993500000000000000000000000000000002020000000000000000000000000000000200000000000000000000000000000006000000000000000000000000000000150000000000000001000000000000000000000000000000020000000000000000000000000000000600000000000000000000000000000016000000000000000100065d34cbd9e0010100065d353723b20700",
        ),
        (
            "revoke_exact_child",
            NativeBusinessOperationV1::Revoke,
            "636f6e736f6c652e636f6d70616e792e696e666f726d6174696f6e2d706f6c69637900000400000000000000000000000000000001000000000000000000000000000000020000000000000000000000000000001e00000000000000070d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a08993503000dec97b4eb9e8c20eca285eba38c0000000000000000000000000000004f0000000000000001",
        ),
    ];
    assert_eq!(literals.len(), 2);
    let mut checked = 0;
    for (name, operation, literal) in literals {
        let bytes = hex::decode(literal).unwrap();
        let (actor, command) = NativePolicyCommand::decode(4, &bytes)
            .expect("COMPANY_INFORMATION_CODEC4_NO_STORE: exact reviewed literal rejected");
        assert_eq!(actor, AccountId::from_uuid(id(1)).unwrap());
        assert_eq!(
            command.encode(actor),
            bytes,
            "accepted literal changed: {name}"
        );
        assert_eq!(command.operation(), operation);
        let selected = NativePolicyCommandRef::from_command(&command);
        assert_eq!(selected.codec_version(), 4);
        assert_eq!(selected.manifest_digest(), command.manifest_digest());
        let payroll = NativePolicyCommandRef::new(
            selected.company(),
            selected.command_id(),
            selected.operation(),
        )
        .unwrap();
        assert_ne!(selected, payroll);
        assert_eq!(
            selected.resolve(payroll),
            Err(NativePolicyWorkflowError::Unavailable)
        );
        assert_eq!(
            payroll.resolve(selected),
            Err(NativePolicyWorkflowError::Unavailable)
        );
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
    assert_eq!(checked, 14);
}
