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
