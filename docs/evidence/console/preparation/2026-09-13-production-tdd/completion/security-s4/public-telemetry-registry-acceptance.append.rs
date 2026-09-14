//! Append child test module at existing console_telemetry.rs. The proposed
//! normalize_registered helper replaces/extends normalize at the same owner;
//! registry is actual immutable compiled release configuration, not test data.
#[cfg(all(test, not(feature = "test-postgres")))]
mod registered_public_telemetry_acceptance {
    use super::*;

    fn valid(registry: &PublicTelemetryRegistry) -> RouteTelemetryRequest {
        RouteTelemetryRequest {
            event_kind: RouteTelemetryEventKind::RumError,
            route_surface: RouteSurface::Console,
            route_path: registry
                .route_template("console.identity")
                .expect("actual shipped route registry")
                .to_owned(),
            release_cycle: registry.release_code().to_owned(),
            duration_ms: None,
            error_name: Some(
                registry
                    .error_code("route_boundary")
                    .expect("actual registered public error")
                    .to_owned(),
            ),
        }
    }
    fn public_shape(registry: &PublicTelemetryRegistry, value: &NormalizedRouteTelemetry) {
        assert!(registry.contains_route(&value.route_path));
        assert!(registry.contains_release(&value.release_cycle));
        assert!(
            value
                .error_name
                .as_ref()
                .is_none_or(|name| registry.contains_error(name))
        );
    }
    #[test]
    fn registered_route_release_and_error_are_real_positive_controls() {
        let registry = compiled_public_telemetry_registry();
        let request = valid(&registry);
        let actual =
            normalize_registered(request, &registry).expect("registered actual release event");
        public_shape(&registry, &actual);
    }
    #[test]
    fn grammar_valid_private_labels_cannot_survive_public_normalization() {
        let registry = compiled_public_telemetry_registry();
        for field in ["route_path", "release_cycle", "error_name"] {
            let mut input = valid(&registry);
            let canary = if field == "route_path" {
                "/people/private_salary_83000"
            } else {
                "private_salary_83000"
            };
            match field {
                "route_path" => input.route_path = canary.into(),
                "release_cycle" => input.release_cycle = canary.into(),
                _ => input.error_name = Some(canary.into()),
            }
            match normalize_registered(input, &registry) {
                Err(_) => {}
                Ok(normalized) => {
                    // A registered generic public fallback is allowed. An ad hoc
                    // scrubbed value that escaped the registry is not sufficient.
                    public_shape(&registry, &normalized);
                    assert!(!normalized.route_path.contains(canary));
                    assert!(!normalized.release_cycle.contains(canary));
                    assert!(
                        !normalized
                            .error_name
                            .as_deref()
                            .unwrap_or("")
                            .contains(canary)
                    );
                }
            }
        }
    }
}
