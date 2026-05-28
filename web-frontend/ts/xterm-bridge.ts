import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { WebglAddon } from "@xterm/addon-webgl";
import "@xterm/xterm/css/xterm.css";

export class XtermBridge {
    private terminal: Terminal;
    private fitAddon: FitAddon;
    private keyCallback: ((key: string) => void) | null = null;
    private resizeCallback: ((cols: number, rows: number) => void) | null =
        null;

    constructor(containerId: string) {
        const fontSize = this.getResponsiveFontSize();

        this.terminal = new Terminal({
            fontFamily:
                '"JetBrains Mono", "Cascadia Code", "Fira Code", monospace',
            fontSize: fontSize,
            theme: {
                background: "#0d0d0d",
                foreground: "#f0f0f0",
                cursor: "#f0f0f0",
            },
            cursorStyle: "block",
            cursorBlink: false,
            allowTransparency: true,
            cols: 100,
            rows: 40,
        });

        this.fitAddon = new FitAddon();
        this.terminal.loadAddon(this.fitAddon);

        const container = document.getElementById(containerId);
        if (!container) {
            throw new Error(`Container element '${containerId}' not found`);
        }
        this.terminal.open(container);

        try {
            const webglAddon = new WebglAddon();
            this.terminal.loadAddon(webglAddon);
        } catch (e) {
            console.warn(
                "WebGL addon failed to load, using canvas renderer:",
                e
            );
        }

        this.fitAddon.fit();
        this.terminal.focus();

        this.terminal.onKey(({ domEvent }) => {
            if (this.keyCallback) {
                let keyName = domEvent.key;
                if (keyName.length === 1 && domEvent.shiftKey) {
                    keyName = keyName.toUpperCase();
                }
                this.keyCallback(keyName);
            }
        });

        window.addEventListener("resize", () => {
            const newFontSize = this.getResponsiveFontSize();
            if (this.terminal.options.fontSize !== newFontSize) {
                this.terminal.options.fontSize = newFontSize;
            }
            this.fitAddon.fit();
            if (this.resizeCallback) {
                this.resizeCallback(this.terminal.cols, this.terminal.rows);
            }
        });
    }

    private getResponsiveFontSize(): number {
        const width = window.innerWidth;
        const height = window.innerHeight;
        const isLandscape = width > height;

        if (isLandscape && height <= 450) return 8;
        if (isLandscape && height <= 550) return 9;
        if (isLandscape && height <= 700) return 10;
        if (width <= 400) return 9;
        if (width <= 500) return 10;
        if (width <= 700) return 11;
        if (width <= 900) return 12;
        return 14;
    }

    write(data: string): void {
        this.terminal.write(data);
    }

    clear(): void {
        this.terminal.clear();
    }

    resize(cols: number, rows: number): void {
        this.terminal.resize(cols, rows);
    }

    cols(): number {
        return this.terminal.cols;
    }

    rows(): number {
        return this.terminal.rows;
    }

    setKeyCallback(callback: (key: string) => void): void {
        this.keyCallback = callback;
    }

    setResizeCallback(callback: (cols: number, rows: number) => void): void {
        this.resizeCallback = callback;
    }
}
