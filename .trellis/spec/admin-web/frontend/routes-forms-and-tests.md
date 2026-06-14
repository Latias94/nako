# Routes, Forms, Data, and Tests

Use this spec for `apps/admin-web` changes. It records current patterns only.

## Scenario: Admin Web Feature Slice

### 1. Scope / Trigger

- Trigger: route, page, filter, form, data-source, generated Admin API contract,
  or Admin Web test changes.
- Evidence: `README.md`, `package.json`, `src/App.tsx`,
  `src/features/jobs/JobsPage.tsx`, `src/features/settings/SettingsPage.tsx`,
  `src/surfaces/media/MediaPages.tsx`, `src/adminApi/client.ts`,
  `src/adminApi/dataSource.ts`, and route tests.
- Authority: ADR 0027 and ADR 0053.

### 2. Signatures

- Routes are declared in `App.tsx` with TanStack Router:
  `createRoute`, optional `validateSearch`, and route components that call
  `route.useRouteContext()`, `route.useSearch()`, and `route.useNavigate()`.
- URL-owned page props follow:
  `search: <RouteSearch>` and `onSearchChange(next: Partial<RouteSearch>): void`.
- Route-owned filters call `navigate({ search: current => normalize...(...) })`.
- Admin pages load data with `useQuery`; mutations use `useMutation`.
- Native forms use controlled React state and `FormEvent<HTMLFormElement>`.

### 3. Contracts

- Do not introduce `react-hook-form`, `zod`, or another form stack unless the
  dependency already exists and code examples support it. Current dependencies
  do not include those libraries.
- Filter controls use `FilterBar`, `FilterField`, native `input`/`select`, and
  accessible `aria-label` text from i18n.
- Filter updates reset `offset` to `0` when the filter changes.
- Search params are normalized in `App.tsx` with helpers such as
  `positiveIntSearch`, `nonNegativeIntSearch`, `stringSearch`, and
  `emptyToUndefined`.
- Admin API live calls go through `AdminApiClient`, `NAKO_ADMIN_ROUTES`, and the
  `AdminDataSource` mapping layer. Pages should not call `fetch` directly.
- Generated Admin API route path parameters use brace templates such as
  `{addon_id}`. Client code should encode them through the shared
  `routeWithParam` helper; do not add colon-template helpers such as
  `addonPath`.
- Generated Admin API contract output must be refreshed with
  `npm run generate:admin-api`; do not edit `src/adminApi/generated/contract.ts`
  by hand.
- Admin Web keeps deterministic mock fallback data for unavailable live reads.
- Mutations are enabled only when the data source is live and the mutation method
  is available.
- New mutation pages should derive write availability with the shared
  `isLiveDataSource` / `isLiveSectionResult` helpers from
  `src/adminApi/dataSource.ts`, and mutation functions should use the matching
  `requireLiveDataSource` / `requireLiveSectionResult` helper as the final
  guard. This keeps mock and hybrid read fallback from authorizing real writes.
- Settings routes that use full-replacement `PUT` requests must keep a complete
  typed payload draft and submit the whole request object after confirmation.
  Do not send only the edited fields from a Settings page unless the backend
  DTO is explicitly a patch request.
- Sensitive tokens stay in memory. Do not add build-time admin tokens or render
  bearer tokens into page text.
- Media Web read sections that render `MediaLoadResult.error` must replace
  source/backend error strings with route-safe static copy before passing the
  result into shared render helpers. Public Client and fixture errors can
  contain source IDs, stream paths, ticket tokens, bearer tokens, fingerprints,
  or raw backend details.

### 4. Validation & Error Matrix

| Condition | Current behavior |
|-----------|------------------|
| Live read method is unavailable | Return mock value with `source: "mock"` and visible fallback error |
| Live read returns HTTP failure | Data source surfaces mock fallback and error text |
| Mutation page is not live | Disable prepare/save/confirm action and throw a visible not-live error if mutation is invoked programmatically |
| Mutation requires confirmation | First click prepares/opens confirmation; second explicit confirm calls data source |
| URL filter changes | Update search params and reset `offset` to `0` |
| Media connection token entered | Store in session state only; tests assert token is not rendered |
| Media Web read returns or throws unsafe error text | Page maps it to safe static copy before rendering |

### 5. Good / Base / Bad Cases

- Good: define route + search normalization in `App.tsx`, pass search props to a
  route-owned page, use controlled native fields, load through `AdminDataSource`,
  preserve mock fallback, and add RTL tests for URL, calls, fallback, and i18n.
- Base: read-only filter route like Jobs, Playback Sessions, Storage Staging, or
  Catalog Governance.
- Bad: page-level `fetch`, hidden global form state, untyped route strings for
  Admin API calls, new form libraries, or broad product UX in this validation app.

### 6. Tests Required

- Use Vitest and React Testing Library:
  `render(<App dataSource={...} />)`, `screen`, `fireEvent`, and `waitFor`.
- Set route state with `window.history.pushState(null, "", "/route?...")`.
- Assert:
  - URL search params after filter changes.
  - data-source calls and payloads.
  - full-replacement Settings mutations submit the complete typed payload, not
    only field deltas.
  - localized copy for `initialLocale="zh-Hans"` when text changes.
  - mock fallback visibility for unavailable live reads.
  - mock or hybrid fallback mutation controls are disabled and the mutation data
    source method is not called.
  - Media Web read errors that include paths, ticket tokens, bearer tokens,
    fingerprints, or backend details are not rendered verbatim.
  - unsafe fields/secrets are not rendered.
- Commands:
  - `npm run check --prefix apps/admin-web`
  - `npm run test --prefix apps/admin-web`
  - `npm run build --prefix apps/admin-web`
  - `npm run verify --prefix apps/admin-web` for full Admin Web validation.

### 7. Wrong vs Correct

#### Wrong

```tsx
async function save() {
  await fetch("/admin/v1/settings/metadata/raw-cache", { method: "PUT" });
}
```

#### Correct

```tsx
const mutation = useMutation({
  mutationFn: async () => {
    requireLiveSectionResult(result, t("settings.rawCache.notLiveError"));

    if (!dataSource.updateMetadataRawCacheSettings) {
      throw new Error(t("settings.rawCache.updateUnavailable"));
    }

    return dataSource.updateMetadataRawCacheSettings(request);
  },
});
```

Keep network behavior behind the typed data source and generated Admin API
client, then assert it through route tests.

## Scenario: Playback Support Evidence Route

### 1. Scope / Trigger

- Trigger: Admin Web renders `/playback/support`, maps URL search params into
  playback support evidence queries, or changes the redaction-safe runtime and
  subject summaries for this route.
- Evidence: `src/features/playback/PlaybackSupportPage.tsx`,
  `src/features/playback/playbackSupportFormatters.ts`,
  `src/adminApi/dataSource.ts`, `src/App.tsx`, and route tests.
- Authority: ADR 0027 and ADR 0053.

### 2. Signatures

- Route path: `/playback/support`.
- Search keys: `session_id?: string` and `source_id?: string`.
- Data source method:
  `loadPlaybackSupport(query?: AdminPlaybackSupportQuery):
  Promise<AdminSectionResult<AdminPlaybackSupportEvidenceResponse>>`.
- Page props:
  `dataSource: AdminDataSource` and `search: PlaybackSupportSearch`.

### 3. Contracts

- `session_id` and `source_id` are independently optional. The route may be
  opened with one, both, or neither value.
- When both search keys are missing, render a visible direct-entry `RouteNotice`
  that tells the operator to open the page from an item or playback session
  context.
- Keep the page read-only. The route may refresh evidence, but it must not add
  mutation controls or expose raw paths, tokens, command lines, backend URLs,
  or credentials.
- Keep long playback support formatting in a feature-local helper module such as
  `src/features/playback/playbackSupportFormatters.ts` instead of duplicating
  helper logic inside the route component.
- The rendered panels should remain limited to safe subject, session, source,
  runtime, and redaction summaries.

### 4. Validation & Error Matrix

| Condition | Current behavior |
|-----------|------------------|
| Both search keys are missing | Show the direct-entry notice and still render safe fallback/live evidence |
| One or both search keys are present | Forward the search unchanged to `loadPlaybackSupport` |
| Live read is unavailable or fails | Return deterministic mock fallback with a visible route notice |
| Generated support evidence contains raw paths, tokens, command lines, URLs, or credentials | Projection omits them and tests reject rendering |

### 5. Good / Base / Bad Cases

- Good: `PlaybackSupportPage` forwards the route search, shows the
  direct-entry notice only when the page is opened without subject context, and
  uses a feature-local formatter helper for long safe summaries.
- Base: route-owned evidence page opened from item detail with only
  `source_id`, or from playback sessions with both `session_id` and
  `source_id`.
- Bad: hiding the page when one search key is missing, rendering unsafe source
  material, or keeping a large pile of inline formatter helpers inside the
  route component.

### 6. Tests Required

- Route tests assert URL search forwarding into `loadPlaybackSupport`.
- Route tests assert the direct-entry notice appears for `/playback/support`
  with no search keys.
- Route tests assert zh-Hans copy for the direct-entry notice.
- Route tests assert unsafe fields like request fingerprints and raw file
  names do not appear in rendered text.
- Data source tests assert route-local playback support fallback and query
  forwarding.
- Run:
  - `npm run check --prefix apps/admin-web`
  - `npm run test --prefix apps/admin-web`

### 7. Wrong vs Correct

#### Wrong

```tsx
if (!search.session_id && !search.source_id) {
  return <EmptyRouteState>No context</EmptyRouteState>;
}
```

#### Correct

```tsx
{!hasDirectAccessHint ? null : (
  <RouteNotice>{t("playbackSupport.directAccessNotice")}</RouteNotice>
)}
```

Keep playback support evidence route-owned, redaction-safe, and easy to enter
from the surrounding item or session flows.

## Scenario: Addon Task Run Operator Projection

### 1. Scope / Trigger

- Trigger: Admin Web renders, lists, details, or retries Addon Task Runs through
  generated Admin API routes.
- Evidence: `src/features/addons/AddonsPage.tsx`, `src/adminApi/client.ts`,
  `src/adminApi/dataSource.ts`, `src/adminApi/types.ts`,
  `src/adminApi/mockData.ts`, and route tests.
- Authority: ADR 0027 and ADR 0053.

### 2. Signatures

- Generated route keys:
  `addonTaskRuns`, `addonTaskRun`, and `addonTaskRunRetry`.
- Client methods:
  `getAddonTaskRuns(addonId, query)`,
  `getAddonTaskRun(addonId, jobId)`, and
  `retryAddonTaskRun(addonId, jobId, request)`.
- Data source methods:
  `loadAddonTaskRuns(addonId, query)`,
  `loadAddonTaskRun(addonId, jobId)`, and
  `retryAddonTaskRun(addonId, jobId)`.
- Page rows use `AddonTaskRunRow`, not the generated raw
  `AddonTaskRunSummary`.

### 3. Contracts

- Admin Web may receive `AddonTaskRunSummary` fields such as `progress`,
  `result`, `declaration_path`, and `manifest_fingerprint` from the generated
  contract, but must not render or pass those fields to route components.
- `AddonTaskRunRow` may expose only job ID, addon ID, declaration ID/name,
  status, resource class, library/source scope IDs, attempt counters,
  retry linkage, retryability, `hasInput`, safe error code, and timestamps.
- Addons route summary may show a bounded recent task-run panel for the selected
  Addon. It is not a generic scheduler UI.
- Retry is available only when the route load source is `live`, the data source
  exposes `retryAddonTaskRun`, and the row is `status === "failed"` with
  `retryable === true`.
- Retry requires an explicit prepare step followed by a second confirm action.
- Mock fallback may show safe read rows, but must never fabricate a successful
  retry mutation.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Live task-run list succeeds | Map generated response into `AddonTaskRunRow[]` and render safe facts |
| Live task-run read fails | Use deterministic safe mock fallback with visible error |
| Route source is mock or hybrid without mutation | Disable retry and show unavailable copy |
| Row is not failed or not retryable | Render a no-retry state, not an action |
| Retry confirm succeeds | Show queued retry job ID/status and invalidate the Addons query |
| Retry HTTP request fails | Surface the error without changing the task-run list to success |
| Generated response contains raw payload, progress/result, URLs, paths, tokens, fingerprints, or declaration paths | Data source/page projection omits them and tests reject rendering |

### 5. Good / Base / Bad Cases

- Good: client builds generated route paths with encoded `addon_id` and
  `job_id`, data source maps to `AddonTaskRunRow`, page renders safe row facts,
  and tests cover confirmation plus redaction.
- Base: read-only task-run list panel with disabled retry when source is not
  live.
- Bad: page imports `AddonTaskRunSummary`, renders `progress` or `result`, calls
  generated routes directly, or lets mock fallback report a successful retry.

### 6. Tests Required

- Client tests assert generated list/detail/retry routes, encoded path params,
  query params, and POST body.
- Data source tests assert safe mapping, fallback reads, and retry rejection on
  live HTTP failure.
- Route tests assert the panel renders, retry needs explicit confirmation, mock
  retry is disabled, and unsafe task-run fields/secrets are absent from
  rendered text.
- Run:
  - `npm run check --prefix apps/admin-web`
  - focused Vitest files for `client`, `dataSource`, and `App`
  - `npm run test --prefix apps/admin-web`

### 7. Wrong vs Correct

#### Wrong

```tsx
run.progress ? <pre>{JSON.stringify(run.progress)}</pre> : null;
```

#### Correct

```tsx
<span>{taskRunInput(run, t)}</span>
```

Keep Addon Task Run pages on the route-local safe projection, not the raw
generated wire DTO.

## Scenario: Addon Event Delivery Operator Projection

### 1. Scope / Trigger

- Trigger: Admin Web lists outbox events, inspects Addon Event Delivery
  attempts/scheduler work, or runs Addon event deliver/replay mutations through
  generated Admin API routes.
- Evidence: `src/features/events/EventsPage.tsx`,
  `src/adminApi/client.ts`, `src/adminApi/dataSource.ts`,
  `src/adminApi/types.ts`, `src/adminApi/mockData.ts`, generated Admin API
  contracts, and route tests.
- Authority: ADR 0027 and ADR 0053.

### 2. Signatures

- Generated route keys:
  `events`, `eventAddonDeliveryAttempts`, `eventAddonSchedulerWork`,
  `eventAddonDeliver`, and `eventAddonReplay`.
- Client methods:
  `getEvents(query)`, `getAddonEventDeliveryAttempts(eventId)`,
  `getAddonEventSchedulerWork(eventId)`, `deliverAddonEvents(eventId)`, and
  `replayAddonEvents(eventId, request)`.
- Data source methods:
  `loadEvents(query)`, `loadAddonEventDeliveryAttempts(eventId)`,
  `loadAddonEventSchedulerWork(eventId)`, `deliverAddonEvents(eventId)`, and
  `replayAddonEvents(eventId, reasonCode)`.
- Page rows use `EventRow`, `EventDeliveryAttemptRow`, and
  `EventSchedulerWorkRow`, not generated raw Addon Event DTOs.

### 3. Contracts

- The Admin API contract must expose Addon delivery attempts as summaries with
  `has_error`, not raw `error`, and dispatch responses with `error_count`, not
  raw `errors`.
- Event list rows may expose only event ID, kind, status, attempt count,
  `hasPayload`, `hasError`, Library/Source IDs, and timestamps.
- Addon delivery attempt rows may expose only Addon ID, declaration ID, attempt
  number, status, HTTP status, `hasError`, replay reason code, replay flag, and
  timestamps.
- Scheduler work rows may expose only Addon ID, manifest ID/version,
  declaration ID, event kind, status, safe reason code, routing plan
  status/target, attempt counters, latest status/HTTP status, and retry/lease
  timestamps.
- Pages must not render raw event subject, payload, idempotency key, raw error,
  dispatch errors, request/response bodies, URLs, tokens, credentials, local
  paths, fingerprints, manifest raw payload, or Addon sidecar internals.
- Deliver and replay are available only when the event list source is `live`
  and the relevant data-source mutation exists.
- Replay requires a non-empty operator reason code and an explicit prepare step
  followed by confirm.
- Mock fallback may show safe read rows but must never fabricate successful
  deliver or replay mutations.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Live event list succeeds | Map generated response into `EventRow[]` and render safe facts |
| Live attempts/work read fails | Use deterministic safe fallback with visible error |
| Route source is mock or hybrid without mutation | Disable deliver/replay and show unavailable copy |
| Replay reason code is empty | Do not call mutation; show reason-required copy |
| Replay prepare is clicked with a reason | Store only selected event ID and reason code, then render confirm |
| Deliver/replay HTTP request fails | Surface the error without changing visible rows to success |
| Generated response or route data contains raw payload/error/url/token/path/fingerprint material | Data source/page projection omits it and tests reject rendering |

### 5. Good / Base / Bad Cases

- Good: client builds generated route paths with encoded `event_id`, data
  source maps into safe Event rows, page renders only counts/booleans/safe
  codes, and tests cover replay confirmation plus redaction.
- Base: read-only Events route with disabled deliver/replay when source is not
  live.
- Bad: page imports `AddonEventDeliveryAttemptSummary`, renders raw subject or
  payload, calls generated routes directly, or lets mock fallback report a
  successful deliver/replay.

### 6. Tests Required

- Client tests assert generated event list/attempts/work/deliver/replay routes,
  encoded `event_id`, query params, and POST bodies.
- Data source tests assert safe mapping, fallback reads, live mutation failure
  behavior, and raw field redaction.
- Route tests assert `/events` renders, URL-owned pagination/filtering,
  zh-Hans copy, mock mutation disabled state, deliver live-only behavior,
  replay prepare/confirm with reason code, and unsafe fields absent from
  rendered text.
- Run:
  - `npm run check --prefix apps/admin-web`
  - focused Vitest files for `client`, `dataSource`, and `App`
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  - `cargo nextest run -p nako-server addon_event --no-fail-fast`

### 7. Wrong vs Correct

#### Wrong

```tsx
attempt.error ? <code>{attempt.error}</code> : null;
```

#### Correct

```tsx
<Badge tone={attempt.hasError ? "danger" : "neutral"}>
  {attempt.hasError ? t("events.error.present") : t("events.error.absent")}
</Badge>
```

Keep Addon Event Delivery pages on route-local safe projections. Raw delivery
error material may exist in storage for retry diagnostics, but it must not
enter Admin Web route rendering.

## Scenario: Access Invitation Operator Projection

### 1. Scope / Trigger

- Trigger: Admin Web lists, creates, or revokes Access Invitations through
  generated Admin API routes.
- Evidence: `src/features/access/AccessPage.tsx`, `src/adminApi/client.ts`,
  `src/adminApi/dataSource.ts`, `src/adminApi/types.ts`,
  `src/adminApi/mockData.ts`, generated Admin API contracts, and route tests.
- Authority: ADR 0027 and ADR 0053.

### 2. Signatures

- Generated route keys:
  `accessInvitations` and `accessInvitationRevoke`.
- Client methods:
  `getAccessInvitations(query)`, `createAccessInvitation(request)`, and
  `revokeAccessInvitation(invitationId)`.
- Data source methods:
  `loadAccessInvitations(query)`, `createAccessInvitation(input)`, and
  `revokeAccessInvitation(invitationId)`.
- Page rows use `AccessInvitationRow`, not the generated raw
  `AdminInvitationRecord`.
- Create results use `AccessInvitationCreateResult`, where the raw
  one-time token may appear only as `rawToken`.

### 3. Contracts

- Access Invitations are an invitation-first operator workflow. Do not expand
  this projection into Jellyfin-style full user creation, password reset,
  session, lockout, or policy editing without a dedicated task.
- List/read projection may expose only invitation ID, creator/redeemer user
  IDs, recipient label, status, roles, timestamps, and page facts.
- List/read projection must not expose raw tokens, token hashes, local paths,
  backend URLs, credentials, source URIs, fingerprints, arbitrary raw payloads,
  or generated DTO fields that are not explicitly mapped into
  `AccessInvitationRow`.
- The raw invitation token is one-time material from create only. Keep it in
  page state from the create mutation result and never add it to list rows,
  mock summaries, query cache rows, or durable frontend storage.
- Revoke is available only when the invitation list source is `live`, the data
  source exposes `revokeAccessInvitation`, and the row is `status === "pending"`.
- Create is available only when the invitation list source is `live` and the
  data source exposes `createAccessInvitation`.
- Revoke requires an explicit prepare step followed by a second confirm action.
- Mock fallback may show deterministic safe read rows, but must never fabricate
  a successful create or revoke mutation.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Live invitation list succeeds | Map generated response into `AccessInvitationRow[]` and render safe facts |
| Live invitation list fails or method is unavailable | Show deterministic safe mock fallback with visible fallback copy |
| Generated list row contains token, token hash, path, URL, credential, source URI, fingerprint, or raw payload material | Data source/page projection omits it and tests reject rendering |
| Create confirm succeeds | Show the one-time raw token only from the mutation result and invalidate/refetch the invitation list |
| Create HTTP request fails | Surface the error without adding a fake invitation row or token |
| Route source is mock or hybrid without mutation | Disable create/revoke and show unavailable copy |
| Revoke prepare is clicked | Store only the candidate invitation ID and render the confirm action |
| Revoke confirm succeeds | Show the returned safe row status and invalidate/refetch the invitation list |
| Revoke HTTP request fails | Surface the error without changing the visible list to revoked |

### 5. Good / Base / Bad Cases

- Good: client builds generated paths with encoded `invitation_id`, data source
  maps to `AccessInvitationRow`, page renders safe invitation facts, create
  keeps the one-time token in local mutation state, and revoke requires
  prepare/confirm.
- Base: read-only invitation list panel with disabled mutations when source is
  not live.
- Bad: page imports `AdminInvitationRecord`, renders token hash or raw token
  from a list response, calls generated routes directly, or lets mock fallback
  report a successful create/revoke.

### 6. Tests Required

- Client tests assert generated list/create/revoke routes, encoded
  `invitation_id` path params, query params, and POST bodies.
- Data source tests assert safe row mapping, fallback reads, create token
  isolation, and mutation rejection on live HTTP failure.
- Route tests assert the panel renders, create/revoke are disabled for mock
  fallback, revoke needs explicit confirmation, zh-Hans copy is present, and
  unsafe invitation fields/secrets are absent from rendered text.
- Run:
  - `npm run check --prefix apps/admin-web`
  - focused Vitest files for `client`, `dataSource`, and `App`
  - `npm run test --prefix apps/admin-web`
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  - `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`

### 7. Wrong vs Correct

#### Wrong

```tsx
<code>{invitation.token_hash}</code>
```

#### Correct

```tsx
<span>{invitationStatusLabel(invitation.status, t)}</span>
```

Keep Access Invitation pages on the route-local safe projection. Raw one-time
tokens are create-result material only, not list/read row material.

## Scenario: Managed Artwork Maintenance Operator Projection

### 1. Scope / Trigger

- Trigger: Admin Web renders Managed Artwork lifecycle, storage drift, or
  remediation diagnostics through generated Admin API routes.
- Evidence: `src/features/artwork/ManagedArtworkMaintenancePage.tsx`,
  `src/adminApi/client.ts`, `src/adminApi/dataSource.ts`,
  `src/adminApi/types.ts`, `src/adminApi/mockData.ts`, generated Admin API
  contracts, and route tests.
- Authority: ADR 0027 and ADR 0053.

### 2. Signatures

- Generated route keys:
  `managedArtworkArtifactLifecycle`,
  `managedArtworkArtifactStorageDrift`, and
  `managedArtworkArtifactRemediationPlan`.
- Client methods:
  `getManagedArtworkArtifactLifecycle(query)`,
  `getManagedArtworkArtifactStorageDrift(query)`, and
  `getManagedArtworkArtifactRemediationPlan(query)`.
- Data source methods:
  `loadManagedArtworkMaintenance(lifecycleQuery, storageDriftQuery,
  remediationPlanQuery)`, plus the three single-diagnostic loaders.
- Page route:
  `/artwork/maintenance` with URL-owned `limit`, `offset`,
  `cleanup_candidates_only`, and `file_scan_limit`.
- Page rows use `ManagedArtworkMaintenanceSummary` and its route-local row
  types, not generated raw wire DTOs.

### 3. Contracts

- This route is read-only. Do not add accept, ingest process/requeue, publish,
  cleanup, delete, or stray-file remediation controls to the page without a
  dedicated confirmation and policy task. A typed Admin API client method for a
  separately specified confirmed mutation does not by itself make this page a
  mutation workflow.
- The lifecycle projection may expose only artifact ID, ingest ID, Media
  Library ID, Media Item ID, artwork kind, selected count, cleanup-candidate
  boolean, dimensions, byte count, media type, hash-presence boolean, and
  timestamps.
- Storage drift and remediation rows may expose only safe issue/reason/action
  enum codes, recognized artifact ID, extension, byte count, Media Library ID,
  Media Item ID, artwork kind, cleanup-candidate boolean, and selected count.
- Pages must not render raw file names, local paths, artifact roots,
  `storage_uri`, `managed-artwork://` handles, source/cache URIs, provider URLs,
  query strings, tokens, credentials, content hashes, etags, or raw backend
  payloads.
- Mock fallback may show deterministic safe read rows. It must not fabricate
  cleanup/remediation mutation success.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Live diagnostics succeed | Map all three generated responses into `ManagedArtworkMaintenanceSummary` |
| One or more live reads fail | Use deterministic safe fallback with visible fallback copy and combined source state |
| URL filter changes | Write route search params and reset `offset` to `0` for filters that change result membership |
| Generated response contains paths, URIs, provider URLs, tokens, hashes, file names, or root paths | Data source/page projection omits them and tests reject rendering |
| Operator wants cleanup/delete/remediation controls | Open a separate mutation UI task with explicit confirmation; keep this route read-only |

### 5. Good / Base / Bad Cases

- Good: generated route constants feed the client, data source maps into safe
  route-local rows, page renders only counts/booleans/enums/IDs/sizes/times,
  and tests cover URL filters, fallback, localization, and redaction.
- Base: read-only diagnostic surface comparable to Storage Staging or Events
  reads.
- Bad: page imports generated wire DTOs, renders `storage_uri` or a filesystem
  path, calls generated routes directly, or adds a cleanup button to this
  read-only slice.

### 6. Tests Required

- Client tests assert all three generated routes and query params.
- Data source tests assert safe mapping, deterministic fallback, and raw field
  redaction.
- Route tests assert `/artwork/maintenance`, URL-owned filters, zh-Hans copy,
  fallback, and unsafe fields absent from rendered text.
- Run:
  - `npm run check --prefix apps/admin-web`
  - focused Vitest files for `client`, `dataSource`, and `App`
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  - `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`

## Scenario: Managed Artwork Candidate Accept Client Command

### 1. Scope / Trigger

- Trigger: changing `AdminApiClient` support for
  `POST /admin/v1/artwork/candidates/{candidate_id}/accept`.
- Evidence: `src/adminApi/client.ts`, `src/adminApi/client.test.ts`, and
  generated Admin API contracts.
- Authority: ADR 0027, ADR 0053, and the `nako-api` Managed Artwork candidate
  accept contract.

### 2. Signatures

- Generated route key: `managedArtworkCandidateAccept`.
- Client method: `acceptManagedArtworkCandidate(candidateId)`.
- Path parameter: `candidate_id`, URL-encoded through `routeWithParam`.
- Request body: empty JSON object `{}`.
- Response: `AcceptManagedArtworkCandidateResponse`.

### 3. Contracts

- Client code must build the URL from `NAKO_ADMIN_ROUTES`; do not add literal
  `/admin/v1/artwork/candidates/{candidate_id}/accept` strings.
- `candidate_id` is the only caller-supplied input and must be encoded as a
  path parameter.
- The body stays `{}` so Admin Web never accepts provider URLs, raw file names,
  local paths, storage URIs, artifact roots, content hashes, tokens, etags, or
  backend payloads from the caller.
- This method is a low-level generated client command. Route/page controls that
  invoke it still require a dedicated live-only workflow task and should not be
  added to the read-only maintenance page by this contract alone.
- Tests must assert unsafe response material is not present in client fixtures.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| `candidateId` contains reserved URL characters | Client URL-encodes the path parameter |
| Mutation request is sent | Client uses `POST` with `JSON.stringify({})` |
| Generated route key is missing | TypeScript check fails until the generated contract is refreshed |
| Client fixture contains path, URI, token, raw hash, file name, or storage handle material | Treat as a redaction failure |
| A page wants to call this method | Add a dedicated live-only workflow task before wiring UI controls |

### 5. Good / Base / Bad Cases

- Good: `AdminApiClient` calls the generated route with
  `candidate%2Funsafe%20id` and an empty body.
- Base: read-only maintenance diagnostics remain read-only while candidate
  accept is available as a typed low-level client command.
- Bad: passing `{ image_url }`, `{ storage_uri }`, or `{ artifact_id }` in the
  mutation body, or putting an accept button into a read-only route without a
  dedicated workflow task.

### 6. Tests Required

- Client tests assert generated route usage, path parameter encoding, `POST`,
  empty body, response typing, and redaction fixture terms.
- Run:
  - `npm run test --prefix apps/admin-web -- adminApi/client.test.ts`
  - `npm run check --prefix apps/admin-web`
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast` when generated
    contract output changes.

### 7. Wrong vs Correct

#### Wrong

```typescript
return this.postJson(NAKO_ADMIN_ROUTES.managedArtworkCandidateAccept, {
  image_url,
});
```

#### Correct

```typescript
return this.postJson(
  routeWithParam(
    NAKO_ADMIN_ROUTES.managedArtworkCandidateAccept,
    "candidate_id",
    candidateId,
  ),
  {},
);
```

The generated route owns the path shape; the client supplies only the opaque
candidate ID and leaves candidate resolution plus ingest queueing to the server.

## Scenario: Managed Artwork Ingest Requeue Client Command

### 1. Scope / Trigger

- Trigger: changing `AdminApiClient` support for
  `POST /admin/v1/artwork/ingests/{ingest_id}/requeue`.
- Evidence: `src/adminApi/client.ts`, `src/adminApi/client.test.ts`, and
  generated Admin API contracts.
- Authority: ADR 0027, ADR 0053, and the `nako-api` Managed Artwork ingest
  requeue contract.

### 2. Signatures

- Generated route key: `managedArtworkIngestRequeue`.
- Client method: `requeueManagedArtworkIngest(ingestId)`.
- Path parameter: `ingest_id`, URL-encoded through `routeWithParam`.
- Request body: empty JSON object `{}`.
- Response: `RequeueManagedArtworkIngestResponse`.

### 3. Contracts

- Client code must build the URL from `NAKO_ADMIN_ROUTES`; do not add literal
  `/admin/v1/artwork/ingests/{ingest_id}/requeue` strings.
- `ingest_id` is the only caller-supplied input and must be encoded as a path
  parameter.
- The body stays `{}` so Admin Web never accepts provider URLs, raw file names,
  local paths, storage URIs, artifact roots, content hashes, tokens, etags, raw
  job input JSON, summary JSON, errors, or backend payloads from the caller.
- This method is a low-level generated client command. Route/page controls that
  invoke it still require a dedicated live-only workflow task and should not be
  added to the read-only maintenance page by this contract alone.
- Tests must assert unsafe response material is not present in client fixtures.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| `ingestId` contains reserved URL characters | Client URL-encodes the path parameter |
| Mutation request is sent | Client uses `POST` with `JSON.stringify({})` |
| Generated route key is missing | TypeScript check fails until the generated contract is refreshed |
| Client fixture contains path, URI, token, raw hash, file name, storage handle, input JSON, or summary JSON material | Treat as a redaction failure |
| A page wants to call this method | Add a dedicated live-only workflow task before wiring UI controls |

### 5. Good / Base / Bad Cases

- Good: `AdminApiClient` calls the generated route with
  `ingest%2Funsafe%20id` and an empty body.
- Base: read-only maintenance diagnostics remain read-only while ingest requeue
  is available as a typed low-level client command.
- Bad: passing `{ input_json }`, `{ summary_json }`, `{ storage_uri }`, or
  `{ artifact_id }` in the mutation body, or putting a requeue button into a
  read-only route without a dedicated workflow task.

### 6. Tests Required

- Client tests assert generated route usage, path parameter encoding, `POST`,
  empty body, response typing, replay-safe response shape, and redaction fixture
  terms.
- Run:
  - `npm run test --prefix apps/admin-web -- adminApi/client.test.ts`
  - `npm run check --prefix apps/admin-web`
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast` when generated
    contract output changes.

### 7. Wrong vs Correct

#### Wrong

```typescript
return this.postJson(NAKO_ADMIN_ROUTES.managedArtworkIngestRequeue, {
  input_json,
});
```

#### Correct

```typescript
return this.postJson(
  routeWithParam(
    NAKO_ADMIN_ROUTES.managedArtworkIngestRequeue,
    "ingest_id",
    ingestId,
  ),
  {},
);
```

The generated route owns the path shape; the client supplies only the opaque
ingest ID and leaves retry validation plus durable job reset to the server.

## Scenario: Managed Artwork Process-Next Client Command

### 1. Scope / Trigger

- Trigger: changing `AdminApiClient` support for
  `POST /admin/v1/artwork/ingests/process-next`.
- Evidence: `src/adminApi/client.ts`, `src/adminApi/client.test.ts`, and
  generated Admin API contracts.
- Authority: ADR 0027, ADR 0053, and the `nako-api` Managed Artwork
  process-next contract.

### 2. Signatures

- Generated route key: `managedArtworkIngestProcessNext`.
- Client method: `processNextManagedArtworkIngest()`.
- Request body: empty JSON object `{}`.
- Response: `ProcessManagedArtworkIngestResponse`.

### 3. Contracts

- Client code must build the URL from `NAKO_ADMIN_ROUTES`; do not add literal
  `/admin/v1/artwork/ingests/process-next` strings.
- The caller supplies no target ID or payload.
- The body stays `{}` so Admin Web never accepts provider URLs, raw file names,
  local paths, storage URIs, artifact roots, content hashes, tokens, etags, raw
  job input JSON, summary JSON, errors, ingest IDs, candidate IDs, or backend
  payloads from the caller.
- This method is a low-level generated client command. Route/page controls that
  invoke it still require a dedicated live-only workflow task and should not be
  added to the read-only maintenance page by this contract alone.
- Tests must assert unsafe response material is not present in client fixtures
  and must cover `processed: false` empty responses.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Mutation request is sent | Client uses `POST` with `JSON.stringify({})` |
| Generated route key is missing | TypeScript check fails until the generated contract is refreshed |
| Response is empty queue | Client accepts `processed: false` with null `ingest`, `artifact`, and `job` |
| Client fixture contains path, URI, token, raw hash, file name, storage handle, input JSON, summary JSON, candidate ID request, or ingest ID request material | Treat as a redaction failure |
| A page wants to call this method | Add a dedicated live-only workflow task before wiring UI controls |

### 5. Good / Base / Bad Cases

- Good: `AdminApiClient` calls the generated route with an empty body and
  accepts both processed and empty response shapes.
- Base: read-only maintenance diagnostics remain read-only while process-next
  is available as a typed low-level client command.
- Bad: passing `{ ingest_id }`, `{ source_uri }`, `{ input_json }`, or
  `{ storage_uri }` in the mutation body, or putting a process button into a
  read-only route without a dedicated workflow task.

### 6. Tests Required

- Client tests assert generated route usage, `POST`, empty body, processed
  response typing, empty response typing, and redaction fixture terms.
- Run:
  - `npm run test --prefix apps/admin-web -- adminApi/client.test.ts`
  - `npm run check --prefix apps/admin-web`
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast` when generated
    contract output changes.

### 7. Wrong vs Correct

#### Wrong

```typescript
return this.postJson(NAKO_ADMIN_ROUTES.managedArtworkIngestProcessNext, {
  ingest_id,
});
```

#### Correct

```typescript
return this.postJson(NAKO_ADMIN_ROUTES.managedArtworkIngestProcessNext, {});
```

The generated route owns the command shape; the client supplies no queue target
or raw worker payload.

## Scenario: Managed Artwork Stray File Cleanup Client Command

### 1. Scope / Trigger

- Trigger: changing `AdminApiClient` support for
  `POST /admin/v1/artwork/artifacts/remediate-stray-files`.
- Evidence: `src/adminApi/client.ts`, `src/adminApi/client.test.ts`, and
  generated Admin API contracts.
- Authority: ADR 0027, ADR 0053, and the `nako-api` Managed Artwork stray file
  cleanup contract.

### 2. Signatures

- Generated route key:
  `managedArtworkArtifactRemediateStrayFiles`.
- Client method:
  `remediateManagedArtworkArtifactStrayFiles(query)`.
- Query:
  `{ confirm?: boolean; file_scan_limit?: number }`.
- Request body:
  empty JSON object `{}`.
- Response:
  `AdminManagedArtworkArtifactStrayFileCleanupResponse`.

### 3. Contracts

- Client code must build the URL from `NAKO_ADMIN_ROUTES`; do not add literal
  `/admin/v1/artwork/artifacts/remediate-stray-files` strings.
- Confirmation and `file_scan_limit` are query parameters, not body fields.
- The body stays `{}` so Admin Web never accepts raw file names, local paths,
  storage URIs, artifact roots, content hashes, tokens, etags, or backend
  payloads from the caller.
- This method is a low-level generated client command. Route/page controls that
  invoke it still require a dedicated UI task with explicit prepare/confirm
  behavior and live-only mutation availability.
- Tests must assert unsafe response material is not present in client fixtures.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| `confirm` and `file_scan_limit` are supplied | Client serializes them into the query string |
| Mutation request is sent | Client uses `POST` with `JSON.stringify({})` |
| Generated route key is missing | TypeScript check fails until the generated contract is refreshed |
| Client fixture contains path, URI, token, raw hash, or file name material | Treat as a redaction failure |
| A page wants to call this method | Add a dedicated live-only confirmation workflow before wiring UI controls |

### 5. Good / Base / Bad Cases

- Good: `AdminApiClient` calls the generated route with
  `?confirm=true&file_scan_limit=50` and an empty body.
- Base: maintenance page remains read-only and can still render remediation plan
  diagnostics.
- Bad: putting a cleanup button directly into the read-only maintenance page or
  passing `{ storage_uri }` in the mutation body.

### 6. Tests Required

- Client tests assert generated route usage, query serialization, `POST`, empty
  body, and redaction fixture terms.
- Run:
  - `npm run test --prefix apps/admin-web -- adminApi/client.test.ts`
  - `npm run check --prefix apps/admin-web`
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast` when generated
    contract output changes.

### 7. Wrong vs Correct

#### Wrong

```typescript
return this.postJson("/admin/v1/artwork/artifacts/remediate-stray-files", {
  storage_uri,
});
```

#### Correct

```typescript
return this.postJson(
  withQuery(NAKO_ADMIN_ROUTES.managedArtworkArtifactRemediateStrayFiles, query),
  {},
);
```

The route contract is generated from `nako-api`, and the server owns cleanup
target discovery.

## Scenario: Managed Artwork Artifact Cleanup Client Command

### 1. Scope / Trigger

- Trigger: changing `AdminApiClient` support for
  `POST /admin/v1/artwork/artifacts/cleanup`.
- Evidence: `src/adminApi/client.ts`, `src/adminApi/client.test.ts`, and
  generated Admin API contracts.
- Authority: ADR 0027, ADR 0053, and the `nako-api` Managed Artwork artifact
  cleanup contract.

### 2. Signatures

- Generated route key: `managedArtworkArtifactCleanup`.
- Client method: `cleanupManagedArtworkArtifacts(query)`.
- Query: `{ confirm?: boolean; limit?: number; offset?: number }`.
- Request body: empty JSON object `{}`.
- Response: `AdminManagedArtworkArtifactCleanupResponse`.

### 3. Contracts

- Client code must build the URL from `NAKO_ADMIN_ROUTES`; do not add literal
  `/admin/v1/artwork/artifacts/cleanup` strings.
- Confirmation, `limit`, and `offset` are query parameters, not body fields.
- The body stays `{}` so Admin Web never accepts raw artifact IDs, file names,
  local paths, storage URIs, artifact roots, content hashes, tokens, etags, or
  backend payloads from the caller.
- This method is a low-level generated client command. Route/page controls that
  invoke it still require a dedicated UI task with explicit prepare/confirm
  behavior and live-only mutation availability.
- Tests must assert unsafe response material is not present in client fixtures.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| `confirm`, `limit`, and `offset` are supplied | Client serializes them into the query string |
| Mutation request is sent | Client uses `POST` with `JSON.stringify({})` |
| Generated route key is missing | TypeScript check fails until the generated contract is refreshed |
| Client fixture contains path, URI, token, raw hash, or file name material | Treat as a redaction failure |
| A page wants to call this method | Add a dedicated live-only confirmation workflow before wiring UI controls |

### 5. Good / Base / Bad Cases

- Good: `AdminApiClient` calls the generated route with
  `?confirm=true&limit=5&offset=10` and an empty body.
- Base: maintenance page remains read-only and can still render lifecycle
  cleanup candidate diagnostics.
- Bad: passing `{ artifact_id }` or `{ storage_uri }` in the mutation body, or
  wiring cleanup into a read-only route without a second confirm action.

### 6. Tests Required

- Client tests assert generated route usage, query serialization, `POST`, empty
  body, and redaction fixture terms.
- Run:
  - `npm run test --prefix apps/admin-web -- adminApi/client.test.ts`
  - `npm run check --prefix apps/admin-web`
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast` when generated
    contract output changes.

### 7. Wrong vs Correct

#### Wrong

```typescript
return this.postJson(NAKO_ADMIN_ROUTES.managedArtworkArtifactCleanup, {
  artifact_id,
});
```

#### Correct

```typescript
return this.postJson(
  withQuery(NAKO_ADMIN_ROUTES.managedArtworkArtifactCleanup, query),
  {},
);
```

The route contract is generated from `nako-api`, and cleanup target discovery
stays server-owned.

## Scenario: Feature-Owned Data Adapter

### 1. Scope / Trigger

- Trigger: an Admin Web feature page needs only a narrow subset of the broad
  `AdminDataSource`, has feature-specific fallback behavior, or owns a
  confirmation/mutation workflow that should not leak generated route details
  into UI components.
- Evidence:
  `src/features/items/sourceDuplicateReconciliationData.ts` and
  `SourceDuplicateReconciliationPage.tsx`.

### 2. Signatures

- Define a small feature adapter interface next to the feature page, for
  example `SourceDuplicateReconciliationDataAdapter`.
- Route wiring in `App.tsx` may create the adapter from the broad
  `AdminDataSource` and localized messages, then pass the adapter to the page.
- The page depends on the feature adapter, URL-owned search props, and event
  callbacks; it should not import the broad `AdminDataSource` for that feature.

### 3. Contracts

- Feature adapters must delegate live network behavior to existing
  `AdminDataSource` methods. Do not call `fetch` or generated routes directly
  from the adapter unless the broad data-source boundary has deliberately moved.
- Adapter-owned fallback must use existing mock data helpers and preserve
  redaction-safe behavior.
- Mutations that have no safe mock success path must reject when the live
  method is unavailable rather than fabricating success.
- Keep localized copy at the route/page boundary. Pass messages into adapter
  factories instead of importing i18n providers into data modules.
- Use `useMemo` in route components when creating adapters from stable route
  context and locale-dependent messages, so query/mutation props do not churn
  unnecessarily.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Live plan/read method exists | Adapter delegates with the same route args and query payload |
| Live plan/read method missing | Adapter returns deterministic mock fallback with visible error |
| Live mutation method exists | Adapter delegates without changing the command payload |
| Live mutation method missing | Adapter rejects with the localized unavailable message |
| Page renders unsafe extra fields in response | Treat as redaction failure and fix page rendering |

### 5. Good / Base / Bad Cases

- Good: source duplicate reconciliation route creates a feature adapter, page
  calls `loadPlan`/`applySuggestion`, and adapter tests prove fallback and
  unavailable mutation behavior.
- Base: keep one-off simple pages on `AdminDataSource` until a feature has a
  real workflow boundary or repeated mapping logic.
- Bad: broad rewrites of `AdminDataSource`, duplicated generated route strings,
  direct page-level `fetch`, or adapter modules that import React hooks.

### 6. Tests Required

- Add feature adapter tests for delegation, fallback, and mutation unavailable
  behavior.
- Keep route tests for URL search, confirmation, i18n, and redaction.
- Run:
  - `npm run check --prefix apps/admin-web`
  - focused Vitest files for the adapter and affected routes
  - `npm run test --prefix apps/admin-web`
  - `npm run build --prefix apps/admin-web` when route/page code changes.

## Scenario: Route-Level Bundle Splitting

### 1. Scope / Trigger

- Trigger: `apps/admin-web/src/App.tsx`, route shells, i18n providers, Media Web
  session wiring, or any feature route import changes that can alter Vite chunk
  boundaries.
- Evidence: `src/App.tsx`, `src/i18n/I18nProvider.tsx`,
  `src/surfaces/media/MediaSession.tsx`, and Vite build output.

### 2. Signatures

- Route runtime values in `App.tsx` should use `React.lazy` with named export
  mapping against route modules:
  `lazy(() => import("./routes/XRouteModule").then(module => ({ default: module.XRouteModule })))`.
- Route modules live under `src/routes/*RouteModule.tsx` and own the
  route-local page import, `RouteI18n` wrapper, and page prop assembly.
- Route search types remain type-only imports from page modules, for example
  `import type { JobsSearch } from "./features/jobs/JobsPage";`.
- Broad default implementations that pull optional product surfaces, large
  fixtures, SDKs, or locale catalogs should be loaded through dynamic import.
- Route-owned localized pages should be wrapped with `RouteI18n` in their lazy
  route module and declare the `I18nNamespace` catalog prefixes they need.

### 3. Contracts

- Keep TanStack route ownership in `App.tsx`: routes still read context,
  params, search, and navigate helpers, then pass typed props into lazy route
  modules.
- Keep search validation and normalization primitives in `App.tsx` or another
  lightweight shell-owned helper. Route modules may receive `onSearchChange`,
  but they should not import TanStack route instances or duplicate URL
  normalization logic.
- Do not convert a type-only route contract import into a runtime import unless
  that module is intentionally part of the initial Admin shell.
- Feature-owned adapters that are only needed by one route should be created in
  a lazy route wrapper next to the feature page, not imported by `App.tsx`.
- Do not wrap page components with `RouteI18n` directly in `App.tsx`. Declare
  route namespaces in the lazy route module, including multi-namespace routes
  such as `["libraryDetail", "libraries"]` and
  `["playback", "playbackSupport"]`.
- `I18nProvider` may defer full message catalogs, but it must keep
  `MessageId`/`AdminLocale` typing, persist locale selection, and avoid
  rendering untranslated IDs after the catalog is loaded.
- `src/i18n/messages.ts` owns only type composition. Runtime message text stays
  in `src/i18n/catalogs/base.ts` and route/feature catalog modules loaded
  through `src/i18n/catalogLoader.ts`.
- Keep `shell`, `nav`, `source`, and `locale` messages in the base catalog so
  the shell can render before a route catalog loads.
- If a page renders another feature's copy, declare all needed namespaces in
  its route wrapper, for example `["playback", "playbackSupport"]`.
- Media Web default data-source wiring should not statically import the Public
  Client SDK or fixture data into the Admin shell.
- Media Web route modules should keep browse/search/library pages, item detail
  playback state, and watch/browser-player code in separate lazy modules. Do
  not put browser playback tickets, HLS adapter probing, video element handlers,
  or playback progress flushing back into `MediaPages.tsx`.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| A route page is lazy-loaded | Its URL search, params, data source, i18n, and fallback behavior stay unchanged |
| A page exposes only search types to `App.tsx` | Import remains `import type`, and build output keeps the page in a route chunk |
| Base locale catalog is loading | The shell may temporarily render no content, then renders localized copy once loaded |
| Route locale catalog is loading | The shell remains rendered while the route outlet waits for all declared namespaces |
| Media Web fixture/live connection is first used | The media data-source module loads on demand and preserves token redaction |
| Build output moves code into route chunks | Main `index-*.js` shrinks and route/page plus route catalog chunks are emitted for the affected modules |

### 5. Good / Base / Bad Cases

- Good: lazy route module declarations, type-only search imports, a shared
  `Suspense` outlet, dynamic message catalog loading, and build output showing
  route module/page/catalog chunks plus a smaller main chunk.
- Good: route catalog modules emit chunks such as `settings-*.js` and
  `storage-*.js`; there is no single route-agnostic `messages-*.js` chunk that
  contains all localized page copy.
- Good: Media Web browse routes import only `MediaPages.tsx` plus lightweight
  media core helpers; `/media/items/$itemId` loads item playback state; and
  `/media/watch/$itemId` loads browser ticket/player/progress code.
- Base: simple routes can still pass broad `AdminDataSource` into the page when
  they do not need a feature adapter.
- Bad: importing page components, adapter factories, media data-source defaults,
  generated SDK clients, fixtures, full message catalogs, or watch/player-only
  media code directly into the Admin shell or browse route path.

### 6. Tests Required

- Run `npm run check --prefix apps/admin-web`.
- Run affected route tests and `npm run test --prefix apps/admin-web` when i18n
  or shell loading changes.
- Run `npm run build --prefix apps/admin-web` and inspect emitted
  `dist/assets/index-*.js`, route chunks, and route catalog chunks.
- Keep route tests async-aware with `findBy...`/`waitFor` when lazy pages or
  dynamic catalogs are involved.
- Add or keep tests that prove a route catalog loads on demand without
  importing unrelated route catalogs.

### 7. Wrong vs Correct

#### Wrong

```typescript
import { SettingsPage } from "./features/settings/SettingsPage";
import { createMediaWebDataSource } from "./surfaces/media/mediaDataSource";
```

#### Correct

```typescript
const SettingsRouteModule = lazy(() =>
  import("./routes/SettingsRouteModule").then((module) => ({
    default: module.SettingsRouteModule,
  })),
);

const dataSource = await import("./mediaDataSource").then((module) =>
  module.createMediaWebDataSource(connection),
);
```

Protect the Admin shell path from route-local and optional-surface code, then
use the production build output as the source of truth for bundle behavior.
