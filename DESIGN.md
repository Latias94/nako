# Nako Design Context

Last updated: 2026-05-26

## Register

Product UI. Design serves repeated administrative tasks and should stay quiet,
dense, and predictable.

## Surface Scope

This file currently defines the Admin Web product UI baseline. Media Web,
desktop playback clients, and native mobile clients should use separate
surface-specific design context before implementation. They may share Nako
terminology, auth state, route-link patterns, and basic component discipline,
but should not inherit Admin Web's light, dense, operations-first visual model
for watch-first browsing and playback.

## Physical Scene

A self-hosted operator reviews library health, jobs, metadata confidence, and
addon readiness from a desktop or laptop at a normal desk, often while also
checking terminal logs or server config. The baseline theme should be a light,
low-glare admin console with restrained contrast; dark mode can come later for
long diagnostics sessions.

## Visual Strategy

Use a restrained product palette with tinted neutrals, a small number of
semantic roles, and one brand accent. Avoid making the surface read as a
single beige, brown, slate, or teal theme.

Suggested OKLCH roles:

- `surface`: `oklch(97% 0.008 105)`
- `surface-raised`: `oklch(99% 0.006 105)`
- `surface-panel`: `oklch(94.5% 0.012 118)`
- `ink`: `oklch(22% 0.026 84)`
- `muted`: `oklch(47% 0.025 94)`
- `line`: `oklch(84% 0.018 105)`
- `brand`: `oklch(58% 0.11 184)`
- `accent-warm`: `oklch(63% 0.12 58)`
- `success`: `oklch(58% 0.12 148)`
- `warning`: `oklch(66% 0.13 72)`
- `danger`: `oklch(55% 0.16 28)`
- `info`: `oklch(57% 0.12 238)`

The brand accent should identify active navigation, primary actions, focus
states, and selected filters. Warm accent is for attention and review states,
not for every inactive card.

## Typography

- Use a single UI family: Inter or a system font stack.
- Keep body text between 0.875rem and 1rem.
- Use compact but clear hierarchy: section titles around 1.125rem, page titles
  around 1.5rem to 1.75rem.
- Do not use display fonts for admin labels, buttons, tables, badges, or
  diagnostics.
- Keep letter spacing at 0 except small uppercase labels, where it must remain
  subtle.

## Layout

- Use a persistent app shell with primary navigation, content routes, and
  route-level actions.
- Prefer route-first pages over one giant anchor dashboard once workflows need
  filters, pagination, detail views, or mutations.
- Use cards only for repeated records, bounded panels, and local tool surfaces.
  Do not nest cards.
- Tables, filter bars, segmented controls, tabs, and detail panels are first
  class patterns for Admin Web.
- Mobile behavior should collapse navigation and preserve task order; do not
  solve responsiveness through fluid type scaling.

## Components

Every interactive component needs default, hover, focus, active, disabled,
loading, and error states. Baseline components:

- App shell and navigation item.
- Route header with source/readiness status.
- Button, icon button, segmented control, tabs, checkbox, toggle, select,
  input, textarea, and slider or number input where numeric values appear.
- Data table with pagination, filters, empty state, loading skeleton, and
  row-level actions.
- Status badge vocabulary for live, mock, planned, healthy, degraded, failed,
  disabled, ready, missing grant, and unsafe response states.
- Detail panel or detail route for jobs, sessions, catalog governance items,
  addons, generated artifacts, and webhook events.

Use lucide-react icons where an icon exists. Avoid invented symbols for common
actions.

## Implementation Phase

Admin Web V2 should be feature-first. The first implementation phase should
compose pages from shadcn/ui-style primitives, dashboard blocks, tables,
filters, forms, sheets, dialogs, and command/search patterns instead of
spending time on a bespoke Nako design system.

The early UI bar is:

- correct workflow and Admin API behavior;
- safe redaction and truthful live/mock/fallback states;
- accessible standard controls;
- responsive layouts that do not break;
- light Nako theming through tokens, icon, labels, and copy.

Productized UX polish, richer motion, custom visual language, and deeper brand
expression should come after the main admin workflows exist and the API gaps
are visible.

## Data And Safety UX

- Show live/mock/planned status in developer and operator-facing diagnostics,
  but do not let those labels dominate every page.
- Never render plaintext secrets, bearer tokens, webhook secrets, resolved
  provider keys, unsafe local paths, raw provider bodies, or raw request
  headers in ordinary views.
- Use Secret Reference labels and redacted fingerprints where identification
  is necessary.
- Treat Addon Hosted Pages as external and untrusted.
- When a route falls back to mock data, keep the fallback local to that
  section and make the error actionable.

## Motion

- Keep transitions between 150ms and 220ms.
- Use motion for state changes, disclosure, loading skeletons, and focus
  feedback.
- Do not choreograph page-load sequences.
- Avoid animating layout properties.

## Accessibility And Responsiveness

- Preserve keyboard navigation for all actions.
- Use visible focus states.
- Keep table overflow intentional and reachable on small screens.
- Preserve labels for icon-only controls through accessible names and
  tooltips where useful.
- Text must not overlap or overflow fixed controls at mobile or desktop
  widths.
