//! Mount inside backend/app/src/console_telemetry.rs; no logging of private input.
#[cfg(test)]
mod safe_event_acceptance {
    use super::*;
    #[test]
    fn safe_event_actual_constructor_and_serializer_accept_only_registered_public_fields() {
        let registry = compiled_operational_event_registry();
        let code = registry
            .event_code("request.completed")
            .expect("actual registered event");
        let route = registry
            .route_template("console.payroll.run")
            .expect("actual route template");
        let incident = OpaqueIncidentId::parse("bdad6075-6e3c-4fe4-b7d5-39dcc6cbdbfd").unwrap();
        let class = registry
            .result_class("success")
            .expect("actual public result class");
        let event = build_safe_operational_event(code, route, incident, class);
        let bytes = encode_safe_operational_event(&event).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let keys = v
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            keys,
            std::collections::BTreeSet::from([
                "code",
                "route_template",
                "incident_id",
                "result_class"
            ])
        );
        assert_eq!(v["code"], "request.completed");
        assert_eq!(v["result_class"], "success");
        assert_eq!(v["incident_id"], "bdad6075-6e3c-4fe4-b7d5-39dcc6cbdbfd");
        assert!(registry.contains_route(v["route_template"].as_str().unwrap()));
        for secret in [
            "private_salary_7313371",
            "/people/private_salary_7313371",
            "SELECT gross_won FROM private_payroll",
        ] {
            assert!(registry.event_code(secret).is_err());
            assert!(registry.route_template(secret).is_err());
            assert!(registry.result_class(secret).is_err());
            assert!(OpaqueIncidentId::parse(secret).is_err());
        }
    }
}
