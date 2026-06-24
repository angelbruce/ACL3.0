// vite.config.ts
import { defineConfig } from "file:///J:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/frontend/acl-web/node_modules/vite/dist/node/index.js";
import vue from "file:///J:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/frontend/acl-web/node_modules/@vitejs/plugin-vue/dist/index.mjs";
import { fileURLToPath, URL } from "node:url";
import AutoImport from "file:///J:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/frontend/acl-web/node_modules/unplugin-auto-import/dist/vite.mjs";
import Components from "file:///J:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/frontend/acl-web/node_modules/unplugin-vue-components/dist/vite.mjs";
import { ElementPlusResolver } from "file:///J:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/frontend/acl-web/node_modules/unplugin-vue-components/dist/resolvers.mjs";
var __vite_injected_original_import_meta_url = "file:///J:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/frontend/acl-web/vite.config.ts";
var vite_config_default = defineConfig({
  base: "/",
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
      "@": fileURLToPath(new URL("./src", __vite_injected_original_import_meta_url))
    }
  },
  build: {
    target: "es2020",
    minify: "terser",
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true,
        pure_funcs: ["console.log", "console.debug", "console.info"]
      },
      format: {
        comments: false
      }
    },
    rollupOptions: {
      output: {
        manualChunks: {
          "element-plus": ["element-plus"],
          "vue-flow": ["@vue-flow/core"],
          "maxgraph": ["@maxgraph/core"],
          "highlight": ["highlight.js", "@highlightjs/vue-plugin"],
          "axios": ["axios"],
          "pinia": ["pinia", "pinia-plugin-persistedstate"],
          "vue-router": ["vue-router"],
          "lucide": ["lucide-vue-next"],
          "vue": ["vue"]
        },
        compact: true
      }
    },
    chunkSizeWarningLimit: 1e3,
    reportCompressedSize: true
  },
  server: {
    host: true,
    port: 3e3,
    hmr: {
      path: "/hmr"
    },
    allowedHosts: ["localhost", "895my83le516.vicp.fun", "127.0.0.1"],
    https: false,
    proxy: {
      "/api": {
        target: "http://localhost",
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, "")
      }
    }
  }
});
export {
  vite_config_default as default
};
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsidml0ZS5jb25maWcudHMiXSwKICAic291cmNlc0NvbnRlbnQiOiBbImNvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9kaXJuYW1lID0gXCJKOlxcXFxsbGFtYV9jcHBcXFxccHJvamVjdFxcXFxBQ0wzLjBNXFxcXGdpdGh1YlxcXFxBQ0wzLjBcXFxccnVzdC1zYWFzXFxcXGZyb250ZW5kXFxcXGFjbC13ZWJcIjtjb25zdCBfX3ZpdGVfaW5qZWN0ZWRfb3JpZ2luYWxfZmlsZW5hbWUgPSBcIko6XFxcXGxsYW1hX2NwcFxcXFxwcm9qZWN0XFxcXEFDTDMuME1cXFxcZ2l0aHViXFxcXEFDTDMuMFxcXFxydXN0LXNhYXNcXFxcZnJvbnRlbmRcXFxcYWNsLXdlYlxcXFx2aXRlLmNvbmZpZy50c1wiO2NvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9pbXBvcnRfbWV0YV91cmwgPSBcImZpbGU6Ly8vSjovbGxhbWFfY3BwL3Byb2plY3QvQUNMMy4wTS9naXRodWIvQUNMMy4wL3J1c3Qtc2Fhcy9mcm9udGVuZC9hY2wtd2ViL3ZpdGUuY29uZmlnLnRzXCI7aW1wb3J0IHsgZGVmaW5lQ29uZmlnIH0gZnJvbSAndml0ZSdcbmltcG9ydCB2dWUgZnJvbSAnQHZpdGVqcy9wbHVnaW4tdnVlJ1xuaW1wb3J0IHsgZmlsZVVSTFRvUGF0aCwgVVJMIH0gZnJvbSAnbm9kZTp1cmwnXG5pbXBvcnQgQXV0b0ltcG9ydCBmcm9tICd1bnBsdWdpbi1hdXRvLWltcG9ydC92aXRlJ1xuaW1wb3J0IENvbXBvbmVudHMgZnJvbSAndW5wbHVnaW4tdnVlLWNvbXBvbmVudHMvdml0ZSdcbmltcG9ydCB7IEVsZW1lbnRQbHVzUmVzb2x2ZXIgfSBmcm9tICd1bnBsdWdpbi12dWUtY29tcG9uZW50cy9yZXNvbHZlcnMnXG5cbmV4cG9ydCBkZWZhdWx0IGRlZmluZUNvbmZpZyh7XG4gIGJhc2U6ICcvJyxcbiAgcGx1Z2luczogW1xuICAgIHZ1ZSgpLFxuICAgIEF1dG9JbXBvcnQoe1xuICAgICAgcmVzb2x2ZXJzOiBbRWxlbWVudFBsdXNSZXNvbHZlcigpXVxuICAgIH0pLFxuICAgIENvbXBvbmVudHMoe1xuICAgICAgcmVzb2x2ZXJzOiBbRWxlbWVudFBsdXNSZXNvbHZlcigpXVxuICAgIH0pXG4gIF0sXG4gIHJlc29sdmU6IHtcbiAgICBhbGlhczoge1xuICAgICAgJ0AnOiBmaWxlVVJMVG9QYXRoKG5ldyBVUkwoJy4vc3JjJywgaW1wb3J0Lm1ldGEudXJsKSlcbiAgICB9XG4gIH0sXG4gIGJ1aWxkOiB7XG4gICAgdGFyZ2V0OiAnZXMyMDIwJyxcbiAgICBtaW5pZnk6ICd0ZXJzZXInLFxuICAgIHRlcnNlck9wdGlvbnM6IHtcbiAgICAgIGNvbXByZXNzOiB7XG4gICAgICAgIGRyb3BfY29uc29sZTogdHJ1ZSxcbiAgICAgICAgZHJvcF9kZWJ1Z2dlcjogdHJ1ZSxcbiAgICAgICAgcHVyZV9mdW5jczogWydjb25zb2xlLmxvZycsICdjb25zb2xlLmRlYnVnJywgJ2NvbnNvbGUuaW5mbyddXG4gICAgICB9LFxuICAgICAgZm9ybWF0OiB7XG4gICAgICAgIGNvbW1lbnRzOiBmYWxzZVxuICAgICAgfVxuICAgIH0sXG4gICAgcm9sbHVwT3B0aW9uczoge1xuICAgICAgb3V0cHV0OiB7XG4gICAgICAgIG1hbnVhbENodW5rczoge1xuICAgICAgICAgICdlbGVtZW50LXBsdXMnOiBbJ2VsZW1lbnQtcGx1cyddLFxuICAgICAgICAgICd2dWUtZmxvdyc6IFsnQHZ1ZS1mbG93L2NvcmUnXSxcbiAgICAgICAgICAnbWF4Z3JhcGgnOiBbJ0BtYXhncmFwaC9jb3JlJ10sXG4gICAgICAgICAgJ2hpZ2hsaWdodCc6IFsnaGlnaGxpZ2h0LmpzJywgJ0BoaWdobGlnaHRqcy92dWUtcGx1Z2luJ10sXG4gICAgICAgICAgJ2F4aW9zJzogWydheGlvcyddLFxuICAgICAgICAgICdwaW5pYSc6IFsncGluaWEnLCAncGluaWEtcGx1Z2luLXBlcnNpc3RlZHN0YXRlJ10sXG4gICAgICAgICAgJ3Z1ZS1yb3V0ZXInOiBbJ3Z1ZS1yb3V0ZXInXSxcbiAgICAgICAgICAnbHVjaWRlJzogWydsdWNpZGUtdnVlLW5leHQnXSxcbiAgICAgICAgICAndnVlJzogWyd2dWUnXVxuICAgICAgICB9LFxuICAgICAgICBjb21wYWN0OiB0cnVlXG4gICAgICB9XG4gICAgfSxcbiAgICBjaHVua1NpemVXYXJuaW5nTGltaXQ6IDEwMDAsXG4gICAgcmVwb3J0Q29tcHJlc3NlZFNpemU6IHRydWVcbiAgfSxcbiAgc2VydmVyOiB7XG4gICAgaG9zdDogdHJ1ZSxcbiAgICBwb3J0OiAzMDAwLFxuICAgIGhtcjoge1xuICAgICAgcGF0aDogJy9obXInXG4gICAgfSxcbiAgICBhbGxvd2VkSG9zdHM6IFsnbG9jYWxob3N0JywnODk1bXk4M2xlNTE2LnZpY3AuZnVuJywnMTI3LjAuMC4xJ10sXG4gICAgaHR0cHM6IGZhbHNlLFxuICAgIHByb3h5OiB7XG4gICAgICAnL2FwaSc6IHtcbiAgICAgICAgdGFyZ2V0OiAnaHR0cDovL2xvY2FsaG9zdCcsXG4gICAgICAgIGNoYW5nZU9yaWdpbjogdHJ1ZSxcbiAgICAgICAgcmV3cml0ZTogKHBhdGgpID0+IHBhdGgucmVwbGFjZSgvXlxcL2FwaS8sICcnKVxuICAgICAgfVxuICAgIH1cbiAgfVxufSlcbiJdLAogICJtYXBwaW5ncyI6ICI7QUFBbVosU0FBUyxvQkFBb0I7QUFDaGIsT0FBTyxTQUFTO0FBQ2hCLFNBQVMsZUFBZSxXQUFXO0FBQ25DLE9BQU8sZ0JBQWdCO0FBQ3ZCLE9BQU8sZ0JBQWdCO0FBQ3ZCLFNBQVMsMkJBQTJCO0FBTCtOLElBQU0sMkNBQTJDO0FBT3BULElBQU8sc0JBQVEsYUFBYTtBQUFBLEVBQzFCLE1BQU07QUFBQSxFQUNOLFNBQVM7QUFBQSxJQUNQLElBQUk7QUFBQSxJQUNKLFdBQVc7QUFBQSxNQUNULFdBQVcsQ0FBQyxvQkFBb0IsQ0FBQztBQUFBLElBQ25DLENBQUM7QUFBQSxJQUNELFdBQVc7QUFBQSxNQUNULFdBQVcsQ0FBQyxvQkFBb0IsQ0FBQztBQUFBLElBQ25DLENBQUM7QUFBQSxFQUNIO0FBQUEsRUFDQSxTQUFTO0FBQUEsSUFDUCxPQUFPO0FBQUEsTUFDTCxLQUFLLGNBQWMsSUFBSSxJQUFJLFNBQVMsd0NBQWUsQ0FBQztBQUFBLElBQ3REO0FBQUEsRUFDRjtBQUFBLEVBQ0EsT0FBTztBQUFBLElBQ0wsUUFBUTtBQUFBLElBQ1IsUUFBUTtBQUFBLElBQ1IsZUFBZTtBQUFBLE1BQ2IsVUFBVTtBQUFBLFFBQ1IsY0FBYztBQUFBLFFBQ2QsZUFBZTtBQUFBLFFBQ2YsWUFBWSxDQUFDLGVBQWUsaUJBQWlCLGNBQWM7QUFBQSxNQUM3RDtBQUFBLE1BQ0EsUUFBUTtBQUFBLFFBQ04sVUFBVTtBQUFBLE1BQ1o7QUFBQSxJQUNGO0FBQUEsSUFDQSxlQUFlO0FBQUEsTUFDYixRQUFRO0FBQUEsUUFDTixjQUFjO0FBQUEsVUFDWixnQkFBZ0IsQ0FBQyxjQUFjO0FBQUEsVUFDL0IsWUFBWSxDQUFDLGdCQUFnQjtBQUFBLFVBQzdCLFlBQVksQ0FBQyxnQkFBZ0I7QUFBQSxVQUM3QixhQUFhLENBQUMsZ0JBQWdCLHlCQUF5QjtBQUFBLFVBQ3ZELFNBQVMsQ0FBQyxPQUFPO0FBQUEsVUFDakIsU0FBUyxDQUFDLFNBQVMsNkJBQTZCO0FBQUEsVUFDaEQsY0FBYyxDQUFDLFlBQVk7QUFBQSxVQUMzQixVQUFVLENBQUMsaUJBQWlCO0FBQUEsVUFDNUIsT0FBTyxDQUFDLEtBQUs7QUFBQSxRQUNmO0FBQUEsUUFDQSxTQUFTO0FBQUEsTUFDWDtBQUFBLElBQ0Y7QUFBQSxJQUNBLHVCQUF1QjtBQUFBLElBQ3ZCLHNCQUFzQjtBQUFBLEVBQ3hCO0FBQUEsRUFDQSxRQUFRO0FBQUEsSUFDTixNQUFNO0FBQUEsSUFDTixNQUFNO0FBQUEsSUFDTixLQUFLO0FBQUEsTUFDSCxNQUFNO0FBQUEsSUFDUjtBQUFBLElBQ0EsY0FBYyxDQUFDLGFBQVkseUJBQXdCLFdBQVc7QUFBQSxJQUM5RCxPQUFPO0FBQUEsSUFDUCxPQUFPO0FBQUEsTUFDTCxRQUFRO0FBQUEsUUFDTixRQUFRO0FBQUEsUUFDUixjQUFjO0FBQUEsUUFDZCxTQUFTLENBQUMsU0FBUyxLQUFLLFFBQVEsVUFBVSxFQUFFO0FBQUEsTUFDOUM7QUFBQSxJQUNGO0FBQUEsRUFDRjtBQUNGLENBQUM7IiwKICAibmFtZXMiOiBbXQp9Cg==
