//! Exact 20-column material. Private raw source values never implement Debug.
use super::*;
use console_identity_application::company_policy::{CompanyProjectionRow, InitialCompanyAction};
use console_kernel_core::OrgId;

const SOURCE: &str = "SELECT actor_account_id, session_id, org_id, command_id, current_group_id, company_epoch, current_policy_receipt_id, context_generation, assignment_id, assignment_revision, role_id, role_revision, registered_clauses, company_name, company_slug, installed_object_type_id, observed_at, source_xid::text AS source_xid, source_backend_pid, source_material FROM public.identity_company_information_manager_current_v1($1,$2,$3,$4,$5) LIMIT 2";

pub(super) struct Projection {
    pub actor_account_id: Uuid,
    pub session_id: Uuid,
    pub org_id: Uuid,
    pub command_id: Uuid,
    pub current_group_id: Uuid,
    pub company_epoch: i64,
    pub current_policy_receipt_id: Option<Uuid>,
    pub context_generation: i64,
    pub assignment_id: Uuid,
    pub assignment_revision: i64,
    pub role_id: Uuid,
    pub role_revision: i64,
    pub registered_clauses: Value,
    pub company_name: String,
    pub company_slug: String,
    pub installed_object_type_id: Uuid,
    pub observed_at: OffsetDateTime,
    pub source_xid: u64,
    pub source_backend_pid: i32,
    pub source_material: Value,
}

pub(super) async fn read(
    tx: &mut Transaction<'_, Postgres>,
    actor: Uuid,
    family: Uuid,
    selector: NativePolicyCommandRef,
    group: Option<Uuid>,
) -> Result<Option<Projection>, Error> {
    let rows = sqlx::query(SOURCE)
        .bind(actor)
        .bind(family)
        .bind(*selector.company().as_uuid())
        .bind(selector.command_id())
        .bind(group)
        .fetch_all(tx.as_mut())
        .await
        .map_err(sql_error)?;
    let row = match rows.as_slice() {
        [] => return Ok(None),
        [row] => row,
        _ => return Err(Error::Unavailable),
    };
    macro_rules! value {
        ($name:ident) => {
            row.try_get(stringify!($name)).map_err(sql_error)?
        };
    }
    let xid: String = row.try_get("source_xid").map_err(sql_error)?;
    Ok(Some(Projection {
        actor_account_id: value!(actor_account_id),
        session_id: value!(session_id),
        org_id: value!(org_id),
        command_id: value!(command_id),
        current_group_id: value!(current_group_id),
        company_epoch: value!(company_epoch),
        current_policy_receipt_id: value!(current_policy_receipt_id),
        context_generation: value!(context_generation),
        assignment_id: value!(assignment_id),
        assignment_revision: value!(assignment_revision),
        role_id: value!(role_id),
        role_revision: value!(role_revision),
        registered_clauses: value!(registered_clauses),
        company_name: value!(company_name),
        company_slug: value!(company_slug),
        installed_object_type_id: value!(installed_object_type_id),
        observed_at: value!(observed_at),
        source_xid: xid_number(&xid)?,
        source_backend_pid: value!(source_backend_pid),
        source_material: value!(source_material),
    }))
}

impl Projection {
    pub(super) fn bind(
        &self,
        selector: NativePolicyCommandRef,
        session: &AccountLiveSession,
    ) -> Result<(), Error> {
        for id in [
            self.actor_account_id,
            self.session_id,
            self.org_id,
            self.command_id,
            self.current_group_id,
            self.assignment_id,
            self.role_id,
            self.installed_object_type_id,
        ] {
            nonnil(id)?;
        }
        for n in [
            self.company_epoch,
            self.context_generation,
            self.assignment_revision,
            self.role_revision,
        ] {
            positive(n)?;
        }
        if self.actor_account_id != session.account_id
            || self.session_id != session.session_id
            || self.org_id != *selector.company().as_uuid()
            || self.command_id != selector.command_id()
            || self.context_generation > 257
            || self.assignment_revision != 1
            || self.role_revision != 1
            || self.source_backend_pid <= 0
            || self.source_xid == 0
            || (self.company_epoch == 1) != self.current_policy_receipt_id.is_none()
            || self.current_policy_receipt_id.is_some_and(|id| id.is_nil())
        {
            return Err(Error::Unavailable);
        }
        exact_time(self.observed_at)?;
        material::bind(self, session)?;
        self.valid_at(self.observed_at)
    }

    pub(super) fn authority(&self) -> Result<CurrentCompanyAuthority, Error> {
        CurrentCompanyAuthority::from_current_projection(
            account(self.actor_account_id)?,
            OrgId::from_uuid(self.org_id),
            self.observed_at,
            CompanyProjectionRow {
                company_epoch: self.company_epoch,
                context_generation: self.context_generation,
                assignment_id: self.assignment_id,
                assignment_revision: self.assignment_revision,
                role_id: self.role_id,
                role_revision: self.role_revision,
                registered_clauses: self.registered_clauses.to_string(),
                company_name: self.company_name.clone(),
                company_slug: self.company_slug.clone(),
            },
            self.current_policy_receipt_id,
        )
        .map_err(|_| Error::Unavailable)
    }

    pub(super) fn view(
        &self,
        selector: NativePolicyCommandRef,
    ) -> Result<NativePolicyFormView, Error> {
        let authority = self.authority()?;
        let workspace = authority
            .clauses()
            .iter()
            .find(|c| !c.delegable() && c.action_kind() == InitialCompanyAction::Discover)
            .ok_or(Error::Unavailable)?
            .action()
            .object_type_id();
        if workspace != self.installed_object_type_id {
            return Err(Error::Unavailable);
        }
        Ok(NativePolicyFormView {
            selector,
            group_id: self.current_group_id,
            company_epoch: positive(self.company_epoch)?,
            acting_account_id: authority.account(),
            administrative_account_id: authority.account(),
            installed_object_type_id: Some(workspace),
            assignment: None,
        })
    }

    pub(super) fn same_source(&self, after: &Self) -> Result<(), Error> {
        macro_rules! same { ($($field:ident),+ $(,)?) => {
            if $(self.$field != after.$field)||+ { return Err(Error::Conflict); }
        }; }
        same!(
            actor_account_id,
            session_id,
            org_id,
            command_id,
            current_group_id,
            company_epoch,
            current_policy_receipt_id,
            context_generation,
            assignment_id,
            assignment_revision,
            role_id,
            role_revision,
            registered_clauses,
            company_name,
            company_slug,
            installed_object_type_id,
            source_xid,
            source_backend_pid,
            source_material
        );
        if after.observed_at < self.observed_at {
            return Err(Error::Unavailable);
        }
        Ok(())
    }

    pub(super) fn valid_at(&self, now: OffsetDateTime) -> Result<(), Error> {
        material::valid_at(&self.source_material, now)
    }
}
