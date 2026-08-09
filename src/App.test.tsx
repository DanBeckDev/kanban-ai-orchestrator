import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { App } from "./App";
import { themeStorageKey } from "./theme/theme-preference";

describe("App", () => {
  beforeEach(() => {
    window.localStorage.clear();
    document.documentElement.className = "dark";
  });

  it("starts in dark appearance with concise product copy", () => {
    render(<App />);

    expect(screen.getByRole("heading", { name: "Kanban" })).toBeVisible();
    expect(screen.getByText("Plan & oversee agent work.")).toBeVisible();
    expect(screen.queryByText("Execution authority")).not.toBeInTheDocument();
    expect(screen.queryByText("Current milestone")).not.toBeInTheDocument();
    expect(document.documentElement).toHaveClass("dark");
    expect(window.localStorage.getItem(themeStorageKey)).toBe("dark");

    fireEvent.click(screen.getByRole("radio", { name: "Dark appearance" }));

    expect(document.documentElement).toHaveClass("dark");
  });

  it("restores and changes a local appearance preference", () => {
    window.localStorage.setItem(themeStorageKey, "light");
    render(<App />);

    expect(document.documentElement).toHaveClass("light");
    fireEvent.click(screen.getByRole("radio", { name: "Dark appearance" }));

    expect(document.documentElement).toHaveClass("dark");
    expect(document.documentElement).not.toHaveClass("light");
    expect(window.localStorage.getItem(themeStorageKey)).toBe("dark");

    fireEvent.click(screen.getByRole("radio", { name: "Light appearance" }));

    expect(document.documentElement).toHaveClass("light");
    expect(window.localStorage.getItem(themeStorageKey)).toBe("light");
  });
});
