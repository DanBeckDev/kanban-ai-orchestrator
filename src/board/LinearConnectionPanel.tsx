import { useState, type FormEvent } from "react";

import type { LinearConnectionStatus, LinearOAuthConfiguration } from "./types";

const defaultRedirectUri = "http://127.0.0.1:38471/linear/oauth/callback";

type LinearConnectionPanelProps = Readonly<{
  busy: boolean;
  status: LinearConnectionStatus;
  onConnect: (configuration: LinearOAuthConfiguration) => Promise<void>;
}>;

export function LinearConnectionPanel({
  busy,
  status,
  onConnect,
}: LinearConnectionPanelProps) {
  const [clientId, setClientId] = useState("");
  const [redirectUri, setRedirectUri] = useState(defaultRedirectUri);

  async function connect(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onConnect({ clientId, redirectUri });
  }

  return (
    <section className="linear-connection-panel">
      <h3>Connect Linear</h3>
      <p>
        Create a Linear OAuth app with this exact loopback redirect URI. The
        browser opens securely with PKCE; tokens stay in your operating system’s
        credential store.
      </p>
      <form aria-label="Connect Linear" onSubmit={connect}>
        <label>
          OAuth client ID
          <input
            required
            value={clientId}
            onChange={(event) => setClientId(event.target.value)}
          />
        </label>
        <label>
          Redirect URI
          <input
            required
            type="url"
            value={redirectUri}
            onChange={(event) => setRedirectUri(event.target.value)}
          />
        </label>
        <button
          disabled={busy || status.kind === "awaiting_authorization"}
          type="submit"
        >
          Connect Linear
        </button>
      </form>
      <p aria-live="polite">{statusDescription(status)}</p>
    </section>
  );
}

function statusDescription(status: LinearConnectionStatus): string {
  switch (status.kind) {
    case "awaiting_authorization":
      return "Waiting for Linear authorization in your browser.";
    case "connected":
      return `Connected with ${status.scopes.join(", ")} access; token expires ${status.expiresAt}.`;
    case "failed":
      return `Connection failed: ${status.message}`;
    case "disconnected":
      return "No Linear account is connected.";
  }
}
