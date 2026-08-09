export type AgentEffort =
  | "provider_default"
  | "focused"
  | "balanced"
  | "thorough";

export type AgentModelPreference =
  | Readonly<{ kind: "provider_default" }>
  | Readonly<{ kind: "named"; name: string }>;

export type OrganiserDefaults = Readonly<{
  plannerProfileName: string;
  model: AgentModelPreference;
  effort: AgentEffort;
}>;

export type TicketWorkerDefaults = Readonly<{
  agentProfileName: string;
  model: AgentModelPreference;
  effort: AgentEffort;
}>;

export type ProjectAgentSettings = Readonly<{
  projectId: string;
  organiser?: OrganiserDefaults;
  ticketWorker?: TicketWorkerDefaults;
}>;

export type SaveProjectAgentSettingsRequest = Readonly<{
  boardId: string;
  organiser?: OrganiserDefaults;
  ticketWorker?: TicketWorkerDefaults;
}>;
