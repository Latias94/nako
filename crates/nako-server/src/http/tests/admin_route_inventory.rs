use std::collections::BTreeMap;

use nako_api::admin_contract::{
    admin_contract_route_exclusions, admin_contract_routes, normalize_admin_route_path,
};

const ADMIN_HTTP_ROUTE_MODULE: &str = include_str!("../admin.rs");
const ADDON_ADMIN_ROUTE_MODULE: &str = include_str!("../addons.rs");

#[test]
fn implemented_admin_routes_are_generated_or_explicitly_excluded() {
    let implemented = implemented_admin_routes_by_path();
    assert!(
        !implemented.is_empty(),
        "Admin route inventory parser did not find any implemented /admin/v1 routes"
    );

    let generated = generated_admin_routes_by_path();
    let exclusions = excluded_admin_routes_by_path();

    let generated_without_server = generated
        .iter()
        .filter(|(path, _key)| !implemented.contains_key(*path))
        .map(|(path, key)| format!("{key} -> {path}"))
        .collect::<Vec<_>>();
    assert!(
        generated_without_server.is_empty(),
        "Generated Admin route constants must map to implemented server routes:\n{}",
        generated_without_server.join("\n")
    );

    let stale_exclusions = exclusions
        .iter()
        .filter(|(path, _reason)| !implemented.contains_key(*path))
        .map(|(path, reason)| format!("{path} ({reason})"))
        .collect::<Vec<_>>();
    assert!(
        stale_exclusions.is_empty(),
        "Admin route exclusions must refer to implemented server routes:\n{}",
        stale_exclusions.join("\n")
    );

    let unclassified = implemented
        .iter()
        .filter(|(path, _sources)| {
            !generated.contains_key(*path) && !exclusions.contains_key(*path)
        })
        .map(|(path, sources)| format!("{path} [{}]", sources.join(", ")))
        .collect::<Vec<_>>();
    assert!(
        unclassified.is_empty(),
        "Implemented Admin routes must be generated or explicitly excluded:\n{}",
        unclassified.join("\n")
    );
}

fn implemented_admin_routes_by_path() -> BTreeMap<String, Vec<String>> {
    let mut routes = BTreeMap::new();
    for route in extract_admin_route_literals("admin.rs", ADMIN_HTTP_ROUTE_MODULE)
        .into_iter()
        .chain(extract_admin_route_literals(
            "addons.rs",
            ADDON_ADMIN_ROUTE_MODULE,
        ))
    {
        routes
            .entry(normalize_admin_route_path(&route.path))
            .or_insert_with(Vec::new)
            .push(format!("{}:{}", route.module, route.path));
    }
    routes
}

fn generated_admin_routes_by_path() -> BTreeMap<String, &'static str> {
    let mut routes = BTreeMap::new();
    for route in admin_contract_routes() {
        let normalized = normalize_admin_route_path(&route.path);
        let previous = routes.insert(normalized, route.key);
        assert!(
            previous.is_none(),
            "Duplicate generated Admin route path for key {}",
            route.key
        );
    }
    routes
}

fn excluded_admin_routes_by_path() -> BTreeMap<String, &'static str> {
    let mut routes = BTreeMap::new();
    for exclusion in admin_contract_route_exclusions() {
        assert!(
            !exclusion.reason.trim().is_empty(),
            "Excluded Admin route must have an explicit reason: {}",
            exclusion.path
        );
        let normalized = normalize_admin_route_path(&exclusion.path);
        let previous = routes.insert(normalized, exclusion.reason);
        assert!(
            previous.is_none(),
            "Duplicate excluded Admin route path: {}",
            exclusion.path
        );
    }
    routes
}

#[derive(Debug)]
struct ImplementedAdminRoute {
    module: &'static str,
    path: String,
}

fn extract_admin_route_literals(module: &'static str, source: &str) -> Vec<ImplementedAdminRoute> {
    const ROUTE_CALL: &str = ".route(";

    let mut routes = Vec::new();
    let mut search_start = 0;
    while let Some(relative_index) = source[search_start..].find(ROUTE_CALL) {
        let route_arg_start = search_start + relative_index + ROUTE_CALL.len();
        let route_arg = source[route_arg_start..].trim_start();

        if let Some(after_quote) = route_arg.strip_prefix('"') {
            if let Some(end_quote) = after_quote.find('"') {
                let path = &after_quote[..end_quote];
                if path.starts_with("/admin/v1/") {
                    routes.push(ImplementedAdminRoute {
                        module,
                        path: path.to_owned(),
                    });
                }
            }
        }

        search_start = route_arg_start;
    }

    routes
}
