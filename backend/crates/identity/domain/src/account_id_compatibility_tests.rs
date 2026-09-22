// Mount inside identity/domain lib.rs after moving and re-exporting exact AccountId.
#[test]
fn original_identity_import_is_the_kernel_account_type() {
    fn kernel_to_identity(value: console_kernel_core::AccountId) -> crate::AccountId {
        value
    }
    fn identity_to_kernel(value: crate::AccountId) -> console_kernel_core::AccountId {
        value
    }
    let raw = uuid::Uuid::from_u128(21);
    let old_path = crate::AccountId::from_uuid(raw).unwrap();
    let kernel_path = identity_to_kernel(old_path);
    assert_eq!(*kernel_path.as_uuid(), raw);
    assert_eq!(kernel_to_identity(kernel_path), old_path);
    assert!(crate::AccountId::from_uuid(uuid::Uuid::nil()).is_err());
}
