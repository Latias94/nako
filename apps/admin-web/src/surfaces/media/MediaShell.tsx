import { Link } from "@tanstack/react-router";
import { Library, LogOut, Search, Server, Settings, Sparkles } from "lucide-react";
import type { ReactNode } from "react";

import { Button } from "../../components/ui/Button";
import { Badge } from "../../components/ui/Badge";
import { useMediaSession } from "./MediaSession";

const mediaNavItems = [
  { to: "/media", label: "Home", icon: Sparkles },
  { to: "/media/libraries", label: "Libraries", icon: Library },
  { to: "/media/search", label: "Search", icon: Search },
] as const;

export function MediaShell({
  activePathname,
  children,
}: {
  activePathname: string;
  children: ReactNode;
}) {
  const { clearConnection, connection, dataSource } = useMediaSession();

  return (
    <div className="mediaShell">
      <aside className="mediaSidebar">
        <div className="mediaBrand">
          <img src="/nako-app-icon-1024.png" alt="" />
          <div>
            <strong>Nako</strong>
            <span>Media</span>
          </div>
        </div>
        <div className="surfaceSwitch" aria-label="Surface">
          <Link className="surfaceSwitchItem active" to="/media">
            <Library size={16} />
            <span>Media</span>
          </Link>
          <Link className="surfaceSwitchItem" to="/overview">
            <Settings size={16} />
            <span>Admin</span>
          </Link>
        </div>
        <nav className="mediaNav" aria-label="Media navigation">
          {mediaNavItems.map((item) => {
            const Icon = item.icon;
            const active =
              activePathname === item.to || activePathname.startsWith(`${item.to}/`);
            return (
              <Link
                className={active ? "mediaNavItem active" : "mediaNavItem"}
                key={item.to}
                to={item.to}
              >
                <Icon size={17} />
                <span>{item.label}</span>
              </Link>
            );
          })}
        </nav>
        <div className="mediaConnection">
          <Badge tone={connection?.mode === "fixture" ? "warning" : "info"}>
            {dataSource?.label ?? "Not connected"}
          </Badge>
          {connection?.mode === "live" ? (
            <span className="mediaConnectionHost">{connection.baseUrl}</span>
          ) : null}
          {connection ? (
            <Button className="mediaLogoutButton" onClick={clearConnection} size="sm" variant="ghost">
              <LogOut size={15} />
              <span>Change connection</span>
            </Button>
          ) : null}
        </div>
      </aside>
      <main className="mediaMain">
        <div className="mediaTopbar">
          <div>
            <p className="mediaKicker">Media Web</p>
            <h1>Local media, current access</h1>
          </div>
          <div className="mediaTopbarStatus">
            <Server size={16} />
            <span>{dataSource?.label ?? "Connect"}</span>
          </div>
        </div>
        {children}
      </main>
    </div>
  );
}
