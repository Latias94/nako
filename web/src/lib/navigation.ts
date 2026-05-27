import {
  CircleUserRound,
  FolderKanban,
  Gauge,
  House,
  LibraryBig,
  PlayCircle,
  Puzzle,
  Search,
  Settings2,
  Workflow,
  type LucideIcon,
} from "lucide-react";

export type SurfaceKey = "media" | "admin";

export interface SurfaceNavItem {
  icon: LucideIcon;
  label: string;
  to: string;
  description: string;
  exact?: boolean;
}

export interface SurfaceDefinition {
  accentVar: string;
  bgVar: string;
  fgVar: string;
  label: string;
  nav: SurfaceNavItem[];
  panelVar: string;
  sidebarVar: string;
  subtitle: string;
  title: string;
  quickLinks: SurfaceNavItem[];
}

export interface CapabilityGap {
  area: string;
  status: "missing" | "partial" | "planned";
  note: string;
}

export const surfaceDefinitions: Record<SurfaceKey, SurfaceDefinition> = {
  media: {
    accentVar: "--media-accent",
    bgVar: "--media-bg",
    fgVar: "--media-fg",
    label: "Media",
    nav: [
      {
        icon: House,
        label: "Home",
        to: "/media",
        description: "Resume, browse, and open playback.",
        exact: true,
      },
      {
        icon: LibraryBig,
        label: "Libraries",
        to: "/media/libraries",
        description: "Browse media libraries and collections.",
      },
      {
        icon: Search,
        label: "Search",
        to: "/media/search",
        description: "Find titles, people, and collections.",
      },
      {
        icon: PlayCircle,
        label: "Watch",
        to: "/media/watch/example-item",
        description: "Open the browser playback view.",
      },
    ],
    panelVar: "--media-panel",
    sidebarVar: "--media-sidebar",
    subtitle: "Browse and playback",
    title: "Nako Media",
    quickLinks: [
      {
        icon: CircleUserRound,
        label: "Account",
        to: "/account",
        description: "Profile, roles, and sessions.",
      },
      {
        icon: Settings2,
        label: "Setup",
        to: "/setup",
        description: "Server connection and first run.",
      },
    ],
  },
  admin: {
    accentVar: "--admin-accent",
    bgVar: "--admin-bg",
    fgVar: "--admin-fg",
    label: "Admin",
    nav: [
      {
        icon: Gauge,
        label: "Overview",
        to: "/admin",
        description: "Health, work, and readiness.",
        exact: true,
      },
      {
        icon: LibraryBig,
        label: "Libraries",
        to: "/admin/libraries",
        description: "Scan, metadata, and write policy.",
      },
      {
        icon: Workflow,
        label: "Jobs",
        to: "/admin/jobs",
        description: "Queues, sessions, and runtime work.",
      },
      {
        icon: Puzzle,
        label: "Addons",
        to: "/admin/addons",
        description: "Sidecars, grants, and entry points.",
      },
      {
        icon: FolderKanban,
        label: "Settings",
        to: "/admin/settings",
        description: "Network, playback, and storage.",
      },
    ],
    panelVar: "--admin-panel",
    sidebarVar: "--admin-sidebar",
    subtitle: "Operator console",
    title: "Nako Admin",
    quickLinks: [
      {
        icon: CircleUserRound,
        label: "Account",
        to: "/account",
        description: "Profile, roles, and sessions.",
      },
      {
        icon: Settings2,
        label: "Setup",
        to: "/setup",
        description: "Server connection and first run.",
      },
    ],
  },
};

export const capabilityGaps: CapabilityGap[] = [
  {
    area: "Desktop native playback core",
    status: "missing",
    note: "The desktop package needs a Rust-owned playback engine before it can claim native playback quality.",
  },
  {
    area: "Management Context Links route matrix",
    status: "partial",
    note: "Media-to-admin jumps need one permission-gated map before they are safe to expose broadly.",
  },
  {
    area: "Account/session switching UX",
    status: "partial",
    note: "The frontend needs a clean account switcher backed by server session authority.",
  },
  {
    area: "Remote access operator UX",
    status: "partial",
    note: "Tunnel and exposure policy need an operator view that separates Nako-owned and user-owned network state.",
  },
  {
    area: "Addon manager lifecycle UI",
    status: "planned",
    note: "Install, update, grant review, and hosted-page entry points need dedicated operator workflows.",
  },
  {
    area: "Acquisition and downloads intake UI",
    status: "planned",
    note: "Download and acquisition intake should become its own release route after the core media flows are live.",
  },
];

export function isRouteActive(pathname: string, item: SurfaceNavItem): boolean {
  if (item.exact) {
    return pathname === item.to;
  }

  return pathname === item.to || pathname.startsWith(`${item.to}/`);
}
