import { foundationMessage, productMetadata } from "./lib/productMetadata";

export function App() {
  return (
    <main className="app-shell">
      <section aria-labelledby="product-title" className="foundation-card">
        <p className="eyebrow">Local-first agent coordination</p>
        <h1 id="product-title">{productMetadata.name}</h1>
        <p>{foundationMessage(productMetadata)}</p>
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
      </section>
    </main>
  );
}
