import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "./App";
import { createAdminDataSource } from "./adminApi/dataSource";
import "./styles.css";

const dataSource = createAdminDataSource({
  baseUrl: import.meta.env.VITE_NAKO_ADMIN_API_BASE_URL,
});

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App dataSource={dataSource} />
  </StrictMode>,
);
