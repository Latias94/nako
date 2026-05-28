import { StrictMode } from "react"
import { createRoot } from "react-dom/client"
import { AppRoot } from "./app-root"
import "./styles/globals.css"

const rootElement = document.getElementById("root")

if (!rootElement) {
  throw new Error("Nako web root element was not found.")
}

createRoot(rootElement).render(
  <StrictMode>
    <AppRoot />
  </StrictMode>,
)
