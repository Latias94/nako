import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { RefreshCw, RotateCcw, Search, Send, X } from "lucide-react";
import { useMemo, useState, type FormEvent } from "react";

import type { AdminDataSource, DataSourceMode } from "../../adminApi/dataSource";
import { mockAdminConsoleData } from "../../adminApi/mockData";
import type {
  EventDeliveryAttemptRow,
  EventDispatchSummary,
  EventListQuery,
  EventReplaySummary,
  EventRow,
  EventSchedulerWorkRow,
  EventSchedulerWorkSummary,
  EventSummary,
} from "../../adminApi/types";
import { SourceLabel } from "../../components/SourceLabel";
import { EmptyRouteState, RouteNotice, RoutePage } from "../../components/layout/RoutePage";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { DataPanel } from "../../components/ui/DataPanel";
import { FilterActions, FilterBar, FilterField } from "../../components/ui/FilterBar";
import { RowsSkeleton } from "../../components/ui/RowsSkeleton";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../../components/ui/Table";
import { useI18n } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";

export type EventsSearch = {
  status?: string;
  kind?: string;
  library_id?: string;
  source_id?: string;
  limit: number;
  offset: number;
};

export type EventsPageProps = {
  dataSource: AdminDataSource;
  search: EventsSearch;
  onSearchChange(next: Partial<EventsSearch>): void;
};

type SectionResult<T> = {
  value: T;
  source: DataSourceMode;
  error?: string;
};

type Translate = (id: MessageId, values?: Record<string, number | string>) => string;
type BadgeTone = "danger" | "info" | "neutral" | "success" | "warning";
type EventCommand =
  | {
      action: "deliver";
      eventId: string;
    }
  | {
      action: "replay";
      eventId: string;
      reasonCode: string;
    };
type EventCommandResult =
  | {
      action: "deliver";
      response: EventDispatchSummary;
    }
  | {
      action: "replay";
      response: EventReplaySummary;
    };

export function EventsPage({ dataSource, search, onSearchChange }: EventsPageProps) {
  const { locale, t } = useI18n();
  const queryClient = useQueryClient();
  const [selectedEventId, setSelectedEventId] = useState<string | null>(null);
  const [replayReasonCode, setReplayReasonCode] = useState("");
  const [replayCandidateId, setReplayCandidateId] = useState<string | null>(null);
  const [commandMessage, setCommandMessage] = useState<string | null>(null);
  const [commandError, setCommandError] = useState<string | null>(null);

  const eventsQuery = useQuery({
    queryKey: ["admin-events", search, locale],
    queryFn: () => loadEvents(dataSource, search, t("events.dataSourceUnavailable")),
  });
  const result = eventsQuery.data ?? {
    value: mockAdminConsoleData.events,
    source: "mock" as const,
  };
  const selectedEvent = useMemo(
    () =>
      result.value.events.find((event) => event.id === selectedEventId) ??
      result.value.events[0] ??
      null,
    [result.value.events, selectedEventId],
  );

  const attemptsQuery = useQuery({
    enabled: Boolean(selectedEvent),
    queryKey: ["admin-event-attempts", selectedEvent?.id, locale],
    queryFn: () =>
      loadDeliveryAttempts(
        dataSource,
        selectedEvent?.id ?? "",
        t("events.attempts.dataSourceUnavailable"),
      ),
  });
  const attemptsResult = attemptsQuery.data ?? {
    value: [] as EventDeliveryAttemptRow[],
    source: "mock" as const,
  };

  const schedulerQuery = useQuery({
    enabled: Boolean(selectedEvent),
    queryKey: ["admin-event-scheduler-work", selectedEvent?.id, locale],
    queryFn: () =>
      loadSchedulerWork(
        dataSource,
        selectedEvent,
        t("events.scheduler.dataSourceUnavailable"),
      ),
  });
  const schedulerResult = schedulerQuery.data ?? {
    value: emptySchedulerWork(selectedEvent),
    source: "mock" as const,
  };

  const commandMutation = useMutation<EventCommandResult, Error, EventCommand>({
    mutationFn: async (command) => {
      if (result.source !== "live") {
        throw new Error(t("events.command.notLiveError"));
      }

      if (command.action === "deliver") {
        if (!dataSource.deliverAddonEvents) {
          throw new Error(t("events.deliver.unavailable"));
        }

        return {
          action: command.action,
          response: await dataSource.deliverAddonEvents(command.eventId),
        };
      }

      if (!dataSource.replayAddonEvents) {
        throw new Error(t("events.replay.unavailable"));
      }

      return {
        action: command.action,
        response: await dataSource.replayAddonEvents(command.eventId, command.reasonCode),
      };
    },
    onMutate: () => {
      setCommandMessage(null);
      setCommandError(null);
    },
    onSuccess: (response) => {
      if (response.action === "deliver") {
        setCommandMessage(
          t("events.deliver.succeeded", {
            delivered: response.response.delivered,
            failed: response.response.failed,
          }),
        );
      } else {
        setReplayCandidateId(null);
        setCommandMessage(
          t("events.replay.succeeded", {
            reasonCode: response.response.reasonCode,
            delivered: response.response.dispatch.delivered,
            failed: response.response.dispatch.failed,
          }),
        );
      }
      void queryClient.invalidateQueries({ queryKey: ["admin-events"] });
      void queryClient.invalidateQueries({ queryKey: ["admin-event-attempts"] });
      void queryClient.invalidateQueries({ queryKey: ["admin-event-scheduler-work"] });
    },
    onError: (error) => {
      setCommandError(errorMessage(error, t("events.command.failed")));
    },
  });

  const activeFilterCount = useMemo(
    () => [search.status, search.kind, search.library_id, search.source_id].filter(Boolean).length,
    [search],
  );
  const canDeliver = result.source === "live" && Boolean(dataSource.deliverAddonEvents);
  const canReplay = result.source === "live" && Boolean(dataSource.replayAddonEvents);

  return (
    <RoutePage
      actions={
        <Button
          disabled={eventsQuery.isFetching}
          onClick={() => void eventsQuery.refetch()}
          variant="outline"
        >
          <RefreshCw size={16} />
          {t("events.refresh")}
        </Button>
      }
      description={t("events.description")}
      kicker={t("events.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("events.title")}
      titleId="events-route-title"
    >
      {result.error ? <RouteNotice>{t("events.fallback", { error: result.error })}</RouteNotice> : null}
      {!eventsQuery.isLoading && result.source !== "live" ? (
        <RouteNotice>{t("events.command.actionsDisabled")}</RouteNotice>
      ) : null}
      {commandError ? <RouteNotice>{commandError}</RouteNotice> : null}
      {commandMessage ? <RouteNotice>{commandMessage}</RouteNotice> : null}

      <FilterBar label={t("events.filters")}>
        <FilterField label={t("events.filter.status")}>
          <select
            aria-label={t("events.filter.statusAria")}
            onChange={(event) => onSearchChange({ status: event.target.value || undefined, offset: 0 })}
            value={search.status ?? ""}
          >
            <option value="">{t("events.filter.anyStatus")}</option>
            <option value="pending">{t("events.status.pending")}</option>
            <option value="running">{t("events.status.running")}</option>
            <option value="succeeded">{t("events.status.succeeded")}</option>
            <option value="failed">{t("events.status.failed")}</option>
          </select>
        </FilterField>
        <FilterField label={t("events.filter.kind")}>
          <input
            aria-label={t("events.filter.kindAria")}
            onChange={(event) => onSearchChange({ kind: event.target.value || undefined, offset: 0 })}
            placeholder="library_scanned"
            value={search.kind ?? ""}
          />
        </FilterField>
        <FilterField label={t("events.filter.library")}>
          <input
            aria-label={t("events.filter.libraryAria")}
            onChange={(event) =>
              onSearchChange({ library_id: event.target.value || undefined, offset: 0 })
            }
            placeholder="library-id"
            value={search.library_id ?? ""}
          />
        </FilterField>
        <FilterField label={t("events.filter.source")}>
          <input
            aria-label={t("events.filter.sourceAria")}
            onChange={(event) =>
              onSearchChange({ source_id: event.target.value || undefined, offset: 0 })
            }
            placeholder="source-id"
            value={search.source_id ?? ""}
          />
        </FilterField>
        <FilterField label={t("events.filter.limit")}>
          <input
            aria-label={t("events.filter.limitAria")}
            min={1}
            onChange={(event) =>
              onSearchChange({
                limit: positiveNumberInput(event.target.value, search.limit, 1),
                offset: 0,
              })
            }
            type="number"
            value={search.limit}
          />
        </FilterField>
        <FilterField label={t("events.filter.offset")}>
          <input
            aria-label={t("events.filter.offsetAria")}
            min={0}
            onChange={(event) =>
              onSearchChange({
                offset: positiveNumberInput(event.target.value, search.offset, 0),
              })
            }
            type="number"
            value={search.offset}
          />
        </FilterField>
        <FilterActions>
          <Badge tone={activeFilterCount > 0 ? "info" : "neutral"}>
            {t("events.filter.active", { count: activeFilterCount })}
          </Badge>
          <Button
            disabled={activeFilterCount === 0}
            onClick={() =>
              onSearchChange({
                status: undefined,
                kind: undefined,
                library_id: undefined,
                source_id: undefined,
                offset: 0,
              })
            }
            variant="ghost"
          >
            <X size={16} />
            {t("events.clear")}
          </Button>
        </FilterActions>
      </FilterBar>

      <div className="eventsSummaryGrid">
        <SummaryTile
          label={t("events.summary.returned")}
          tone="info"
          value={result.value.page.returned}
        />
        <SummaryTile
          label={t("events.summary.pending")}
          tone="warning"
          value={result.value.events.filter((event) => event.status === "pending").length}
        />
        <SummaryTile
          label={t("events.summary.failed")}
          tone={
            result.value.events.some((event) => event.status === "failed" || event.hasError)
              ? "danger"
              : "neutral"
          }
          value={result.value.events.filter((event) => event.status === "failed" || event.hasError).length}
        />
        <SummaryTile
          label={t("events.summary.nextAttempt")}
          tone={result.value.events.some((event) => event.nextAttemptAt) ? "warning" : "neutral"}
          value={result.value.events.filter((event) => event.nextAttemptAt).length}
        />
      </div>

      <DataPanel
        description={t("events.list.description", {
          returned: result.value.page.returned,
          offset: result.value.page.offset,
          limit: result.value.page.limit,
        })}
        headerAccessory={
          <div className="searchHint">
            <Search size={15} />
            {t("events.list.urlFilters")}
          </div>
        }
        title={t("events.list.title")}
      >
        {eventsQuery.isLoading ? <RowsSkeleton label={t("events.loading")} /> : null}
        {!eventsQuery.isLoading && result.value.events.length === 0 ? (
          <EmptyRouteState>{t("events.empty")}</EmptyRouteState>
        ) : null}
        {!eventsQuery.isLoading && result.value.events.length > 0 ? (
          <EventTable
            events={result.value.events}
            selectedEventId={selectedEvent?.id ?? null}
            t={t}
            onSelect={setSelectedEventId}
          />
        ) : null}
      </DataPanel>

      <div className="eventsRouteGrid">
        <DataPanel
          description={t("events.selected.description")}
          title={t("events.selected.title")}
        >
          {selectedEvent ? (
            <SelectedEventFacts event={selectedEvent} t={t} />
          ) : (
            <EmptyRouteState>{t("events.selected.empty")}</EmptyRouteState>
          )}
        </DataPanel>

        <DataPanel
          description={t("events.commands.description")}
          headerAccessory={
            <div className="searchHint">
              <Send size={15} />
              {t("events.commands.liveOnly")}
            </div>
          }
          title={t("events.commands.title")}
        >
          {selectedEvent ? (
            <EventCommandPanel
              canDeliver={canDeliver}
              canReplay={canReplay}
              event={selectedEvent}
              isPending={commandMutation.isPending}
              onCancelReplay={() => {
                setReplayCandidateId(null);
                commandMutation.reset();
              }}
              onDeliver={() =>
                commandMutation.mutate({
                  action: "deliver",
                  eventId: selectedEvent.id,
                })
              }
              onPrepareReplay={() => {
                const reasonCode = replayReasonCode.trim();
                setCommandMessage(null);
                commandMutation.reset();
                if (!reasonCode) {
                  setCommandError(t("events.replay.reasonRequired"));
                  return;
                }
                setCommandError(null);
                setReplayCandidateId(selectedEvent.id);
              }}
              onReplay={(eventId, reasonCode) =>
                commandMutation.mutate({
                  action: "replay",
                  eventId,
                  reasonCode,
                })
              }
              replayCandidateId={replayCandidateId}
              replayReasonCode={replayReasonCode}
              setReplayReasonCode={(value) => {
                setReplayReasonCode(value);
                setReplayCandidateId(null);
                setCommandError(null);
              }}
              source={result.source}
              t={t}
            />
          ) : (
            <EmptyRouteState>{t("events.commands.empty")}</EmptyRouteState>
          )}
        </DataPanel>

        <DataPanel
          description={t("events.scheduler.description", {
            due: schedulerResult.value.dueWorkCount,
            blocked: schedulerResult.value.blockedWorkCount,
          })}
          headerAccessory={<SourceLabel source={schedulerResult.source} />}
          title={t("events.scheduler.title")}
        >
          {schedulerResult.error ? (
            <div className="settingsInlineNotice">{t("events.scheduler.fallback", { error: schedulerResult.error })}</div>
          ) : null}
          {schedulerQuery.isLoading ? <RowsSkeleton label={t("events.scheduler.loading")} /> : null}
          {!schedulerQuery.isLoading ? (
            <SchedulerWorkList t={t} work={schedulerResult.value.work} />
          ) : null}
        </DataPanel>

        <DataPanel
          description={t("events.attempts.description", {
            count: attemptsResult.value.length,
          })}
          headerAccessory={<SourceLabel source={attemptsResult.source} />}
          title={t("events.attempts.title")}
        >
          {attemptsResult.error ? (
            <div className="settingsInlineNotice">{t("events.attempts.fallback", { error: attemptsResult.error })}</div>
          ) : null}
          {attemptsQuery.isLoading ? <RowsSkeleton label={t("events.attempts.loading")} /> : null}
          {!attemptsQuery.isLoading ? (
            <DeliveryAttemptsList attempts={attemptsResult.value} t={t} />
          ) : null}
        </DataPanel>
      </div>
    </RoutePage>
  );
}

async function loadEvents(
  dataSource: AdminDataSource,
  search: EventsSearch,
  unavailableMessage: string,
): Promise<SectionResult<EventSummary>> {
  if (!dataSource.loadEvents) {
    return {
      value: mockAdminConsoleData.events,
      source: "mock",
      error: unavailableMessage,
    };
  }

  return dataSource.loadEvents(toEventListQuery(search));
}

function toEventListQuery(search: EventsSearch): EventListQuery {
  return {
    status: search.status,
    kind: search.kind,
    library_id: search.library_id,
    source_id: search.source_id,
    limit: search.limit,
    offset: search.offset,
  };
}

async function loadDeliveryAttempts(
  dataSource: AdminDataSource,
  eventId: string,
  unavailableMessage: string,
): Promise<SectionResult<EventDeliveryAttemptRow[]>> {
  if (!dataSource.loadAddonEventDeliveryAttempts) {
    return {
      value: [],
      source: "mock",
      error: unavailableMessage,
    };
  }

  return dataSource.loadAddonEventDeliveryAttempts(eventId);
}

async function loadSchedulerWork(
  dataSource: AdminDataSource,
  event: EventRow | null,
  unavailableMessage: string,
): Promise<SectionResult<EventSchedulerWorkSummary>> {
  if (!event || !dataSource.loadAddonEventSchedulerWork) {
    return {
      value: emptySchedulerWork(event),
      source: "mock",
      error: event ? unavailableMessage : undefined,
    };
  }

  return dataSource.loadAddonEventSchedulerWork(event.id);
}

function EventTable({
  events,
  onSelect,
  selectedEventId,
  t,
}: {
  events: EventRow[];
  onSelect(eventId: string): void;
  selectedEventId: string | null;
  t: Translate;
}) {
  return (
    <div className="tableScroll">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>{t("events.column.event")}</TableHead>
            <TableHead>{t("events.column.status")}</TableHead>
            <TableHead>{t("events.column.scope")}</TableHead>
            <TableHead>{t("events.column.delivery")}</TableHead>
            <TableHead>{t("events.column.updated")}</TableHead>
            <TableHead>{t("events.column.actions")}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {events.map((event) => (
            <TableRow className={selectedEventId === event.id ? "eventsSelectedRow" : undefined} key={event.id}>
              <TableCell>
                <div className="routePrimaryCell">
                  <strong>{event.kind}</strong>
                  <span>{event.id}</span>
                </div>
              </TableCell>
              <TableCell>
                <Badge tone={eventStatusTone(event.status, event.hasError)}>{event.status}</Badge>
              </TableCell>
              <TableCell>{eventScope(event, t)}</TableCell>
              <TableCell>
                <div className="eventsBadgeStack">
                  <Badge tone={event.attempts > 0 ? "info" : "neutral"}>
                    {t("events.attempts.countShort", { count: event.attempts })}
                  </Badge>
                  <Badge tone={event.hasPayload ? "warning" : "neutral"}>
                    {event.hasPayload ? t("events.payload.present") : t("events.payload.absent")}
                  </Badge>
                  <Badge tone={event.hasError ? "danger" : "neutral"}>
                    {event.hasError ? t("events.error.present") : t("events.error.absent")}
                  </Badge>
                </div>
              </TableCell>
              <TableCell>
                <div className="eventsTimeCell">
                  <span>{timestampLabel(event.updatedAt)}</span>
                  {event.nextAttemptAt ? (
                    <small>{t("events.nextAttemptAt", { time: timestampLabel(event.nextAttemptAt) })}</small>
                  ) : null}
                </div>
              </TableCell>
              <TableCell>
                <Button
                  aria-label={t("events.selectAria", { eventId: event.id })}
                  onClick={() => onSelect(event.id)}
                  size="sm"
                  variant={selectedEventId === event.id ? "default" : "outline"}
                >
                  {selectedEventId === event.id ? t("events.selectedAction") : t("events.select")}
                </Button>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}

function SelectedEventFacts({ event, t }: { event: EventRow; t: Translate }) {
  return (
    <div className="eventsFactList">
      <FactRow label={t("events.fact.eventId")} value={event.id} />
      <FactRow badge={event.status} label={t("events.fact.status")} tone={eventStatusTone(event.status, event.hasError)} value={event.kind} />
      <FactRow label={t("events.fact.scope")} value={eventScope(event, t)} />
      <FactRow
        badge={event.hasPayload ? t("events.payload.present") : t("events.payload.absent")}
        label={t("events.fact.payload")}
        tone={event.hasPayload ? "warning" : "neutral"}
        value={t("events.fact.attempts", { count: event.attempts })}
      />
      <FactRow
        badge={event.hasError ? t("events.error.present") : t("events.error.absent")}
        label={t("events.fact.updated")}
        tone={event.hasError ? "danger" : "neutral"}
        value={timestampLabel(event.updatedAt)}
      />
      <FactRow
        label={t("events.fact.nextAttempt")}
        value={event.nextAttemptAt ? timestampLabel(event.nextAttemptAt) : t("events.none")}
      />
    </div>
  );
}

function EventCommandPanel({
  canDeliver,
  canReplay,
  event,
  isPending,
  onCancelReplay,
  onDeliver,
  onPrepareReplay,
  onReplay,
  replayCandidateId,
  replayReasonCode,
  setReplayReasonCode,
  source,
  t,
}: {
  canDeliver: boolean;
  canReplay: boolean;
  event: EventRow;
  isPending: boolean;
  onCancelReplay(): void;
  onDeliver(): void;
  onPrepareReplay(): void;
  onReplay(eventId: string, reasonCode: string): void;
  replayCandidateId: string | null;
  replayReasonCode: string;
  setReplayReasonCode(value: string): void;
  source: DataSourceMode;
  t: Translate;
}) {
  const reasonCode = replayReasonCode.trim();

  return (
    <div className="eventsCommandPanel">
      {source !== "live" ? (
        <div className="settingsInlineNotice">{t("events.command.notLiveError")}</div>
      ) : null}
      <div className="eventsCommandRow">
        <div>
          <strong>{t("events.deliver.title")}</strong>
          <span>{t("events.deliver.description")}</span>
        </div>
        <Button disabled={!canDeliver || isPending} onClick={onDeliver}>
          <Send size={16} />
          {isPending ? t("events.deliver.running") : t("events.deliver.action")}
        </Button>
      </div>
      <form
        className="eventsReplayForm"
        onSubmit={(formEvent: FormEvent<HTMLFormElement>) => {
          formEvent.preventDefault();
          if (replayCandidateId === event.id) {
            onReplay(event.id, reasonCode);
          } else {
            onPrepareReplay();
          }
        }}
      >
        <label>
          {t("events.replay.reasonCode")}
          <input
            aria-label={t("events.replay.reasonCodeAria")}
            onChange={(inputEvent) => setReplayReasonCode(inputEvent.target.value)}
            placeholder="operator_requested"
            value={replayReasonCode}
          />
        </label>
        {replayCandidateId === event.id ? (
          <div className="eventsReplayConfirm">
            <small>{t("events.replay.confirmCopy", { eventId: event.id, reasonCode })}</small>
            <div>
              <Button disabled={!canReplay || isPending || !reasonCode} type="submit">
                <RotateCcw size={16} />
                {isPending ? t("events.replay.running") : t("events.replay.confirm")}
              </Button>
              <Button disabled={isPending} onClick={onCancelReplay} type="button" variant="ghost">
                {t("events.replay.cancel")}
              </Button>
            </div>
          </div>
        ) : (
          <Button disabled={!canReplay || isPending} type="submit" variant="outline">
            <RotateCcw size={16} />
            {t("events.replay.prepare")}
          </Button>
        )}
      </form>
    </div>
  );
}

function SchedulerWorkList({ t, work }: { t: Translate; work: EventSchedulerWorkRow[] }) {
  if (work.length === 0) {
    return <EmptyRouteState>{t("events.scheduler.empty")}</EmptyRouteState>;
  }

  return (
    <div className="eventsWorkList">
      {work.map((item) => (
        <div className="eventsWorkRow" key={`${item.addonId}:${item.declarationId}:${item.eventKind}`}>
          <div>
            <strong>{item.declarationId}</strong>
            <span>{t("events.scheduler.addon", { addonId: item.addonId })}</span>
            <small>{t("events.scheduler.manifest", { manifestId: item.manifestId, version: item.manifestVersion })}</small>
            <small>{t("events.scheduler.routing", { status: item.routingPlanStatus, target: item.routingPlanTarget })}</small>
            <small>{t("events.scheduler.attempts", { attempt: item.nextAttemptNumber, max: item.maxAttempts })}</small>
            {item.latestAttemptStatus ? (
              <small>{t("events.scheduler.latest", { status: item.latestAttemptStatus })}</small>
            ) : null}
            {item.safeReasonCode ? (
              <small>{t("events.scheduler.safeReason", { reasonCode: item.safeReasonCode })}</small>
            ) : null}
          </div>
          <div className="eventsRowBadges">
            <Badge tone={schedulerTone(item.status)}>{item.status}</Badge>
            <Badge tone={item.latestHttpStatus && item.latestHttpStatus >= 400 ? "danger" : "neutral"}>
              {item.latestHttpStatus ? String(item.latestHttpStatus) : t("events.none")}
            </Badge>
          </div>
        </div>
      ))}
    </div>
  );
}

function DeliveryAttemptsList({
  attempts,
  t,
}: {
  attempts: EventDeliveryAttemptRow[];
  t: Translate;
}) {
  if (attempts.length === 0) {
    return <EmptyRouteState>{t("events.attempts.empty")}</EmptyRouteState>;
  }

  return (
    <div className="eventsWorkList">
      {attempts.map((attempt) => (
        <div className="eventsWorkRow" key={attempt.id}>
          <div>
            <strong>{attempt.declarationId}</strong>
            <span>{t("events.attempts.addon", { addonId: attempt.addonId })}</span>
            <small>{t("events.attempts.number", { attempt: attempt.attemptNumber })}</small>
            <small>{t("events.attempts.requested", { time: timestampLabel(attempt.requestedAt) })}</small>
            {attempt.completedAt ? (
              <small>{t("events.attempts.completed", { time: timestampLabel(attempt.completedAt) })}</small>
            ) : null}
            {attempt.nextRetryAt ? (
              <small>{t("events.attempts.nextRetry", { time: timestampLabel(attempt.nextRetryAt) })}</small>
            ) : null}
            {attempt.replayReasonCode ? (
              <small>{t("events.attempts.replayReason", { reasonCode: attempt.replayReasonCode })}</small>
            ) : null}
          </div>
          <div className="eventsRowBadges">
            <Badge tone={deliveryTone(attempt.status, attempt.hasError)}>{attempt.status}</Badge>
            <Badge tone={attempt.httpStatus && attempt.httpStatus >= 400 ? "danger" : "neutral"}>
              {attempt.httpStatus ? String(attempt.httpStatus) : t("events.none")}
            </Badge>
            <Badge tone={attempt.forcedReplay ? "info" : "neutral"}>
              {attempt.forcedReplay ? t("events.attempts.forcedReplay") : t("events.attempts.normalDelivery")}
            </Badge>
            <Badge tone={attempt.hasError ? "danger" : "neutral"}>
              {attempt.hasError ? t("events.error.present") : t("events.error.absent")}
            </Badge>
          </div>
        </div>
      ))}
    </div>
  );
}

function SummaryTile({
  label,
  tone,
  value,
}: {
  label: string;
  tone: BadgeTone;
  value: number;
}) {
  return (
    <div className="eventsSummaryTile">
      <span>{label}</span>
      <strong>{value}</strong>
      <Badge tone={tone}>{label}</Badge>
    </div>
  );
}

function FactRow({
  badge,
  label,
  tone = "neutral",
  value,
}: {
  badge?: string;
  label: string;
  tone?: BadgeTone;
  value: string;
}) {
  return (
    <div className="eventsFactRow">
      <div>
        <span>{label}</span>
        <strong>{value}</strong>
      </div>
      {badge ? <Badge tone={tone}>{badge}</Badge> : null}
    </div>
  );
}

function emptySchedulerWork(event: EventRow | null): EventSchedulerWorkSummary {
  return {
    event: event ?? {
      id: "",
      kind: "",
      status: "",
      attempts: 0,
      hasPayload: false,
      hasError: false,
      libraryId: null,
      sourceId: null,
      occurredAt: "",
      updatedAt: "",
      nextAttemptAt: null,
    },
    dueWorkCount: 0,
    blockedWorkCount: 0,
    work: [],
  };
}

function eventScope(event: EventRow, t: Translate) {
  if (event.libraryId && event.sourceId) {
    return t("events.scope.librarySource", {
      libraryId: event.libraryId,
      sourceId: event.sourceId,
    });
  }

  if (event.libraryId) {
    return t("events.scope.library", { libraryId: event.libraryId });
  }

  if (event.sourceId) {
    return t("events.scope.source", { sourceId: event.sourceId });
  }

  return t("events.scope.global");
}

function eventStatusTone(status: string, hasError: boolean): BadgeTone {
  if (hasError || status === "failed") {
    return "danger";
  }

  if (status === "pending") {
    return "warning";
  }

  if (status === "running") {
    return "info";
  }

  return "success";
}

function deliveryTone(status: string, hasError: boolean): BadgeTone {
  if (hasError || status === "failed") {
    return "danger";
  }

  if (status === "running") {
    return "info";
  }

  if (status === "pending") {
    return "warning";
  }

  return "success";
}

function schedulerTone(status: string): BadgeTone {
  if (status === "exhausted") {
    return "danger";
  }

  if (status === "due" || status === "retry_due") {
    return "warning";
  }

  if (status === "in_flight") {
    return "info";
  }

  return "neutral";
}

function timestampLabel(value: string) {
  const parsed = Date.parse(value);
  if (Number.isNaN(parsed)) {
    return value || "n/a";
  }

  return new Date(parsed).toISOString();
}

function positiveNumberInput(value: string, fallback: number, min: number) {
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed >= min ? parsed : fallback;
}

function errorMessage(error: unknown, fallback: string) {
  return error instanceof Error && error.message ? error.message : fallback;
}
