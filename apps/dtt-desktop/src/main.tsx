import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import "./i18n";
import "./index.css";
import { App } from "./app/App";

const root = document.getElementById("root");

if (root === null) {
  throw new Error("缺少应用挂载节点");
}

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
