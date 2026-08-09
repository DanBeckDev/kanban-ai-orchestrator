import {
  KanbanSquareDashedIcon,
  NetworkIcon,
  Settings2Icon,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

export type MainBoardView = "workflow" | "dependencies" | "settings";

type BoardViewMenuProps = Readonly<{
  activeView: MainBoardView;
  onViewChange: (view: MainBoardView) => void;
}>;

const boardViews: readonly Readonly<{
  label: string;
  value: MainBoardView;
  icon: typeof KanbanSquareDashedIcon;
}>[] = [
  { label: "Workflow", value: "workflow", icon: KanbanSquareDashedIcon },
  { label: "Dependencies", value: "dependencies", icon: NetworkIcon },
  { label: "Settings", value: "settings", icon: Settings2Icon },
];

export function BoardViewMenu({
  activeView,
  onViewChange,
}: BoardViewMenuProps) {
  const active = boardViews.find(({ value }) => value === activeView);
  const ActiveIcon = active?.icon ?? KanbanSquareDashedIcon;

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button id="board-view-menu" type="button" variant="outline">
          <ActiveIcon data-icon="inline-start" />
          {active?.label ?? "Workflow"}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start">
        <DropdownMenuGroup>
          <DropdownMenuLabel>Board view</DropdownMenuLabel>
          <DropdownMenuRadioGroup
            aria-label="Board view"
            onValueChange={(value) => {
              if (isMainBoardView(value)) onViewChange(value);
            }}
            value={activeView}
          >
            {boardViews.map(({ icon: Icon, label, value }) => (
              <DropdownMenuRadioItem key={value} value={value}>
                <Icon />
                {label}
              </DropdownMenuRadioItem>
            ))}
          </DropdownMenuRadioGroup>
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function isMainBoardView(value: string): value is MainBoardView {
  return boardViews.some((boardView) => boardView.value === value);
}
