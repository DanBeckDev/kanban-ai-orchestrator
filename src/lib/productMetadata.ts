export type ProductMetadata = Readonly<{
  name: string;
  milestone: string;
}>;

export const productMetadata: ProductMetadata = {
  name: "Kanban AI Orchestrator",
  milestone: "Foundation",
};

export function foundationMessage(metadata: ProductMetadata): string {
  return `${metadata.name} is ready for its durable local core.`;
}
