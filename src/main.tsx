import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { initialiseTheme } from "./theme/theme-preference";
import "./styles.css";

const rootElement = document.getElementById("root");

if (rootElement === null) {
  throw new Error("The application root is missing.");
}

initialiseTheme(document, window.localStorage);

createRoot(rootElement).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
