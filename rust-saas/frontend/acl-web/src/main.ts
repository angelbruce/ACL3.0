import { createApp } from 'vue'
import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import App from './App.vue'
import router from './router'
import './style.css'
import '@maxgraph/core/css/common.css'



// 1. 引入主题样式 (例如 stackoverflow-light, github-dark 等)
import 'highlight.js/styles/stackoverflow-light.css'

// 2. 引入核心库和 Vue 插件
import hljs from 'highlight.js/lib/core'
import hljsVuePlugin from '@highlightjs/vue-plugin'

// 3. 按需注册语言 (只注册项目中常用的语言，减小体积)
import javascript from 'highlight.js/lib/languages/javascript'
import typescript from 'highlight.js/lib/languages/typescript'
import xml from 'highlight.js/lib/languages/xml' // html 也使用 xml 解析器
import json from 'highlight.js/lib/languages/json'
import python from 'highlight.js/lib/languages/python'
import yaml from 'highlight.js/lib/languages/yaml'


const app = createApp(App)

const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)

hljs.registerLanguage('javascript', javascript)
hljs.registerLanguage('typescript', typescript)
hljs.registerLanguage('xml', xml)
hljs.registerLanguage('json', json)
hljs.registerLanguage('python', python)
hljs.registerLanguage('yaml', yaml)

app.use(hljsVuePlugin)
app.use(pinia)
app.use(router)

app.mount('#app')
