import type { ReactNode } from "react";

export function FilterBar({ children, label }: { children: ReactNode; label: string }) {
  return (
    <div className="filterBar" aria-label={label}>
      {children}
    </div>
  );
}

export function FilterField({ children, label }: { children: ReactNode; label: string }) {
  return (
    <label className="filterField">
      <span>{label}</span>
      {children}
    </label>
  );
}

export function FilterActions({ children }: { children: ReactNode }) {
  return <div className="filterActions">{children}</div>;
}
