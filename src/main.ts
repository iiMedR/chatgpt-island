import "./styles.css";

import { showConfirmation, showSettings, showHandoff } from "./trusted-ui";
const view = new URLSearchParams(window.location.search).get("view");
if (view === "confirmation") void showConfirmation();
else if (view === "settings") void showSettings();
else if (view === "handoff") void showHandoff();
else if (import.meta.env.DEV) void import("./launcher").then(module => module.showLauncher());
