import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { App } from "../../App";
import type { MediaDataSourceFactory, MediaWebDataSource } from "./mediaDataSource";
import { createFixtureMediaDataSource } from "./mediaDataSource";

afterEach(() => {
  window.history.pushState(null, "", "/");
});

describe("Media Web surface", () => {
  it("renders the Media connect surface under the shared app shell", async () => {
    window.history.pushState(null, "", "/media");

    render(<App dataSource={emptyAdminDataSource()} />);

    expect(await screen.findByRole("heading", { name: "Enter a Nako server" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /Admin/ })).toBeInTheDocument();
    expect(screen.getByLabelText("Server URL")).toBeInTheDocument();
    expect(screen.queryByText("Admin API")).not.toBeInTheDocument();
  });

  it("connects through the live Public Client data source without rendering the token", async () => {
    window.history.pushState(null, "", "/media/libraries");
    const dataSource = createFixtureMediaDataSource();
    const checkConnection = vi.fn(async () => undefined);
    const factory = vi.fn((connection) => ({
      ...dataSource,
      checkConnection,
      label: connection.mode === "live" ? "Live Public Client API" : dataSource.label,
      source: connection.mode === "live" ? "live" : dataSource.source,
    }) as MediaWebDataSource) satisfies MediaDataSourceFactory;

    const { container } = render(
      <App dataSource={emptyAdminDataSource()} mediaDataSourceFactory={factory} />,
    );

    fireEvent.change(await screen.findByLabelText("Server URL"), {
      target: { value: "http://nako.test" },
    });
    fireEvent.change(screen.getByLabelText("Access token"), {
      target: { value: "secret-token" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Connect" }));

    expect(await screen.findByRole("heading", { name: "Media Libraries" })).toBeInTheDocument();
    expect(screen.getAllByText("Live Public Client API").length).toBeGreaterThan(0);
    expect(await screen.findByText("Anime Vault")).toBeInTheDocument();
    expect(factory).toHaveBeenCalledWith({
      baseUrl: "http://nako.test",
      bearerToken: "secret-token",
      mode: "live",
    });
    expect(container.textContent).not.toContain("secret-token");
  });

  it("keeps fixture mode visible for development data", async () => {
    window.history.pushState(null, "", "/media/libraries");

    render(<App dataSource={emptyAdminDataSource()} />);

    fireEvent.click(await screen.findByRole("button", { name: "Use fixture demo" }));

    expect(await screen.findByRole("heading", { name: "Media Libraries" })).toBeInTheDocument();
    expect(screen.getAllByText("Fixture mode").length).toBeGreaterThan(0);
    expect(await screen.findByText("Films")).toBeInTheDocument();
  });
});

function emptyAdminDataSource() {
  return {
    async load() {
      throw new Error("admin data source should not load for media routes");
    },
  };
}
