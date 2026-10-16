// Include INSIDE native_people_policy_http_owner after independent review.
// Existing owner acceptance creates actual pending material; HTTP alone performs
// its effect. No database truth writes, fixture grants or fabricated receipts.
mod recovery_and_validation {
    use super::*;

    #[sqlx::test(migrations = false)]
    async fn people_pending_actionless_recovery_executes_only_original_bytes_and_validation_retains_subject(
        pool: PgPool,
    ) {
        let (app, key, state) =
            super::super::native_people_codec2_install_probe::configured_successor_fixture(&pool)
                .await;
        let mut cleanup = None;
        let outcome=AssertUnwindSafe(async {
            let (app,cookies,created)=create_owned_company(&pool,app).await;
            let (verifier,issuer,ttl)=bindings(&account_browser_config(&pool,app._artifacts.root.clone(),&key));
            let runtime=login_test_pool(&pool,TestDatabaseLogin::Business).await;cleanup=Some(runtime.clone());
            let store=PgOrgStore::new(runtime).with_native_account_policy(verifier,issuer,ttl);
            let policy=CompanyPolicy::new().unwrap();let company=OrgId::from_uuid(created.company);
            let actor=AccountId::from_uuid(created.administrator).unwrap();
            let installed=transition(&pool,&app,&cookies,&store,&policy,company,actor,
                NativeBusinessOperationV1::Install,None,1,None,None).await;
            let root=format!("/companies/{company}/policy/people-directory");
            let mut predecessor=installed.receipt_id;
            for (index,action) in [DirectoryActionV1::Read,DirectoryActionV1::Create].into_iter().enumerate(){
                let action_path=if action==DirectoryActionV1::Read{"read"}else{"create"};
                let title=if action==DirectoryActionV1::Read{"사람 열람 권한"}else{"사람 등록 권한"};
                let epoch=index as u64+2;
                let page=preflight(&pool,&app,&format!("{root}/{action_path}/grant"),&cookies,StatusCode::OK).await;
                let html=native_entry_html(&page,StatusCode::OK);
                let command:Uuid=field(html,"command_id").parse().unwrap();
                let proof=field(html,"csrf_proof");
                let mut fields=vec![("command_id".into(),command.to_string()),("expected_company_epoch".into(),epoch.to_string()),
                    ("csrf_proof".into(),proof.clone())];
                for key in ["recipient_account_id","expected_role_revision","assignment_id","expected_assignment_revision"]{
                    fields.push((key.into(),field(html,key)));
                }
                fields.push(("expires_at_local".into(),"not-a-date".into()));
                let before_validation=all_rows(&pool).await;
                let mut invalid=post(&app,&format!("{root}/{action_path}/grants"),&cookies,&fields).await;
                // Validation deliberately retains this exact submitted form proof.
                // Permit only its single expected hidden-field occurrence; keep
                // cookie leak/private-header checks, and forbid proof headers.
                assert_eq!(invalid.bytes.windows(proof.len()).filter(|w|*w==proof.as_bytes()).count(),1);
                for (_,value) in &invalid.headers{assert!(!value.as_bytes().windows(proof.len()).any(|w|w==proof.as_bytes()));}
                invalid.sent_secrets.retain(|secret|secret!=&proof);
                let invalid_html=native_entry_html(&invalid,StatusCode::UNPROCESSABLE_ENTITY);
                assert!(invalid_html.contains(title));
                assert!(invalid_html.contains(&format!("action=\"{root}/{action_path}/grants\"")));
                for (key,value) in &fields {assert_eq!(field(invalid_html,key),*value,"validation replaced {key}");}
                assert!(invalid_html.contains("aria-invalid=\"true\""));
                assert!(before_validation==all_rows(&pool).await,"expiry validation persisted or minted proof");
                let until=time::OffsetDateTime::from_unix_timestamp((clock(&pool).await.unix_timestamp()/60+1440)*60).unwrap();
                let input=NativePolicyCommand::People(NativePeoplePolicyCommandV1::grant(command,company,epoch,action,actor,None,until).unwrap());
                let credentials=AccountEnrollmentCredentials::for_mutation(&cookies.0[ACCESS],&proof).unwrap();
                let accepted=accept_native_policy_command(&store,&policy,&credentials,&input,&TraceContext::generate()).await.unwrap();
                assert!(accepted.inserted);
                let pending=match accepted.status{NativePolicyStatus::AcceptedPending(a)=>a,_=>panic!("actual owner did not accept pending")};
                assert_eq!(pending.input,input);
                let after_accept=all_rows(&pool).await;
                assert!(before_validation.keys().eq(after_accept.keys()));
                for(table,old)in &before_validation{
                    assert_eq!(added_rows(old,&after_accept[table]).expect("prepare rewrote history").len(),
                        usize::from(table=="native_company_policy_inputs_v1"||table=="audit_events"),"prepare effect in {table}");
                }
                let recovery=format!("{root}/requests/grant/{command}");
                let pending_page=preflight(&pool,&app,&recovery,&cookies,StatusCode::OK).await;
                let pending_html=native_entry_html(&pending_page,StatusCode::OK);
                assert!(pending_html.contains(title)&&pending_html.contains("data-policy-outcome=\"pending\""));
                assert!(pending_html.contains(&format!("action=\"{recovery}/retry\"")));
                assert!(!pending_html.contains("name=\"command_id\"")&&!pending_html.contains("name=\"expected_company_epoch\""));
                let retry=vec![("csrf_proof".into(),field(pending_html,"csrf_proof"))];
                let before_retry=all_rows(&pool).await;
                // Exact limiter increments were already independently proved.
                // Compose ONLY those verified increments into the pre-accept
                // reference, then verify the entire accept+execute census.
                let mut reference=before_validation.clone();
                reference.insert("auth_rate_limit".into(),before_retry["auth_rate_limit"].clone());
                let mut replaced=retry.clone();replaced.push(("command_id".into(),command.to_string()));
                assert_eq!(post(&app,&format!("{recovery}/retry"),&cookies,&replaced).await.status,StatusCode::BAD_REQUEST);
                assert!(before_retry==all_rows(&pool).await,"replacement retry wrote effects");
                let response=post(&app,&format!("{recovery}/retry"),&cookies,&retry).await;
                assert_eq!(response.status,StatusCode::SEE_OTHER);response.private();
                assert_eq!(response.headers.get(header::LOCATION).unwrap(),recovery.as_str());
                let selector=NativePolicyCommandRef::from_command(&input);
                let terminal=match native_policy_command_status(&store,&policy,&read_credentials(&cookies),selector).await.unwrap(){
                    NativePolicyStatus::Terminal(t)=>t,_=>panic!("retry response lacks exact terminal")};
                assert_eq!(terminal.accepted,pending);assert_eq!(terminal.accepted.input,input);
                assert_eq!((terminal.epoch_before,terminal.epoch_after),(epoch,epoch+1));
                let bound:bool=sqlx::query_scalar("SELECT i.codec_version=2 AND i.input_bytes=$4 AND i.input_digest=$5 AND r.input_digest=i.input_digest AND r.intake_receipt_id=$6 AND r.intake_receipt_id=i.intake_receipt_id AND r.receipt_id=$7 AND r.predecessor_receipt_id=$8 AND r.effect_xid<>i.acceptance_xid FROM public.native_company_policy_inputs_v1 i JOIN public.native_company_policy_receipts_v1 r USING(actor_account_id,command_id,org_id) WHERE i.actor_account_id=$1 AND i.command_id=$2 AND i.org_id=$3")
                    .bind(actor.as_uuid()).bind(command).bind(company.as_uuid()).bind(input.encode(actor))
                    .bind(Sha256::digest(input.encode(actor)).to_vec()).bind(pending.intake_receipt_id).bind(terminal.receipt_id).bind(predecessor)
                    .fetch_one(&pool).await.unwrap();assert!(bound);
                let after=all_rows(&pool).await;census(&reference,&after,&input,&terminal,Some(predecessor));
                let terminal_page=document(&app,&recovery,&cookies).await;let terminal_html=native_entry_html(&terminal_page,StatusCode::OK);
                assert!(terminal_html.contains(title)&&terminal_html.contains("data-policy-outcome=\"committed\""));
                assert!(!terminal_html.contains("name=\"csrf_proof\""));
                let replay=post(&app,&format!("{recovery}/retry"),&cookies,&retry).await;
                assert_eq!(replay.status,StatusCode::SEE_OTHER);
                assert_eq!(replay.headers.get(header::LOCATION).unwrap(),recovery.as_str());
                assert!(after==all_rows(&pool).await,"terminal reopening or proof-only duplicate wrote effects");
                // Route family/operation selection cannot reopen another input.
                for (route,expected) in [
                    (format!("/companies/{company}/policy/payroll-read/requests/grant/{command}"),StatusCode::SERVICE_UNAVAILABLE),
                    (format!("{root}/requests/revoke/{command}"),StatusCode::CONFLICT)]{
                    let response=document(&app,&route,&cookies).await;
                    assert_eq!(response.status,expected,"mismatched locator: {route}");
                    let html=String::from_utf8(response.bytes).unwrap();
                    assert!(!html.contains(&terminal.receipt_id.to_string())&&!html.contains("name=\"csrf_proof\""));
                    assert!(after==all_rows(&pool).await,"wrong locator wrote effects");
                }
                for route in [format!("{root}/READ/grant"),format!("{root}/create/install"),
                    format!("{root}/requests/grant/not-a-command"),format!("{root}/requests/GRANT/{command}")]{
                    assert_eq!(document(&app,&route,&cookies).await.status,StatusCode::NOT_FOUND);
                    assert!(after==all_rows(&pool).await,"malformed route wrote effects");
                }
                predecessor=terminal.receipt_id;
            }
        }).catch_unwind().await;
        if let Some(runtime) = cleanup {
            runtime.close().await;
        }
        close_states(&[state], outcome).await;
    }
}
