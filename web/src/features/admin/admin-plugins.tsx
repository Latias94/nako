"use client"

import { useMemo, useState } from "react"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import {
  AlertTriangle,
  CheckCircle2,
  ClipboardList,
  ExternalLink,
  PackageCheck,
  PauseCircle,
  PlayCircle,
  Puzzle,
  Search,
  Server,
  Shield,
  SlidersHorizontal,
} from "lucide-react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  ADMIN_ADDON_MANAGER_FIXTURE,
  createAdminAddonManagerDataSource,
  type AdminAddonCatalogEntryReadModel,
  type AdminAddonInstalledReadModel,
} from "@/src/api/admin/addons-data-source"
import { createAdminMutationDataSource } from "@/src/api/admin/mutations-data-source"

export function AdminPlugins() {
  const queryClient = useQueryClient()
  const { data: addonData = ADMIN_ADDON_MANAGER_FIXTURE } = useQuery({
    queryKey: ["nako", "admin", "addons"],
    queryFn: () => createAdminAddonManagerDataSource().loadAddonManager(),
    staleTime: 30 * 1000,
    retry: 0,
  })
  const mutationSource = createAdminMutationDataSource()
  const canMutate = addonData.source === "live" && mutationSource.canMutate
  const [query, setQuery] = useState("")
  const [message, setMessage] = useState<string | null>(null)
  const statusMutation = useMutation({
    mutationFn: async (input: { addon: AdminAddonInstalledReadModel; status: "enabled" | "disabled" }) => {
      if (!canMutate) {
        throw new Error(mutationSource.unavailableReason ?? "Admin mutation is unavailable")
      }

      return mutationSource.updateAddonStatus(input.addon.id, input.status)
    },
    onSuccess(result) {
      setMessage(result.message)
      void queryClient.invalidateQueries({ queryKey: ["nako", "admin", "addons"] })
    },
    onError(error) {
      setMessage(error instanceof Error ? error.message : "Admin mutation failed")
    },
  })

  const filteredInstalled = useMemo(() => {
    const normalized = query.trim().toLowerCase()
    if (!normalized) return addonData.installed
    return addonData.installed.filter(
      (addon) =>
        addon.name.toLowerCase().includes(normalized) ||
        addon.manifestId.toLowerCase().includes(normalized) ||
        addon.baseUrl.toLowerCase().includes(normalized),
    )
  }, [addonData.installed, query])

  const filteredCatalog = useMemo(() => {
    const normalized = query.trim().toLowerCase()
    if (!normalized) return addonData.catalog
    return addonData.catalog.filter(
      (entry) =>
        entry.name.toLowerCase().includes(normalized) ||
        entry.manifestId.toLowerCase().includes(normalized) ||
        entry.description?.toLowerCase().includes(normalized),
    )
  }, [addonData.catalog, query])

  const enabledCount = addonData.installed.filter((addon) => addon.status === "enabled").length

  const runStatusMutation = (addon: AdminAddonInstalledReadModel, status: "enabled" | "disabled") => {
    setMessage(null)
    if (!canMutate) {
      setMessage(mutationSource.unavailableReason ?? "连接 live Admin API 后才能管理 Addon")
      return
    }

    const verb = status === "enabled" ? "启用" : "禁用"
    if (typeof window !== "undefined" && !window.confirm(`确认${verb} Addon「${addon.name}」？`)) {
      return
    }

    statusMutation.mutate({ addon, status })
  }

  return (
    <div className="space-y-6 p-1">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Addon Manager</h1>
          <p className="text-sm text-muted-foreground">
            管理 Nako sidecar addons、授权范围和安装边界
            <span className="ml-2 text-xs">
              {addonData.source === "live" ? "Live Admin API" : "Fixture fallback"}
              {addonData.error ? ` · ${addonData.error}` : ""}
            </span>
          </p>
        </div>
        <div className="relative w-full lg:w-80">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            className="pl-9"
            placeholder="搜索 addon、manifest 或 sidecar URL"
          />
        </div>
      </div>

      {(message || !canMutate) && (
        <div className="rounded-md border border-border/50 bg-muted/30 px-3 py-2 text-xs text-muted-foreground">
          {message ?? mutationSource.unavailableReason}
        </div>
      )}

      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <MetricCard icon={Puzzle} label="已注册" value={String(addonData.installed.length)} />
        <MetricCard icon={CheckCircle2} label="已启用" value={String(enabledCount)} />
        <MetricCard icon={PackageCheck} label="目录条目" value={String(addonData.catalog.length)} />
        <MetricCard icon={Shield} label="目录源" value={String(addonData.sources.length)} />
      </div>

      <Tabs defaultValue="installed" className="space-y-4">
        <TabsList>
          <TabsTrigger value="installed">已注册 Addons</TabsTrigger>
          <TabsTrigger value="catalog">官方目录</TabsTrigger>
          <TabsTrigger value="sources">目录源</TabsTrigger>
        </TabsList>

        <TabsContent value="installed" className="space-y-3">
          {filteredInstalled.length === 0 ? (
            <EmptyState text="没有已注册的 Addon" />
          ) : (
            <div className="grid gap-3 xl:grid-cols-2">
              {filteredInstalled.map((addon) => (
                <InstalledAddonCard
                  key={addon.id}
                  addon={addon}
                  canMutate={canMutate}
                  isPending={statusMutation.isPending}
                  onStatusChange={runStatusMutation}
                />
              ))}
            </div>
          )}
        </TabsContent>

        <TabsContent value="catalog" className="space-y-3">
          {filteredCatalog.length === 0 ? (
            <EmptyState text="没有匹配的目录条目" />
          ) : (
            <div className="grid gap-3 xl:grid-cols-2">
              {filteredCatalog.map((entry) => (
                <CatalogEntryCard key={`${entry.sourceId}:${entry.entryId}`} entry={entry} />
              ))}
            </div>
          )}
        </TabsContent>

        <TabsContent value="sources" className="space-y-3">
          {addonData.sources.map((source) => (
            <Card key={source.id} className="border-border/50 bg-card/50">
              <CardContent className="flex flex-col gap-3 p-4 md:flex-row md:items-center md:justify-between">
                <div>
                  <div className="flex items-center gap-2">
                    <Server className="h-4 w-4 text-muted-foreground" />
                    <h3 className="font-medium">{source.name}</h3>
                    <Badge variant="outline">{source.entryCount} entries</Badge>
                  </div>
                  <p className="mt-1 text-sm text-muted-foreground">
                    {source.description ?? "No source description"}
                  </p>
                </div>
                <div className="flex flex-wrap gap-2">
                  <CapabilityBadge active={source.providesPackageSigning} label="package signing" />
                  <CapabilityBadge active={source.providesProcessSupervision} label="process supervision" />
                  <CapabilityBadge active={source.providesProviderBreadth} label="provider breadth" />
                </div>
              </CardContent>
            </Card>
          ))}
        </TabsContent>
      </Tabs>
    </div>
  )
}

function InstalledAddonCard({
  addon,
  canMutate,
  isPending,
  onStatusChange,
}: {
  addon: AdminAddonInstalledReadModel
  canMutate: boolean
  isPending: boolean
  onStatusChange: (addon: AdminAddonInstalledReadModel, status: "enabled" | "disabled") => void
}) {
  const enabled = addon.status === "enabled"

  return (
    <Card className="border-border/50 bg-card/50">
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between gap-3">
          <div className="min-w-0">
            <CardTitle className="truncate text-base">{addon.name}</CardTitle>
            <p className="mt-1 truncate font-mono text-xs text-muted-foreground">{addon.manifestId}</p>
          </div>
          <StatusBadge status={addon.status} />
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid gap-2 text-xs text-muted-foreground sm:grid-cols-2">
          <Fact label="version" value={addon.version} />
          <Fact label="protocol" value={addon.protocolVersion} />
          <Fact label="sidecar" value={addon.baseUrl} wide />
          <Fact label="updated" value={addon.updatedAt} wide />
        </div>
        <TokenRow label="granted scopes" values={addon.grantedScopes} />
        <div className="flex flex-wrap gap-2">
          <Button
            size="sm"
            variant={enabled ? "outline" : "default"}
            disabled={!canMutate || isPending}
            onClick={() => onStatusChange(addon, enabled ? "disabled" : "enabled")}
          >
            {enabled ? <PauseCircle className="mr-2 h-4 w-4" /> : <PlayCircle className="mr-2 h-4 w-4" />}
            {enabled ? "禁用" : "启用"}
          </Button>
          <Button size="sm" variant="outline" disabled>
            <SlidersHorizontal className="mr-2 h-4 w-4" />
            Grants
          </Button>
          <Button size="sm" variant="outline" disabled>
            <ExternalLink className="mr-2 h-4 w-4" />
            Hosted pages
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}

function CatalogEntryCard({ entry }: { entry: AdminAddonCatalogEntryReadModel }) {
  return (
    <Card className="border-border/50 bg-card/50">
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between gap-3">
          <div className="min-w-0">
            <CardTitle className="truncate text-base">{entry.name}</CardTitle>
            <p className="mt-1 truncate font-mono text-xs text-muted-foreground">{entry.manifestId}</p>
          </div>
          {entry.installedStatus ? <StatusBadge status={entry.installedStatus} /> : <Badge variant="outline">available</Badge>}
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-sm text-muted-foreground">{entry.description ?? "No catalog description"}</p>
        <div className="grid gap-2 text-xs text-muted-foreground sm:grid-cols-2">
          <Fact label="runtime" value={entry.runtimeKind} />
          <Fact label="protocol" value={entry.protocolVersion} />
          <Fact label="version" value={entry.version} />
          <Fact label="signing" value={entry.packageSigningVerified ? "verified" : "manual"} />
        </div>
        <TokenRow label="resources" values={entry.resources} />
        <TokenRow label="requested scopes" values={entry.scopes} />
        <div className="rounded-md border border-border/50 bg-muted/30 p-3 text-xs text-muted-foreground">
          <div className="flex items-center gap-2 font-medium text-foreground">
            <ClipboardList className="h-4 w-4" />
            Lifecycle boundary
          </div>
          <p className="mt-1">{entry.lifecycleBoundary.message}</p>
        </div>
      </CardContent>
    </Card>
  )
}

function MetricCard({
  icon: Icon,
  label,
  value,
}: {
  icon: typeof Puzzle
  label: string
  value: string
}) {
  return (
    <Card className="border-border/50 bg-card/50">
      <CardContent className="flex items-center gap-3 p-4">
        <div className="rounded-lg bg-muted p-2">
          <Icon className="h-5 w-5 text-muted-foreground" />
        </div>
        <div>
          <p className="text-2xl font-semibold">{value}</p>
          <p className="text-xs text-muted-foreground">{label}</p>
        </div>
      </CardContent>
    </Card>
  )
}

function StatusBadge({ status }: { status: string }) {
  if (status === "enabled") {
    return (
      <Badge variant="secondary" className="bg-green-500/10 text-green-500">
        enabled
      </Badge>
    )
  }

  if (status === "disabled") {
    return <Badge variant="secondary">disabled</Badge>
  }

  return <Badge variant="outline">{status}</Badge>
}

function CapabilityBadge({ active, label }: { active: boolean; label: string }) {
  return (
    <Badge variant="outline" className={active ? "border-green-500/40 text-green-500" : "text-muted-foreground"}>
      {active ? <CheckCircle2 className="mr-1 h-3 w-3" /> : <AlertTriangle className="mr-1 h-3 w-3" />}
      {label}
    </Badge>
  )
}

function Fact({ label, value, wide = false }: { label: string; value: string; wide?: boolean }) {
  return (
    <div className={wide ? "sm:col-span-2" : undefined}>
      <span className="uppercase tracking-wide">{label}</span>
      <p className="mt-0.5 truncate font-mono text-foreground">{value}</p>
    </div>
  )
}

function TokenRow({ label, values }: { label: string; values: string[] }) {
  return (
    <div>
      <p className="mb-2 text-xs uppercase tracking-wide text-muted-foreground">{label}</p>
      <div className="flex flex-wrap gap-1.5">
        {values.length === 0 ? (
          <Badge variant="outline" className="text-muted-foreground">
            none
          </Badge>
        ) : (
          values.map((value) => (
            <Badge key={value} variant="outline" className="font-mono text-[10px]">
              {value}
            </Badge>
          ))
        )}
      </div>
    </div>
  )
}

function EmptyState({ text }: { text: string }) {
  return (
    <Card className="border-border/50 bg-card/50">
      <CardContent className="flex flex-col items-center justify-center gap-2 py-12 text-center">
        <Puzzle className="h-10 w-10 text-muted-foreground/50" />
        <p className="text-sm text-muted-foreground">{text}</p>
      </CardContent>
    </Card>
  )
}
