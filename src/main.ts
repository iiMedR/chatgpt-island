import "./styles.css";
import { showLauncher } from "./launcher";
import { showConfirmation, showSettings } from "./trusted-ui";
const view = new URLSearchParams(window.location.search).get("view");
if (view === "confirmation") void showConfirmation();
else if (view === "settings") void showSettings();
else showLauncher();
