import { useState, type FormEvent } from "react";

import { Button } from "../../components/ui/Button";
import type { MediaConnection } from "./mediaDataSource";
import { useMediaSession } from "./MediaSession";

export function MediaConnectPage() {
  const { connect, connectionError, connecting } = useMediaSession();
  const [baseUrl, setBaseUrl] = useState("http://127.0.0.1:3000");
  const [bearerToken, setBearerToken] = useState("");

  async function submitLiveConnection(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const connection: MediaConnection = {
      mode: "live",
      baseUrl: baseUrl.trim(),
      bearerToken,
    };
    await connect(connection);
  }

  return (
    <section className="mediaConnect" aria-labelledby="media-connect-title">
      <div className="mediaConnectIntro">
        <p className="mediaKicker">Connect</p>
        <h2 id="media-connect-title">Enter a Nako server</h2>
        <p>
          The token stays in memory for this browser session. Fixture mode uses
          local development data.
        </p>
      </div>
      <form className="mediaConnectForm" onSubmit={submitLiveConnection}>
        <label>
          <span>Server URL</span>
          <input
            autoComplete="url"
            onChange={(event) => setBaseUrl(event.currentTarget.value)}
            required
            type="url"
            value={baseUrl}
          />
        </label>
        <label>
          <span>Access token</span>
          <input
            autoComplete="off"
            onChange={(event) => setBearerToken(event.currentTarget.value)}
            required
            type="password"
            value={bearerToken}
          />
        </label>
        {connectionError ? <div className="mediaError">{connectionError}</div> : null}
        <div className="mediaConnectActions">
          <Button disabled={connecting} type="submit">
            {connecting ? "Connecting" : "Connect"}
          </Button>
          <Button onClick={() => void connect({ mode: "fixture" })} type="button" variant="outline">
            Use fixture demo
          </Button>
        </div>
      </form>
    </section>
  );
}
