//! Finite retained Manager Current read. All authority remains provisional
//! until the original source/Auth, ordinary Cedar and COMMIT are confirmed.
use super::*;
use console_identity_application::company_policy::{
    CurrentCompanyAuthority,
    workflow::company_information_current::{
        CompanyInformationManagerCurrentScope, CompanyInformationManagerCurrentStore,
        authorize_company_information_current,
    },
};

mod material;
mod projection;
use projection::Projection;

pub struct PgCompanyInformationCurrentScope<'a> {
    tx: Transaction<'static, Postgres>,
    config: &'a NativeAccountReadConfig,
    credentials: &'a AccountEnrollmentCredentials,
    selector: NativePolicyCommandRef,
    session: AccountLiveSession,
    source: Option<Projection>,
    authority: Option<CurrentCompanyAuthority>,
    view: Option<NativePolicyFormView>,
}

impl CompanyInformationManagerCurrentStore for PgOrgStore {
    type Credentials = AccountEnrollmentCredentials;
    type Scope<'a> = PgCompanyInformationCurrentScope<'a>;

    async fn lock_company_information_current<'a>(
        &'a self,
        credentials: &'a Self::Credentials,
        selector: NativePolicyCommandRef,
    ) -> Result<Self::Scope<'a>, Error> {
        if selector.codec_version() != 4 || selector.operation() != NativeBusinessOperationV1::Grant
        {
            return Err(Error::Unavailable);
        }
        let config = self.native_read_config().map_err(|_| Error::Unavailable)?;
        if !matches!(config.mode, NativeAccountMode::Policy { .. }) {
            return Err(Error::Unavailable);
        }
        let mut tx = self
            .native_read_transaction()
            .await
            .map_err(|_| Error::Unavailable)?;
        // Cryptographic namespace only: the SQL owner acquires Group first.
        let (actor, family) = credentials
            .session_ids_in_tx(&mut tx, &config.verifier, config.absolute_ttl)
            .await
            .map_err(auth_error)?;
        let source = projection::read(&mut tx, actor, family, selector, None).await?;
        let session = credentials
            .read_session_in_tx(&mut tx, &config.verifier, config.absolute_ttl)
            .await
            .map_err(auth_error)?;
        if (session.account_id, session.session_id) != (actor, family) {
            return Err(Error::Unavailable);
        }
        let (authority, view) = if let Some(source) = &source {
            source.bind(selector, &session)?;
            let (xid, pid): (String, i32) = sqlx::query_as(
                "SELECT pg_catalog.pg_current_xact_id()::text,pg_catalog.pg_backend_pid()",
            )
            .fetch_one(tx.as_mut())
            .await
            .map_err(sql_error)?;
            if xid_number(&xid)? != source.source_xid || pid != source.source_backend_pid {
                return Err(Error::Unavailable);
            }
            let authority = source.authority()?;
            let view = source.view(selector)?;
            (Some(authority), Some(view))
        } else {
            (None, None)
        };
        Ok(PgCompanyInformationCurrentScope {
            tx,
            config,
            credentials,
            selector,
            session,
            source,
            authority,
            view,
        })
    }
}

impl PgCompanyInformationCurrentScope<'_> {
    async fn original_session(&mut self) -> Result<AccountLiveSession, Error> {
        let session = self
            .credentials
            .read_session_in_tx(
                &mut self.tx,
                &self.config.verifier,
                self.config.absolute_ttl,
            )
            .await
            .map_err(auth_error)?;
        if session.account_id != self.session.account_id
            || session.session_id != self.session.session_id
            || session.security_generation != self.session.security_generation
            || session.auth_time != self.session.auth_time
            || session.assurance != self.session.assurance
            || session.expires_at != self.session.expires_at
            || session.family_expires_at != self.session.family_expires_at
        {
            return Err(Error::AuthenticationInvalid);
        }
        Ok(session)
    }
}

impl CompanyInformationManagerCurrentScope for PgCompanyInformationCurrentScope<'_> {
    fn selector(&self) -> NativePolicyCommandRef {
        self.selector
    }
    fn authority(&self) -> Option<&CurrentCompanyAuthority> {
        self.authority.as_ref()
    }
    fn view(&self) -> Option<&NativePolicyFormView> {
        self.view.as_ref()
    }

    async fn finish_not_found(mut self) -> Result<(), Error> {
        // Absence/initial Deny never executes another Group planning read.
        let session = self.original_session().await?;
        let now = ensure_account_session_fresh_in_tx(&mut self.tx, &session)
            .await
            .map_err(auth_error)?;
        if let Some(source) = &self.source {
            source.valid_at(now)?;
        }
        self.tx.rollback().await.map_err(|_| Error::Unavailable)
    }

    async fn finish<P: CompanyPolicyDecisionPort + ?Sized>(
        mut self,
        policy: &P,
    ) -> Result<(), Error> {
        let group = self
            .source
            .as_ref()
            .ok_or(Error::Unavailable)?
            .current_group_id;
        sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
            .execute(self.tx.as_mut())
            .await
            .map_err(sql_error)?;
        let final_source = projection::read(
            &mut self.tx,
            self.session.account_id,
            self.session.session_id,
            self.selector,
            Some(group),
        )
        .await?;
        // Resolve live Auth even when a formerly present source returns zero.
        let session = self.original_session().await?;
        let final_source = final_source.ok_or(Error::Conflict)?;
        let original = self.source.as_ref().ok_or(Error::Unavailable)?;
        original.same_source(&final_source)?;
        final_source.bind(self.selector, &session)?;
        let authority = final_source.authority()?;
        let decision = authorize_company_information_current(policy, &authority)?;
        // A fresh database sample after every wait and both decisions. Final
        // Deny also completes the original Auth check before disclosing absence.
        let now = ensure_account_session_fresh_in_tx(&mut self.tx, &session)
            .await
            .map_err(auth_error)?;
        final_source.valid_at(now)?;
        if decision == CompanyPolicyDecision::Deny {
            return Err(Error::NotFound);
        }
        self.tx.commit().await.map_err(|_| Error::Unavailable)
    }
}
