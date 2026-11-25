import { svelte } from '@sveltejs/vite-plugin-svelte'
import { defineConfig } from 'vite'

export default defineConfig({
  // GitHub Pages serves the app from /<repo>/.
  base: process.env.BASE_PATH ?? '/',
  plugins: [svelte()],
  optimizeDeps: {
    exclude: ['@myriaddreamin/typst.ts', '@myriaddreamin/typst-ts-renderer', '@myriaddreamin/typst-ts-web-compiler'],
  },
})
