import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { EmptyRouteState, RouteNotice, RoutePage } from "./layout/RoutePage";
import { DataPanel } from "./ui/DataPanel";
import { FilterActions, FilterBar, FilterField } from "./ui/FilterBar";
import { RowsSkeleton } from "./ui/RowsSkeleton";

describe("Admin Web V2 shared components", () => {
  it("composes route headers, safe notices, and empty states", () => {
    render(
      <RoutePage
        actions={<button type="button">Refresh</button>}
        description="Route-owned diagnostics."
        kicker="Operations"
        status={<span>Mock fallback</span>}
        title="Jobs"
        titleId="jobs-title"
      >
        <RouteNotice>Admin API request failed. Showing fallback data.</RouteNotice>
        <EmptyRouteState>No jobs match the current filters.</EmptyRouteState>
      </RoutePage>,
    );

    expect(screen.getByRole("heading", { name: "Jobs" })).toBeInTheDocument();
    expect(screen.getByText("Route-owned diagnostics.")).toBeInTheDocument();
    expect(screen.getByText("Mock fallback")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Refresh" })).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent("fallback data");
    expect(screen.getByText("No jobs match the current filters.")).toBeInTheDocument();
  });

  it("keeps filter and data-panel structure accessible", () => {
    render(
      <>
        <FilterBar label="Job filters">
          <FilterField label="Status">
            <select aria-label="Job status filter">
              <option>Any status</option>
            </select>
          </FilterField>
          <FilterActions>
            <button type="button">Clear</button>
          </FilterActions>
        </FilterBar>
        <DataPanel
          description="3 returned, offset 0, limit 20"
          headerAccessory={<span>URL filters are authoritative</span>}
          title="Job queue"
        >
          <RowsSkeleton label="Loading jobs" rows={2} />
        </DataPanel>
      </>,
    );

    expect(screen.getByLabelText("Job status filter")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Clear" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Job queue" })).toBeInTheDocument();
    expect(screen.getByText("3 returned, offset 0, limit 20")).toBeInTheDocument();
    expect(screen.getByRole("status", { name: "Loading jobs" })).toBeInTheDocument();
  });
});
