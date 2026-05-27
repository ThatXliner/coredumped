import init from './pkg/xlyph_tui.js';

async function run() {
    await init();
}

run().catch(console.error);
