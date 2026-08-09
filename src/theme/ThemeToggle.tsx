import { MoonIcon, SunIcon } from "lucide-react";

import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

import { isTheme } from "./theme-preference";
import { useTheme } from "./ThemeProvider";

export function ThemeToggle() {
  const { theme, setTheme } = useTheme();

  return (
    <div className="appearance-control">
      <span id="appearance-label">Appearance</span>
      <ToggleGroup
        aria-labelledby="appearance-label"
        onValueChange={setSelectedTheme}
        size="sm"
        type="single"
        value={theme}
        variant="outline"
      >
        <ToggleGroupItem aria-label="Dark appearance" value="dark">
          <MoonIcon aria-hidden="true" data-icon="inline-start" />
          Dark
        </ToggleGroupItem>
        <ToggleGroupItem aria-label="Light appearance" value="light">
          <SunIcon aria-hidden="true" data-icon="inline-start" />
          Light
        </ToggleGroupItem>
      </ToggleGroup>
    </div>
  );

  function setSelectedTheme(value: string) {
    if (isTheme(value)) setTheme(value);
  }
}
