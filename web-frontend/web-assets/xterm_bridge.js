export class XtermBridge {
    constructor(containerId) {
        this.terminal = new Terminal({
            fontFamily: '"JetBrains Mono", "Cascadia Code", "Fira Code", monospace',
            fontSize: 14,
            theme: {
                background: '#0d0d0d',
                foreground: '#f0f0f0',
                cursor: '#f0f0f0',
            },
            cursorStyle: 'block',
            cursorBlink: false,
            allowTransparency: true,
            cols: 100,
            rows: 40,
        });

        this.fitAddon = new FitAddon.FitAddon();
        this.terminal.loadAddon(this.fitAddon);

        const container = document.getElementById(containerId);
        this.terminal.open(container);

        try {
            const webglAddon = new WebglAddon.WebglAddon();
            this.terminal.loadAddon(webglAddon);
        } catch (e) {
            console.warn('WebGL addon failed to load, using canvas renderer:', e);
        }

        this.fitAddon.fit();
        this.terminal.focus();

        this.keyCallback = null;
        this.resizeCallback = null;

        this.terminal.onKey(({ key, domEvent }) => {
            if (this.keyCallback) {
                let keyName = domEvent.key;
                if (keyName.length === 1 && domEvent.shiftKey) {
                    keyName = keyName.toUpperCase();
                }
                this.keyCallback(keyName);
            }
        });

        window.addEventListener('resize', () => {
            this.fitAddon.fit();
            if (this.resizeCallback) {
                this.resizeCallback(this.terminal.cols, this.terminal.rows);
            }
        });
    }

    write(data) {
        this.terminal.write(data);
    }

    clear() {
        this.terminal.clear();
    }

    resize(cols, rows) {
        this.terminal.resize(cols, rows);
    }

    cols() {
        return this.terminal.cols;
    }

    rows() {
        return this.terminal.rows;
    }

    setKeyCallback(callback) {
        this.keyCallback = callback;
    }

    setResizeCallback(callback) {
        this.resizeCallback = callback;
    }
}
