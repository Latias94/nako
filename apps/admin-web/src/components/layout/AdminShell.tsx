import { Link } from "@tanstack/react-router";
import { Library, Settings, type LucideIcon } from "lucide-react";
import type { ReactNode } from "react";

import { supportedAdminLocales, useI18n } from "../../i18n/I18nProvider";
import type { AdminLocale } from "../../i18n/messages";

export type AdminRouteTo =
  | "/overview"
  | "/jobs"
  | "/events"
  | "/access"
  | "/libraries"
  | "/catalog"
  | "/catalog/governance"
  | "/acquisition/intake"
  | "/automation/generated-artifacts"
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
  locale,
  navItems,
  onLocaleChange,
}: {
  activePathname: string;
  children: ReactNode;
  locale: AdminLocale;
  navItems: readonly AdminShellNavItem[];
  onLocaleChange(locale: AdminLocale): void;
}) {
  const { t } = useI18n();

  return (
    <div className="adminRouteShell">
      <aside className="routeSidebar">
        <div className="routeBrand">
          <img src="/nako-app-icon-1024.png" alt="" />
          <div>
            <strong>Nako</strong>
            <span>{t("shell.product")}</span>
          </div>
        </div>
        <div className="adminSurfaceSwitch" aria-label="Surface">
          <Link className="adminSurfaceSwitchItem active" to="/overview">
            <Settings size={16} />
            <span>Admin</span>
          </Link>
          <Link className="adminSurfaceSwitchItem" to="/media">
            <Library size={16} />
            <span>Media</span>
          </Link>
        </div>
        <nav className="routeNav" aria-label={t("shell.primaryNavigation")}>
          {navItems.map((item) => {
            const Icon = item.icon;
            const active =
              activePathname === item.to || activePathname.startsWith(`${item.to}/`);
            return (
              <Link
                activeOptions={{ exact: true }}
                className={active ? "routeNavItem active" : "routeNavItem"}
                key={item.to}
                to={item.to}
              >
                <Icon size={17} />
                <span>{item.label}</span>
              </Link>
            );
          })}
        </nav>
        <div className="routeSidebarFooter">
          <label className="routeLocaleControl">
            <span>{t("shell.locale")}</span>
            <select
              aria-label={t("shell.locale")}
              onChange={(event) => onLocaleChange(event.currentTarget.value as AdminLocale)}
              value={locale}
            >
              {supportedAdminLocales().map((option) => (
                <option key={option} value={option}>
                  {option === "zh-Hans" ? t("locale.zhHans") : t("locale.enUS")}
                </option>
              ))}
            </select>
          </label>
        </div>
      </aside>
      <main className="routeMain">{children}</main>
    </div>
  );
}
