//! Group identities are independent of Company and never establish authority.
use console_kernel_core::KernelError;
use uuid::Uuid;

macro_rules! group_identity {
    ($name:ident, $message:literal) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(Uuid);

        impl $name {
            pub fn from_uuid(value: Uuid) -> Result<Self, KernelError> {
                if value.is_nil() {
                    return Err(KernelError::validation($message));
                }
                Ok(Self(value))
            }

            pub const fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }
    };
}

group_identity!(GroupId, "Group ID is nil");
group_identity!(GroupIncarnation, "Group incarnation is nil");
