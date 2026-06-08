# Jellyfin route template comparison

## Question

How do mature media-server control-plane routes represent path parameters, and
how should that influence Nako's generated Admin API contract templates?

## Local Reference Evidence

- `repo-ref/jellyfin/Jellyfin.Api/Controllers/ScheduledTasksController.cs`
  uses controller attributes such as `HttpGet("{taskId}")`,
  `HttpPost("Running/{taskId}")`, and `HttpPost("{taskId}/Triggers")`.
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/RemoteImageController.cs`
  uses routes such as `Items/{itemId}/RemoteImages` and
  `Items/{itemId}/RemoteImages/Download`.
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/PluginsController.cs`
  uses routes such as `{pluginId}/{version}/Enable`,
  `{pluginId}/Configuration`, and `{pluginId}/Manifest`.

## Takeaway

Jellyfin's controller route attributes consistently expose path parameters with
brace-style templates. Nako should not copy Jellyfin implementation code, but
the route-shape convention supports making Nako's generated Admin contract use
one placeholder style: `{param}`.

## Nako Mapping

- Server route declarations may use the syntax required by Axum.
- Generated Admin API contracts should be consumer-facing route templates, not
  Axum implementation details.
- Admin Web already has a generic `routeWithParam(path, name, value)` helper for
  brace-style placeholders. Keeping generated templates in brace style removes
  the `addonPath` exception and makes generated route coverage easier to audit.

## Risks And Tests

- Risk: generated artifact drift if only one TypeScript output is refreshed.
  Test with the existing `admin_web_generated_contract_matches_generator_output`
  contract check.
- Risk: Addon client routes stop encoding IDs correctly.
  Test Admin Web client methods with unsafe Addon IDs and expected encoded
  paths.
- Risk: future generator edits reintroduce colon-style placeholders.
  Add a `nako-api` contract regression test that rejects colon-style parameters
  in generated route paths.
