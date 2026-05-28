import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm";

export default defineConfig({
    base: "/coredumped/",
    plugins: [wasm()],
    build: {
        target: "esnext",
    },
});
