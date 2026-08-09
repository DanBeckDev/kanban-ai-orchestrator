export function planDraft() {
  return {
    workItems: [
      {
        id: "foundation",
        title: "Foundation",
        description: "Create the shared contract.",
        acceptanceCriteria: ["Contract is verified."],
      },
      {
        id: "interface",
        title: "Interface",
        description: "Use the shared contract.",
        acceptanceCriteria: ["Interface is verified."],
        requiresHumanReview: true,
      },
    ],
    dependencies: [
      {
        id: "foundation-interface",
        upstreamWorkItemId: "foundation",
        downstreamWorkItemId: "interface",
        kind: "blocks" as const,
        reason: "The interface needs the shared contract.",
        owner: "orchestrator",
        nextAction: "Finish the foundation task.",
      },
    ],
    unresolvedAssumptions: ["The local base branch exists."],
  };
}
