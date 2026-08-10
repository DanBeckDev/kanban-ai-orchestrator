import { useState, type FormEvent } from "react";

import type { LinearConnectionStatus, LinearOAuthConfiguration } from "./types";

const defaultRedirectUri = "http://127.0.0.1:38471/linear/oauth/callback";

type LinearConnectionPanelProps = Readonly<{
  busy: boolean;
  status: LinearConnectionStatus;
  onConnect: (configuration: LinearOAuthConfiguration) => Promise<void>;
  onEnableCommentAccess?: () => Promise<void>;
}>;

export function LinearConnectionPanel({
  busy,
  status,
  onConnect,
  onEnableCommentAccess,
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
            autoComplete="off"
            name="linear-oauth-client-id"
            required
            value={clientId}
            onChange={(event) => setClientId(event.target.value)}
          />
        </label>
        <label>
          Redirect URI
          <input
            autoComplete="url"
            name="linear-oauth-redirect-uri"
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
      {status.kind === "connected" &&
        !status.scopes.includes("comments:create") &&
        !status.scopes.includes("write") &&
        onEnableCommentAccess !== undefined && (
          <button
            disabled={busy}
            type="button"
            onClick={() => void onEnableCommentAccess()}
          >
            Enable manually sent Linear comments
          </button>
        )}
      <p aria-live="polite">{statusDescription(status)}</p>
    </section>
  );
}

function statusDescription(status: LinearConnectionStatus): string {
  switch (status.kind) {
    case "awaiting_authorization":
      return "Finish connecting Linear in your browser. Return here when it is complete.";
    case "connected":
      return "Connected. You can now load Linear issues and choose what to share.";
    case "failed":
      return "Kanban could not connect Linear. Check the app setup, then try again.";
    case "disconnected":
      return "No Linear account is connected. Connect Linear to load issues or share updates.";
  }
}
