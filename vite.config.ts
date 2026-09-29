import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

// https://vite.dev/config/
// Tauri expects a fixed dev server port.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      // Tauri handles the Rust side; don't watch it from Vite.
      ignored: ['**/src-tauri/**'],
    },
  },
})
