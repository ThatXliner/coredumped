import "./style.css";
import { XtermBridge } from "./xterm-bridge";

// Export for WASM to use
(window as unknown as { XtermBridge: typeof XtermBridge }).XtermBridge =
    XtermBridge;

// Initialize WASM
async function init() {
    const wasm = await import("../pkg/coredumped_web.js");
    await wasm.default();
}

init().catch(console.error);
