import { defineConfig, loadEnv } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd())

  return {
    server: {
      host: env.VITE_DEV_HOST || 'localhost',
      port: 5173,
      allowedHosts: true,
      proxy: {
        '/api': {
          target: env.VITE_API_PROXY,
          changeOrigin: true,
        },
      },
    },
    plugins: [svelte()],
  }
})
