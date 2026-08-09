import { describe, expect, it, vi } from "vitest";

const { open } = vi.hoisted(() => ({ open: vi.fn() }));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open }));

import { selectRepository } from "./repositoryPicker";

describe("selectRepository", () => {
  it("requests one native directory and preserves cancellation", async () => {
    open.mockResolvedValueOnce(null);

    await expect(selectRepository()).resolves.toBeNull();
    expect(open).toHaveBeenCalledWith({
      directory: true,
      multiple: false,
      title: "Choose a local Git repository",
    });
  });
});
