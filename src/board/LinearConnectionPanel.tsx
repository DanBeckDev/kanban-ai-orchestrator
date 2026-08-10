import { useState, type FormEvent } from "react";
import { CircleAlertIcon, ExternalLinkIcon } from "lucide-react";

import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";

import {
  commentsAreAuthorized,
  connectedLinearDescription,
  linearLoopbackRedirectUri,
} from "./linearConnectionPresentation";
import type { LinearConnectionStatus, LinearOAuthConfiguration } from "./types";

type LinearConnectionPanelProps = Readonly<{
  busy: boolean;
  status: LinearConnectionStatus;
  onConnect: (configuration: LinearOAuthConfiguration) => Promise<void>;
  onEnableCommentAccess?: () => Promise<void>;
  productManagedConfiguration: LinearOAuthConfiguration | undefined;
}>;

export function LinearConnectionPanel({
  busy,
  status,
  onConnect,
  onEnableCommentAccess,
  productManagedConfiguration,
}: LinearConnectionPanelProps) {
  const [clientId, setClientId] = useState("");

  async function connectSelfManaged(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onConnect({ clientId, redirectUri: linearLoopbackRedirectUri });
  }

  return (
    <section
      aria-labelledby="linear-connection-title"
      className="linear-connection-panel"
    >
      <div>
        <h3 id="linear-connection-title">Connect Linear</h3>
        <p>
          Linear is optional. Start by reading work from Linear; Kanban only
          sends an update after you grant comment access and choose Send.
        </p>
      </div>
      {productManagedConfiguration === undefined ? (
        <ManagedConnectionUnavailable status={status} />
      ) : (
        <ManagedConnectionAction
          busy={busy}
          status={status}
          configuration={productManagedConfiguration}
          onConnect={onConnect}
        />
      )}
      <ConnectionStatus
        busy={busy}
        status={status}
        onEnableCommentAccess={onEnableCommentAccess}
      />
      <SelfManagedConnection
        busy={busy}
        clientId={clientId}
        status={status}
        onClientIdChange={setClientId}
        onConnect={connectSelfManaged}
      />
    </section>
  );
}

function ManagedConnectionUnavailable({
  status,
}: Readonly<{
  status: LinearConnectionStatus;
}>) {
  const detail =
    status.kind === "connected"
      ? "Your existing connection remains available. If your organisation manages a Linear OAuth app, use the self-managed option below to change it."
      : "No new managed connection will be made. If your organisation manages a Linear OAuth app, use the self-managed option below; otherwise continue with local work.";

  return (
    <Alert>
      <CircleAlertIcon aria-hidden="true" />
      <AlertTitle>
        Managed Linear connection is not available in this build
      </AlertTitle>
      <AlertDescription>{detail}</AlertDescription>
    </Alert>
  );
}

function ManagedConnectionAction({
  busy,
  status,
  configuration,
  onConnect,
}: Readonly<{
  busy: boolean;
  status: LinearConnectionStatus;
  configuration: LinearOAuthConfiguration;
  onConnect: (configuration: LinearOAuthConfiguration) => Promise<void>;
}>) {
  return (
    <Card size="sm">
      <CardHeader>
        <CardTitle as="h4">Connect in read-only mode</CardTitle>
        <CardDescription>
          Load and link Linear issues. Kanban will not change Linear.
        </CardDescription>
      </CardHeader>
      <CardFooter>
        <Button
          disabled={busy || status.kind === "awaiting_authorization"}
          onClick={() => void onConnect(configuration)}
          type="button"
        >
          Connect Linear
        </Button>
      </CardFooter>
    </Card>
  );
}

function ConnectionStatus({
  busy,
  status,
  onEnableCommentAccess,
}: Readonly<{
  busy: boolean;
  status: LinearConnectionStatus;
  onEnableCommentAccess?: () => Promise<void>;
}>) {
  const canEnableCommentAccess =
    status.kind === "connected" &&
    !commentsAreAuthorized(status) &&
    onEnableCommentAccess !== undefined;

  return (
    <section aria-label="Linear connection status">
      <p aria-live="polite">{statusDescription(status)}</p>
      {canEnableCommentAccess && (
        <Button
          disabled={busy}
          onClick={() => void onEnableCommentAccess()}
          type="button"
          variant="outline"
        >
          Enable manually sent Linear comments
        </Button>
      )}
    </section>
  );
}

function SelfManagedConnection({
  busy,
  clientId,
  status,
  onClientIdChange,
  onConnect,
}: Readonly<{
  busy: boolean;
  clientId: string;
  status: LinearConnectionStatus;
  onClientIdChange: (clientId: string) => void;
  onConnect: (event: FormEvent<HTMLFormElement>) => Promise<void>;
}>) {
  return (
    <Accordion collapsible type="single">
      <AccordionItem value="self-managed-connection">
        <AccordionTrigger>Use a self-managed Linear app</AccordionTrigger>
        <AccordionContent>
          <Card size="sm">
            <CardHeader>
              <CardTitle as="h4">Self-managed OAuth setup</CardTitle>
              <CardDescription>
                Use this only when your organisation owns the Linear OAuth app.
                Kanban requests read access first, refreshes it only for an
                action when needed, and keeps tokens in this computer&apos;s
                credential store. You can revoke access in Linear at any time.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <form
                aria-label="Set up a self-managed Linear app"
                onSubmit={onConnect}
              >
                <FieldGroup>
                  <Field>
                    <FieldLabel htmlFor="linear-oauth-client-id">
                      OAuth client ID
                    </FieldLabel>
                    <Input
                      autoComplete="off"
                      disabled={
                        busy || status.kind === "awaiting_authorization"
                      }
                      id="linear-oauth-client-id"
                      name="linear-oauth-client-id"
                      onChange={(event) => onClientIdChange(event.target.value)}
                      required
                      value={clientId}
                    />
                  </Field>
                  <Field>
                    <FieldLabel htmlFor="linear-oauth-redirect-uri">
                      Callback URL
                    </FieldLabel>
                    <Input
                      autoComplete="url"
                      id="linear-oauth-redirect-uri"
                      name="linear-oauth-redirect-uri"
                      readOnly
                      value={linearLoopbackRedirectUri}
                    />
                    <FieldDescription>
                      Add this exact callback URL to your Linear OAuth app. It
                      only accepts a local browser callback.
                    </FieldDescription>
                  </Field>
                  <Button
                    disabled={busy || status.kind === "awaiting_authorization"}
                    type="submit"
                  >
                    Connect self-managed app
                  </Button>
                </FieldGroup>
              </form>
            </CardContent>
            <CardFooter>
              <Button asChild size="sm" type="button" variant="link">
                <a
                  href="https://linear.app/developers/oauth-2-0-authentication"
                  rel="noreferrer"
                  target="_blank"
                >
                  <ExternalLinkIcon data-icon="inline-start" />
                  Linear OAuth setup guide
                </a>
              </Button>
            </CardFooter>
          </Card>
        </AccordionContent>
      </AccordionItem>
    </Accordion>
  );
}

function statusDescription(status: LinearConnectionStatus): string {
  switch (status.kind) {
    case "awaiting_authorization":
      return "Finish connecting Linear in your browser. Return here when it is complete.";
    case "connected":
      return connectedLinearDescription(status);
    case "failed":
      return "Kanban could not connect Linear. Reopen setup, check the app details, then connect again.";
    case "disconnected":
      return "No Linear account is connected. Existing local links are unchanged; connect Linear to load issues or choose linked execution.";
  }
}
