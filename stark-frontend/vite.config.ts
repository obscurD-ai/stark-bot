import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

// Docker: backend on port 8080 (API + WS), frontend dev server on 8080
// Local:  backend on port 8080 (API + WS), frontend dev server on 5173
const isDocker = process.env.NODE_ENV === 'development' && process.env.DOCKER === '1';
const backendTarget = isDocker ? 'http://backend:8080' : 'http://localhost:8080';
const wsTarget = isDocker ? 'ws://backend:8080' : 'ws://localhost:8080';
const serverPort = isDocker ? 8080 : 5173;

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src')
    }
  },
  server: {
    port: serverPort,
    host: isDocker ? '0.0.0.0' : 'localhost',
    proxy: {
      '/api': backendTarget,
      '/ws': {
        target: wsTarget,
        ws: true,
        changeOrigin: true
      }
    }
  }
});
