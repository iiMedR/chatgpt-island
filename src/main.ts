import "./styles.css";
import { showLauncher } from "./launcher";
import { showConfirmation, showSettings, showHandoff } from "./trusted-ui";
const view = new URLSearchParams(window.location.search).get("view");
if (view === "confirmation") void showConfirmation();
else if (view === "settings") void showSettings();
else if (view === "handoff") void showHandoff();
else showLauncher();
