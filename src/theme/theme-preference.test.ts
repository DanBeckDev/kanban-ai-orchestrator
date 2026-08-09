import { beforeEach, describe, expect, it } from "vitest";

import {
  applyTheme,
  defaultTheme,
  initialiseTheme,
  readThemePreference,
  saveThemePreference,
  themeStorageKey,
} from "./theme-preference";

describe("theme preference", () => {
  beforeEach(() => {
    document.documentElement.className = "";
    document.head.innerHTML = '<meta content="#10111c" name="theme-color" />';
    window.localStorage.clear();
  });

  it("uses dark when no valid local preference exists", () => {
    window.localStorage.setItem(themeStorageKey, "system");

    expect(readThemePreference(window.localStorage)).toBe(defaultTheme);
    expect(initialiseTheme(document, window.localStorage)).toBe("dark");
    expect(document.documentElement).toHaveClass("dark");
  });

  it("stores and applies the selected appearance to the page and browser chrome", () => {
    saveThemePreference(window.localStorage, "light");
    applyTheme(document, readThemePreference(window.localStorage));

    expect(document.documentElement).toHaveClass("light");
    expect(document.documentElement).not.toHaveClass("dark");
    expect(document.querySelector('meta[name="theme-color"]')).toHaveAttribute(
      "content",
      "#f9f9ff",
    );

    applyTheme(document, "dark");

    expect(document.documentElement).toHaveClass("dark");
    expect(document.querySelector('meta[name="theme-color"]')).toHaveAttribute(
      "content",
      "#10111c",
    );
  });
});
