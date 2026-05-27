import type { CSSProperties, ReactNode } from "react";

import { Link } from "@tanstack/react-router";
import {
  ArrowRight,
  ExternalLink,
  ScanSearch,
} from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/cn";
import { surfaceDefinitions, type SurfaceKey } from "@/lib/navigation";

interface SurfaceShellProps {
  actions?: ReactNode;
  children: ReactNode;
  eyebrow: string;
  status?: string;
  summary: string;
  surface: SurfaceKey;
  title: string;
}

export function SurfaceShell({
  actions,
  children,
  eyebrow,
  status,
  summary,
  surface,
  title,
}: SurfaceShellProps) {
  const definition = surfaceDefinitions[surface];
  const admin = surface === "admin";
  const style = {
    "--app-bg": `var(${definition.bgVar})`,
    "--app-sidebar": `var(${definition.sidebarVar})`,
    "--app-panel": `var(${definition.panelVar})`,
    "--app-panel-soft": admin ? "var(--admin-panel-soft)" : "var(--app-panel-soft)",
    "--app-line": admin ? "var(--admin-line)" : "var(--app-line)",
    "--app-fg": `var(${definition.fgVar})`,
    "--app-muted": admin ? "var(--admin-muted)" : "var(--app-muted)",
    "--app-accent": `var(${definition.accentVar})`,
    "--app-accent-ink": admin ? "var(--admin-accent-ink)" : "var(--app-accent-ink)",
  } as CSSProperties;

  return (
    <div
      style={style}
      className="min-h-screen bg-[color:var(--app-bg)] text-[color:var(--app-fg)]"
    >
      <div className="grid min-h-screen lg:grid-cols-[280px_minmax(0,1fr)]">
        <aside
          style={{
            backgroundColor: `var(${definition.sidebarVar})`,
          }}
          className="flex flex-col gap-5 border-b border-[color:var(--app-line)] px-4 py-4 lg:border-b-0 lg:border-r"
        >
          <div className="flex items-center gap-3 border-b border-[color:var(--app-line)] pb-4">
            <div className="grid h-10 w-10 place-items-center rounded-xl bg-[color:var(--app-accent)] text-sm font-black text-[color:var(--app-accent-ink)]">
              N
            </div>
            <div className="min-w-0">
              <p className="truncate text-sm font-semibold tracking-wide">{definition.title}</p>
              <p className="truncate text-xs text-[color:var(--app-muted)]">{definition.subtitle}</p>
            </div>
          </div>

          <div className="flex flex-wrap gap-2">
            {Object.entries(surfaceDefinitions).map(([key, item]) => {
              const active = key === surface;

              return (
                <Link
                  key={key}
                  to={active ? `/${key}` : `/${key}`}
                  className={cn(
                    "inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-xs font-semibold transition-colors",
                    active
                      ? "border-transparent bg-[color:var(--app-accent)] text-[color:var(--app-accent-ink)]"
                      : "border-[color:var(--app-line)] text-[color:var(--app-fg)] hover:bg-white/5",
                  )}
                >
                  {item.label}
                </Link>
              );
            })}
          </div>

          <nav className="grid gap-1">
            {definition.nav.map((item) => {
              const Icon = item.icon;

              return (
                <Link
                  key={item.to}
                  to={item.to}
                  className="grid grid-cols-[20px_minmax(0,1fr)] gap-3 rounded-lg border border-transparent px-3 py-3 text-left transition-colors hover:border-[color:var(--app-line)] hover:bg-white/5"
                >
                  <Icon className="mt-0.5 h-4 w-4" />
                  <span className="min-w-0">
                    <span className="block text-sm font-semibold">{item.label}</span>
                    <span className="mt-0.5 block text-xs text-[color:var(--app-muted)]">
                      {item.description}
                    </span>
                  </span>
                </Link>
              );
            })}
          </nav>

          <div className="mt-auto grid gap-2 border-t border-[color:var(--app-line)] pt-4">
            <p className="text-xs uppercase tracking-[0.18em] text-[color:var(--app-muted)]">
              Quick Links
            </p>
            <div className="grid gap-1">
              {definition.quickLinks.map((item) => {
                const Icon = item.icon;

                return (
                  <Link
                    key={item.to}
                    to={item.to}
                    className="grid grid-cols-[20px_minmax(0,1fr)] gap-3 rounded-lg border border-[color:var(--app-line)] px-3 py-3 text-left transition-colors hover:bg-white/5"
                  >
                    <Icon className="mt-0.5 h-4 w-4" />
                    <span className="min-w-0">
                      <span className="block text-sm font-semibold">{item.label}</span>
                      <span className="mt-0.5 block text-xs text-[color:var(--app-muted)]">
                        {item.description}
                      </span>
                    </span>
                  </Link>
                );
              })}
            </div>
          </div>
        </aside>

        <main className="min-w-0 p-5 sm:p-6 xl:p-8">
          <div className="flex flex-col gap-5">
            <header className="flex flex-col gap-4 border-b border-[color:var(--app-line)] pb-5 xl:flex-row xl:items-start xl:justify-between">
              <div className="grid gap-2">
                <p className="text-xs font-semibold uppercase tracking-[0.18em] text-[color:var(--app-muted)]">
                  {eyebrow}
                </p>
                <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">{title}</h1>
                <p className="max-w-3xl text-sm leading-6 text-[color:var(--app-muted)]">
                  {summary}
                </p>
              </div>

              <div className="flex flex-wrap items-center gap-2">
                {status ? <Badge className="border-transparent bg-white/10">{status}</Badge> : null}
                {actions}
              </div>
            </header>

            {children}
          </div>
        </main>
      </div>
    </div>
  );
}

export function SectionCard({
  children,
  className,
  title,
  summary,
}: {
  children: ReactNode;
  className?: string;
  summary?: string;
  title: string;
}) {
  return (
    <section
      className={cn(
        "rounded-xl border border-[color:var(--app-line)] bg-[color:var(--app-panel)]",
        className,
      )}
    >
      <div className="border-b border-[color:var(--app-line)] px-4 py-3">
        <h2 className="text-sm font-semibold uppercase tracking-[0.16em] text-[color:var(--app-muted)]">
          {title}
        </h2>
        {summary ? <p className="mt-1 text-sm text-[color:var(--app-muted)]">{summary}</p> : null}
      </div>
      <div className="p-4">{children}</div>
    </section>
  );
}

export function MetricsGrid({ children }: { children: ReactNode }) {
  return <div className="grid gap-4 md:grid-cols-3">{children}</div>;
}

export function MetricCard({
  label,
  value,
  detail,
}: {
  detail: string;
  label: string;
  value: string;
}) {
  return (
    <div className="grid min-h-32 gap-2 rounded-xl border border-[color:var(--app-line)] bg-[color:var(--app-panel)] p-4">
      <p className="text-xs font-semibold uppercase tracking-[0.16em] text-[color:var(--app-muted)]">
        {label}
      </p>
      <p className="text-2xl font-semibold tracking-tight">{value}</p>
      <p className="text-sm leading-6 text-[color:var(--app-muted)]">{detail}</p>
    </div>
  );
}

export function RouteLinkCard({
  description,
  icon: Icon,
  title,
  to,
}: {
  description: string;
  icon: typeof ExternalLink;
  title: string;
  to: string;
}) {
  return (
    <Link
      to={to}
      className="grid gap-3 rounded-xl border border-[color:var(--app-line)] bg-[color:var(--app-panel)] p-4 transition-colors hover:bg-[color:var(--app-panel-soft)]"
    >
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <Icon className="h-4 w-4 text-[color:var(--app-accent)]" />
          <span className="text-sm font-semibold">{title}</span>
        </div>
        <ArrowRight className="h-4 w-4 text-[color:var(--app-muted)]" />
      </div>
      <p className="text-sm leading-6 text-[color:var(--app-muted)]">{description}</p>
    </Link>
  );
}

export function InlineActionLink({
  description,
  icon: Icon,
  label,
  to,
}: {
  description: string;
  icon: typeof ScanSearch;
  label: string;
  to: string;
}) {
  return (
    <Link
      to={to}
      className="inline-flex items-center gap-2 rounded-full border border-[color:var(--app-line)] px-3 py-1.5 text-xs font-semibold text-[color:var(--app-fg)] transition-colors hover:bg-white/5"
      title={description}
    >
      <Icon className="h-3.5 w-3.5" />
      {label}
    </Link>
  );
}
