import { BoardWorkspace } from "./board/BoardWorkspace";
import type { BoardGateway } from "./board/types";
import { productMetadata } from "./lib/productMetadata";

type AppProps = Readonly<{
  gateway?: BoardGateway;
}>;

export function App({ gateway }: AppProps) {
  return (
    <main className="app-shell">
      <section aria-labelledby="product-title" className="application-frame">
        <header className="application-header">
          <div>
            <p className="eyebrow">Local-first agent coordination</p>
            <h1 id="product-title">{productMetadata.name}</h1>
          </div>
          <dl>
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
        <BoardWorkspace gateway={gateway} />
      </section>
    </main>
  );
}
