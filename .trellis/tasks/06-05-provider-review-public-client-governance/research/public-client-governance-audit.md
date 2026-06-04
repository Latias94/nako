# Public Client Governance Audit

## Question

What guardrails already prevent provider governance and Metadata Candidate
Review routes from leaking into Public Client contracts, and what is still
missing after the related hierarchy Admin surface shipped?

## Findings

- `crates/nako-client-protocol/src/lib.rs` owns `PUBLIC_CLIENT_ROUTES` and has
  route inventory tests for public routes and generic internal/secret terms.
  It does not directly name provider governance, Candidate Review, batch apply,
  or related hierarchy route fragments.
- `crates/nako-api/src/lib.rs` defines
  `PROVIDER_GOVERNANCE_PUBLIC_FORBIDDEN_TERMS` for Public OpenAPI and generated
  SDK tests. It already covers Candidate Review and Provider Mapping terms, but
  it does not explicitly name related hierarchy application terms.
- `crates/nako-api/src/openapi.rs` has
  `public_openapi_excludes_provider_governance_routes_and_types`. It explicitly
  excludes existing Candidate Review routes through root apply, but its explicit
  excluded-path list is missing the related hierarchy plan/apply routes added by
  the Admin hierarchy task.
- `crates/nako-api/src/admin_contract.rs` includes the related hierarchy Admin
  route constants and already tests Admin route shapes against Public Client
  route inventory.
- `crates/nako-server/src/http/tests/system.rs` includes
  `admin_v1_metadata_candidate_review_related_hierarchy_routes_reject_non_admin_session`,
  so server-side Admin-only auth for the new routes is covered.

## Conclusion

The implementation should stay in contract/test guardrails:

- add direct provider governance forbidden-route coverage to
  `nako-client-protocol`;
- extend the shared provider-governance forbidden terms with related hierarchy
  terms;
- add related hierarchy paths to the Public OpenAPI explicit exclusion list.

No behavior or API surface should change.
