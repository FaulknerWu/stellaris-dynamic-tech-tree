import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { bootstrapLocale } from "./i18n/controller";
import { loadPersistedData, savePersistedData } from "./app/store";
import "./index.css";
import { App } from "./app/App";

const root = document.getElementById("root");

if (root === null) {
  throw new Error("Missing root element");
}

const persisted = await loadPersistedData();
await bootstrapLocale(persisted.localePreference);
let initialSaveFailed = false;
try { await savePersistedData(persisted); } catch { initialSaveFailed = true; }
createRoot(root).render(
  <StrictMode>
    <App persisted={persisted} initialSaveFailed={initialSaveFailed} />
  </StrictMode>,
);
