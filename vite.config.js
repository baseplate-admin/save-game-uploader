import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [sveltekit()],
    esbuild: {
        target: 'esnext',
        legalComments: 'external',
    },
    css: {
        devSourcemap: true,
        // Switch to lightning.css when tailwind supports it
        transformer: 'postcss',
    },
    build: {
        commonjsOptions: {
            transformMixedEsModules: true,
        },
        chunkSizeWarningLimit: 2048,
        emptyOutDir: true,
        target: 'esnext',
        cssTarget: 'esnext',
        minify: 'terser',
        //sourcemap: true
    },
    worker: {
        format: 'es',
    },

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent vite from obscuring rust errors
    clearScreen: false,
    // 2. tauri expects a fixed port, fail if that port is not available
    server: {
        port: 1420,
        strictPort: true,
        host: host || false,
        hmr: host
            ? {
                  protocol: 'ws',
                  host,
                  port: 1421,
              }
            : undefined,
        watch: {
            // 3. tell vite to ignore watching `src-tauri`
            ignored: ['**/src-tauri/**'],
        },
    },
});
