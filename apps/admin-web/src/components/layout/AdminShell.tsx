import { Link } from "@tanstack/react-router";
import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";

export type AdminRouteTo =
  | "/overview"
  | "/jobs"
  | "/libraries"
  | "/catalog/governance"
  | "/playback/sessions"
  | "/storage/staging"
  | "/addons"
  | "/settings"
  | "/legacy";

export type AdminShellNavItem = {
  to: AdminRouteTo;
  label: string;
  icon: LucideIcon;
};

export function AdminShell({
  activePathname,
  children,
  navItems,
}: {
  activePathname: string;
  children: ReactNode;
  navItems: readonly AdminShellNavItem[];
}) {
  return (
    <div className="adminRouteShell">
      <aside className="routeSidebar" aria-label="Primary navigation">
        <div className="routeBrand">
          <img src="/nako-app-icon-1024.png" alt="" />
          <div>
            <strong>Nako</strong>
            <span>Admin Web V2</span>
          </div>
        </div>
        <nav className="routeNav">
          {navItems.map((item) => {
            const Icon = item.icon;
            return (
              <Link
                className={activePathname === item.to ? "routeNavItem active" : "routeNavItem"}
                key={item.to}
                to={item.to}
              >
                <Icon size={17} />
                <span>{item.label}</span>
              </Link>
            );
          })}
        </nav>
      </aside>
      <main className="routeMain">{children}</main>
    </div>
  );
}
