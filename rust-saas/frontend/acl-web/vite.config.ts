import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers'

export default defineConfig({
  base: '/',
  plugins: [
    vue(),
    AutoImport({
      resolvers: [ElementPlusResolver()]
    }),
    Components({
      resolvers: [ElementPlusResolver()]
    })
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  build: {
    target: 'es2020',
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true,
        pure_funcs: ['console.log', 'console.debug', 'console.info']
      },
      format: {
        comments: false
      }
    },
    rollupOptions: {
      output: {
        manualChunks: {
          'element-plus': ['element-plus'],
          'vue-flow': ['@vue-flow/core'],
          'maxgraph': ['@maxgraph/core'],
          'highlight': ['highlight.js', '@highlightjs/vue-plugin'],
          'axios': ['axios'],
          'pinia': ['pinia', 'pinia-plugin-persistedstate'],
          'vue-router': ['vue-router'],
          'lucide': ['lucide-vue-next'],
          'vue': ['vue']
        },
        compact: true
      }
    },
    chunkSizeWarningLimit: 1000,
    reportCompressedSize: true
  },
  server: {
    host: true,
    port: 3000,
    hmr: {
      path: '/hmr'
    },
    allowedHosts: ['localhost','895my83le516.vicp.fun','127.0.0.1'],
    https: false,
    proxy: {
      '/api': {
        target: 'http://localhost:8089',
        changeOrigin: true
      }
    }
  }
})
