/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

interface ImportMetaEnv {
  readonly VITE_API_AUTH_URL: string
  readonly VITE_API_AGENT_URL: string
  readonly VITE_API_FLOW_URL: string
  readonly VITE_API_SESSION_URL: string
  readonly VITE_API_LLM_URL: string
  readonly VITE_API_MCP_URL: string
  readonly VITE_API_ADMIN_URL: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
