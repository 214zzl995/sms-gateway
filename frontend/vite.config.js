import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vite.dev/config/
export default defineConfig({
  server: {
    host: '192.168.0.22', 
    port: 5173, 
  },
  plugins: [svelte()],
})
