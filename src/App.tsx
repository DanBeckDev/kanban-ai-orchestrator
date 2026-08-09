import { TooltipProvider } from "./components/ui/tooltip";
import { productMetadata } from "./lib/productMetadata";

import { BoardWorkspace } from "./board/BoardWorkspace";
import type { RepositoryPicker } from "./board/BoardSetup";
import type { BoardGateway } from "./board/types";

type AppProps = Readonly<{
  gateway?: BoardGateway;
  repositoryPicker?: RepositoryPicker;
}>;

export function App({ gateway, repositoryPicker }: AppProps) {
  return (
    <TooltipProvider>
      <main className="app-shell">
        <section aria-labelledby="product-title" className="application-frame">
          <header className="application-header">
            <div>
              <p className="eyebrow">Local-first agent coordination</p>
              <h1 id="product-title">{productMetadata.name}</h1>
            </div>
            <dl aria-label="Application status" className="application-status">
              <div>
                <dt>Execution authority</dt>
                <dd>Rust local core</dd>
              </div>
              <div>
                <dt>Current milestone</dt>
                <dd>{productMetadata.milestone}</dd>
              </div>
            </dl>
          </header>
          <BoardWorkspace
            gateway={gateway}
            repositoryPicker={repositoryPicker}
          />
        </section>
      </main>
    </TooltipProvider>
  );
}
