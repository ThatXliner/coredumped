import init from './pkg/coredumped_web.js';

async function run() {
    await init();
}

run().catch(console.error);
