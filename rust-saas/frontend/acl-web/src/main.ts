import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
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

// 3. 按需注册语言 (只注册你项目中用到的语言，减小体积)
import javascript from 'highlight.js/lib/languages/javascript'
import typescript from 'highlight.js/lib/languages/typescript'
import xml from 'highlight.js/lib/languages/xml' // html 也使用 xml 解析器
import json from 'highlight.js/lib/languages/json'
import python from 'highlight.js/lib/languages/python'
import java from 'highlight.js/lib/languages/java'
import css from 'highlight.js/lib/languages/css'
import sql from 'highlight.js/lib/languages/sql'    
import cpp from 'highlight.js/lib/languages/cpp'    
import yaml from 'highlight.js/lib/languages/yaml' 
import kotlin from 'highlight.js/lib/languages/kotlin' 
import rust from 'highlight.js/lib/languages/rust' 
import csharp from 'highlight.js/lib/languages/csharp' 


const app = createApp(App)

const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)

hljs.registerLanguage('javascript', javascript)
hljs.registerLanguage('typescript', typescript)
hljs.registerLanguage('xml', xml)
hljs.registerLanguage('json', json)
hljs.registerLanguage('python', python)
hljs.registerLanguage('java', java)
hljs.registerLanguage('java', java)
hljs.registerLanguage('css', css)
hljs.registerLanguage('sql', sql)
hljs.registerLanguage('cpp', cpp)
hljs.registerLanguage('yaml', yaml)
hljs.registerLanguage('kotlin', kotlin)
hljs.registerLanguage('rust', rust)
hljs.registerLanguage('csharp', csharp)

app.use(hljsVuePlugin) // 注册插件
app.use(pinia)
app.use(router)
app.use(ElementPlus)

app.mount('#app')
