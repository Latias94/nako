import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "./App";
import { createLazyAdminDataSource } from "./adminApi/lazyDataSource";
import "./design/tokens.css";
import "./styles.css";

const dataSource = createLazyAdminDataSource({
  baseUrl: import.meta.env.VITE_NAKO_ADMIN_API_BASE_URL,
});

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App dataSource={dataSource} />
  </StrictMode>,
);
