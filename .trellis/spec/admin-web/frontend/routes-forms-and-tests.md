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
- Generated Admin API contract output must be refreshed with
  `npm run generate:admin-api`; do not edit `src/adminApi/generated/contract.ts`
  by hand.
- Admin Web keeps deterministic mock fallback data for unavailable live reads.
- Mutations are enabled only when the data source is live and the mutation method
  is available.
- Settings routes that use full-replacement `PUT` requests must keep a complete
  typed payload draft and submit the whole request object after confirmation.
  Do not send only the edited fields from a Settings page unless the backend
  DTO is explicitly a patch request.
- Sensitive tokens stay in memory. Do not add build-time admin tokens or render
  bearer tokens into page text.

### 4. Validation & Error Matrix

| Condition | Current behavior |
|-----------|------------------|
| Live read method is unavailable | Return mock value with `source: "mock"` and visible fallback error |
| Live read returns HTTP failure | Data source surfaces mock fallback and error text |
| Mutation page is not live | Disable save action or throw a visible not-live error |
| Mutation requires confirmation | First click prepares/opens confirmation; second explicit confirm calls data source |
| URL filter changes | Update search params and reset `offset` to `0` |
| Media connection token entered | Store in session state only; tests assert token is not rendered |

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
    if (result.source !== "live" || !dataSource.updateMetadataRawCacheSettings) {
      throw new Error("Mutation unavailable");
    }
    return dataSource.updateMetadataRawCacheSettings(request);
  },
});
```

Keep network behavior behind the typed data source and generated Admin API
client, then assert it through route tests.

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
