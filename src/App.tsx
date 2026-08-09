import { TooltipProvider } from "./components/ui/tooltip";
import { productMetadata } from "./lib/productMetadata";
import { ThemeProvider } from "./theme/ThemeProvider";
import { ThemeToggle } from "./theme/ThemeToggle";

import { BoardWorkspace } from "./board/BoardWorkspace";
import type { RepositoryPicker } from "./board/BoardSetup";
import type { BoardGateway } from "./board/types";

type AppProps = Readonly<{
  gateway?: BoardGateway;
  repositoryPicker?: RepositoryPicker;
}>;

export function App({ gateway, repositoryPicker }: AppProps) {
  return (
    <ThemeProvider>
      <TooltipProvider>
        <a className="skip-link" href="#board-content">
          Skip to board
        </a>
        <div className="app-shell">
          <div className="application-frame">
            <header className="application-header">
              <div>
                <h1 id="product-title">{productMetadata.name}</h1>
                <p>Plan &amp; oversee agent work.</p>
              </div>
              <ThemeToggle />
            </header>
            <main id="board-content" tabIndex={-1}>
              <BoardWorkspace
                gateway={gateway}
                repositoryPicker={repositoryPicker}
              />
            </main>
          </div>
        </div>
      </TooltipProvider>
    </ThemeProvider>
  );
}
