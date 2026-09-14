//! Verus HARNESS candidate for exact-source-bound owners, not replacement bodies.
//! source_binding_gate supplies only reviewed byte-identical executable extracts
//! and exact source structs in bound_owner.rs when production owners exist.
//! Ghost views below are specified by actual source fields in that binding; they
//! cannot be implemented as equality of outputs or supplied allowed=true.
use vstd::prelude::*;
include!("bound_owner.rs");
verus! {
// AllowedSourceView reads only explicitly permitted actual fields and all public
// shape inputs (including absence, order, errors, caller-visible dependency facts).
// Clock/random tape are independently chosen public inputs, not secret outputs.
pub open spec fn low_equivalent(a: OwnerSource, b: OwnerSource, authority: CurrentFieldAuthority, env: PublicEnvironment) -> bool {
 a.context_view()==b.context_view() && a.public_environment()==env && b.public_environment()==env
 && allowed_source_view(a,authority)==allowed_source_view(b,authority)
}
pub fn projection_pair(a: OwnerSource,b:OwnerSource,ctx:&ActiveContext,decisions:&CurrentFieldAuthority,env:&PublicEnvironment) -> (out:(AuthorizedProjection,AuthorizedProjection))
 requires current_authority_from_owner(ctx,decisions), low_equivalent(a,b,*decisions,*env)
 ensures logical_projection_view(out.0)==logical_projection_view(out.1)
{
 let left=project_authorized_result(ctx,&a,decisions,env);
 let right=project_authorized_result(ctx,&b,decisions,env);
 assert(logical_projection_view(left)==logical_projection_view(right));
 (left,right)
}
pub fn current_binding_rejects_other_company(actual:&CurrentAuthority,presented:&UntrustedContextSelection)->(ok:bool)
 requires actual.company_id()!=presented.company_id()
 ensures !ok
{let result=bind_active_context(actual,presented);assert(result.is_err());result.is_ok()}
pub fn event_pair(code:RegisteredEventCode,route:PublicRouteTemplate,incident:OpaqueIncidentId,result:PublicResultClass)->(events:(SafeOperationalEvent,SafeOperationalEvent))
 ensures event_view(events.0)==event_view(events.1)
{let a=build_safe_operational_event(code,route,incident,result);let b=build_safe_operational_event(code,route,incident,result);assert(event_view(a)==event_view(b));(a,b)}
pub fn egress_wrong_binding_refuses(ctx:&ActiveContext,p:&AuthorizedProjection,b:&AdmittedEgressBinding,selected:&DestinationSelection)->(ok:bool)
 requires destination_view(*selected)!=admitted_destination_view(*b)
 ensures !ok
{let result=admit_bound_egress(ctx,p,b,selected);assert(result.is_err());result.is_ok()}
pub fn useful_a_projection(a:OwnerSource,ctx:&ActiveContext,decisions:&CurrentFieldAuthority,env:&PublicEnvironment,address:FieldAddress)->(out:AuthorizedProjection)
 requires current_authority_from_owner(ctx,decisions), decisions.permits(address), a.has_scalar(address)
 ensures out.has_scalar(address), out.scalar(address)==a.scalar(address)
{let out=project_authorized_result(ctx,&a,decisions,env);assert(out.has_scalar(address));assert(out.scalar(address)==a.scalar(address));out}
pub fn whole_event_schedule_pair(a:OwnerSource,b:OwnerSource,ctx:&ActiveContext,decisions:&CurrentFieldAuthority,env:&PublicEnvironment)->(out:(EventSchedule,EventSchedule))
 requires current_authority_from_owner(ctx,decisions), low_equivalent(a,b,*decisions,*env)
 ensures schedule_view(out.0)==schedule_view(out.1)
{let left=project_and_schedule_response(ctx,&a,decisions,env);let right=project_and_schedule_response(ctx,&b,decisions,env);assert(schedule_view(left)==schedule_view(right));(left,right)}
// Whole event schedule and sink serializers must be source-bound callers, not an
// assumption that individually safe records imply equal count/order. The runtime
// observation harness checks exact sequences and the required extra-event mutant.
}
