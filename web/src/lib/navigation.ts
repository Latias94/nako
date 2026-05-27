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
    accentVar: "--app-accent",
    bgVar: "--app-bg",
    fgVar: "--app-fg",
    label: "Media",
    nav: [
      {
        icon: House,
        label: "Home",
        to: "/media",
        description: "Browser-first landing area for viewing and playback.",
        exact: true,
      },
      {
        icon: LibraryBig,
        label: "Libraries",
        to: "/media/libraries",
        description: "Browse libraries, sources, and library-scoped items.",
      },
      {
        icon: Search,
        label: "Search",
        to: "/media/search",
        description: "Search across the public catalog surface.",
      },
      {
        icon: PlayCircle,
        label: "Watch",
        to: "/media/watch/example-item",
        description: "Reserve the browser playback ticket seam.",
      },
    ],
    panelVar: "--app-panel",
    sidebarVar: "--app-sidebar",
    subtitle: "Release frontend line",
    title: "Nako Media",
    quickLinks: [
      {
        icon: CircleUserRound,
        label: "Account",
        to: "/account",
        description: "Current principal and session shell.",
      },
      {
        icon: Settings2,
        label: "Setup",
        to: "/setup",
        description: "Bootstrap and connection flow.",
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
        description: "Operational summary, backlog signals, and health.",
        exact: true,
      },
      {
        icon: LibraryBig,
        label: "Libraries",
        to: "/admin/libraries",
        description: "Library policy, metadata control, and safe operations.",
      },
      {
        icon: Workflow,
        label: "Jobs",
        to: "/admin/jobs",
        description: "Task runtime and queue state.",
      },
      {
        icon: Puzzle,
        label: "Addons",
        to: "/admin/addons",
        description: "Addon surfaces, onboarding, and future lifecycle work.",
      },
      {
        icon: FolderKanban,
        label: "Settings",
        to: "/admin/settings",
        description: "Network, playback, and configuration authority.",
      },
    ],
    panelVar: "--admin-panel",
    sidebarVar: "--admin-sidebar",
    subtitle: "Validation console",
    title: "Nako Admin",
    quickLinks: [
      {
        icon: CircleUserRound,
        label: "Account",
        to: "/account",
        description: "Current principal and session shell.",
      },
      {
        icon: Settings2,
        label: "Setup",
        to: "/setup",
        description: "Bootstrap and connection flow.",
      },
    ],
  },
};

export const capabilityGaps: CapabilityGap[] = [
  {
    area: "Desktop native playback core",
    status: "missing",
    note: "Tauri shell work is separate from a real Rust-owned player core.",
  },
  {
    area: "Management Context Links route matrix",
    status: "partial",
    note: "The new frontend still needs the permission-gated media-to-admin link map.",
  },
  {
    area: "Account/session switching UX",
    status: "partial",
    note: "Identity and session authority exist; the new frontend still needs a clean switcher flow.",
  },
  {
    area: "Remote access operator UX",
    status: "partial",
    note: "Network tunnel and exposure policies need product-grade frontend surfaces.",
  },
  {
    area: "Addon manager lifecycle UI",
    status: "planned",
    note: "Backend control plane exists; the release frontend still needs operator workflows.",
  },
  {
    area: "Acquisition and downloads intake UI",
    status: "planned",
    note: "The server can accept intake concepts, but the product shell still needs release routes.",
  },
];
