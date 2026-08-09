export type ProductMetadata = Readonly<{
  name: string;
  milestone: string;
}>;

export const productMetadata: ProductMetadata = {
  name: "Kanban AI Orchestrator",
  milestone: "Local board core",
};

export function foundationMessage(metadata: ProductMetadata): string {
  return `${metadata.name} persists local projects, boards, tasks, and dependencies through its Rust core.`;
}
