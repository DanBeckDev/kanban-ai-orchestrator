export type TicketEffectAction =
  | "refine_specification"
  | "give_worker_guidance"
  | "prepare_start"
  | "prepare_restart"
  | "explain_evidence"
  | "return_for_correction"
  | "recover_interrupted";

export type TicketEffectOutcome =
  | "pending"
  | "awaiting_approval"
  | "applied"
  | "rejected"
  | "cancelled"
  | "denied"
  | "stale"
  | "recovered";

export type TicketEffectResolution = "apply" | "reject" | "cancel";

export type TicketEffectProposal = Readonly<{
  title?: string;
  description?: string;
  acceptanceCriteria: readonly string[];
  workerGuidance?: string;
  evidenceExplanation?: string;
}>;

export type TicketEffect = Readonly<{
  id: string;
  boardId: string;
  workItemId: string;
  organiserProfileName: string;
  action: TicketEffectAction;
  promptSummary: string;
  recommendation: string;
  rationale: string;
  proposal: TicketEffectProposal;
  authorityMode: "manual" | "autonomous";
  supervisionRevision?: number;
  policyResult: "not_required" | "allowed" | "denied";
  outcome: TicketEffectOutcome;
  expectedWorkItemSequence: number;
  recordedAt: string;
  outcomeAt?: string;
}>;

export type TicketEffectPromptRequest = Readonly<{
  requestId: string;
  workItemId: string;
  action: TicketEffectAction;
  prompt: string;
}>;

export type ResolveTicketEffectRequest = Readonly<{
  effectId: string;
  resolution: TicketEffectResolution;
}>;
