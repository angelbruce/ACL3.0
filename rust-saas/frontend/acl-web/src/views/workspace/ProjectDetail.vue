<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft, Send, Loader2, Bot, User, Wrench, Copy, Check, ChevronDown, Settings, Brain, FileText, Plus, 
  Trash2, X, Save,Play, FolderCheck, FileArchiveIcon, FileCode, 
  Train,
  TrainIcon,
  FileOutput,
  LucideFolderOutput,
  ComponentIcon,
  CheckCircle2} from 'lucide-vue-next'
import { useWorkspaceStore, useLlmStore, useAuthStore, useAgentStore } from '@/stores'
import { llmService, workspaceService, authService, type StreamResponse } from '@/api'
import type { ProjectFile, LlmModel, Agent, ProjectChatMessage } from '@/types'
import { fa } from 'element-plus/es/locales.mjs'
import ProjectContainerConfig from '@/views/workspace/ProjectContainerConfig.vue'

const route = useRoute()
const router = useRouter()
const workspaceStore = useWorkspaceStore()
const llmStore = useLlmStore()
const authStore = useAuthStore()
const agentStore = useAgentStore()

const messagesContainer = ref<HTMLElement | null>(null)
const inputMessage = ref('请执行。')
const sending = ref(false)
const streamingContent = ref('')
const copiedId = ref<number | null>(null)
const showModelDropdown = ref(false)
const showAgentDropdown = ref(false)
const showAgentDetails = ref(false)
const showNewFileModal = ref(false)
const newFileName = ref('')
const selectedModel = ref<LlmModel | null>(null)
const selectedAgent = ref<Agent | null>(null)
const loading = ref(true)
const loadError = ref<string | null>(null)
const selectedFile = ref<ProjectFile | null>(null)
const fileContent = ref('')
const editingFile = ref(true)
const autoSaveEnabled = ref(true)
const showHistory = ref(false)

const projectId = computed(() => Number(route.params.id))
const project = computed(() => workspaceStore.currentProject)
const projectFiles = computed(() => {
  return [...workspaceStore.projectFiles].sort((a, b) => 
    new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
  )
})
const projectMessages = computed(() => workspaceStore.projectMessages)

const scrollToBottom = () => {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
    }
  })
}

const getMessageIcon = (role: string) => {
  switch (role) {
    case 'user': return User
    case 'assistant': return Bot
    case 'tool': return Wrench
    default: return Bot
  }
}

const playing = ref(false)
let audioRef = ref<HTMLAudioElement>()

const play = async (file: ProjectFile) => {
    playing.value = true
    // workspaceStore.getArticleVoice(file.id).then((blob) => {
    //   audioRef.value.src = URL.createObjectURL(blob);
    // })

    workspaceStore.getArticleVoiceLink(file.id).then((link) => {
      console.log(link)
      audioRef.value.src = link
      audioRef.value.play()
      audioRef.value.onended = () => {
        playing.value = false
      }
    })
}

const parseMarkdown = (text: string) => {
  let result = text
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;')
    .replace(/\\\n/g, '&nbsp;')
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/\[DONE\]/g, '')
  
  const lines = result.split('\n')
  const processedLines: string[] = []
  let inList = false
  
  for (const line of lines) {
    const headingMatch = line.match(/^(#{1,6})\s+(.+)$/)
    if (headingMatch) {
      if (inList) {
        processedLines.push('</div>')
        inList = false
      }
      const level = headingMatch[1].length
      processedLines.push(`<h${level} class="heading-${level}">${headingMatch[2].trim()}</h${level}>`)
    } else if (/^---+$/.test(line)) {
      if (inList) {
        processedLines.push('</div>')
        inList = false
      }
      processedLines.push('<hr class="divider" />')
    } else {
      const listMatch = line.match(/^\*\s+(.+)$/)
      if (listMatch) {
        if (!inList) {
          processedLines.push('<div class="list-container">')
          inList = true
        }
        processedLines.push(`<div class="list-item" style="display:block;">➢ ${listMatch[1].trim()}</div>`)
      } else {
        if (inList) {
          processedLines.push('</div>')
          inList = false
        }
        processedLines.push(line.replace(/\*(.+?)\*/g, '<em>$1</em>'))
      }
    }
  }
  
  if (inList) {
    processedLines.push('</div>')
  }

  
  return processedLines.join('\n')
}

interface Tree  {
    id: number,
    name:string,
    data:ProjectFile | null,
    children:Tree[],
    expanded: boolean 
}

const createFileTree = (files: ProjectFile[]) : Tree[]  => {
  let tree: Tree = {
    id: 0,
    name: '/', data: null,    children: [],
    expanded: true,
  }
  
  if (!files || files.length === 0) return [tree]

  let paths: Record<string, Tree> | null = {}
  let id = - 99999999

  for(let file of files) {
    var directory = file.directory || '/'
    if(directory===null || directory.length === 0) directory= "/"
    var segs = directory.split('/')
    let currentPath: Tree | null = tree
    var foot = "/";
    for(let seg of segs) {
      if(seg === null || seg.length === 0) continue
      foot += seg + "/"
      var path =  paths[foot] || null
      if(path === null) {
        path = { id: id++, name: seg, data: null, children: [], expanded: true }
        paths[foot] = path
        currentPath.children.push(path)
      } 

      currentPath = path
    }

    if(currentPath !== null) {
      currentPath.children.push({id: file.id, name : file.name,data:file,children:[], expanded: true})
    }
  }

  console.log('123',tree)
   return [tree]
}

interface FileMatch {
  fileName: string|null
  content: string
}
const extractNovelContent = (text: string): FileMatch[] => {
  const patternRegex = /```[\S]{3,}([ \t]+[\S]+[ \t]*)?\s*([\s\S]*?)```/g
  const matches = [...text.matchAll(patternRegex)]

  let files: FileMatch[] = []
  for(let m of matches) {
    if(m.length > 2) {
      files.push({
        fileName: m[1]?.trim(),
        content: m[2]
      })
    } else if (m.length === 2) {
      files.push({
        fileName: null,
        content: m[1]
      })
    }
  }
  
  return files
}

const getMessageClass = (role: string) => {
  switch (role) {
    case 'user': return 'bg-primary-50 border-primary-100'
    case 'assistant': return 'bg-surface-50 border-surface-200'
    case 'tool': return 'bg-green-50 border-green-100'
    default: return 'bg-surface-50 border-surface-200'
  }
}

const getAvatarClass = (role: string) => {
  switch (role) {
    case 'user': return 'bg-primary-100 text-primary-600'
    case 'assistant': return 'bg-surface-100 text-surface-600'
    case 'tool': return 'bg-green-100 text-green-600'
    default: return 'bg-surface-100 text-surface-600'
  }
}

const copyMessage = async (message: ProjectChatMessage) => {
  await navigator.clipboard.writeText(message.content)
  copiedId.value = message.id
  setTimeout(() => { copiedId.value = null }, 2000)
}

const selectModel = async (model: LlmModel) => { 
  selectedModel.value = model
  showModelDropdown.value = false 
  await updateProjectSettings()
}

const selectAgent = async (agent: Agent) => { 
  selectedAgent.value = agent
  showAgentDropdown.value = false
  await agentStore.fetchAgent(agent.id) 
  await updateProjectSettings()
}

const updateProjectSettings = async () => {
  if (!project.value) return
  try {
    await workspaceStore.updateProjectSettings(projectId.value, {
      model_id: selectedModel.value?.id || null,
      agent_id: selectedAgent.value?.id || null,
      name: project.value?.name || null,
      description: project.value?.description || null,
    })
  } catch {
    // 忽略错误
  }
}

const sendMessage = async () => {
  if (!selectedAgent.value) throw new Error('请先选择智能体')
  if (!inputMessage.value.trim() || sending.value) return
  if (!authStore.user) { router.push('/login'); return }

  const userMessage = inputMessage.value

  try {
    await workspaceStore.addProjectMessage(projectId.value, userMessage, 'user')
  } catch { return }

  sending.value = true
  streamingContent.value = ''

  try {
    if (!selectedModel.value) {
      throw new Error('请先选择模型') 
    }

    const chatMessages = [];

   chatMessages.push({
        role: 'system',
        content: `[任务指令]：必须完成编写创作：编写${project.value?.name || ''}:${project.value?.description || ''}。`
    });

    for(let msg of projectMessages.value) {
      if(!msg.content || !msg.role) continue;
      if(msg.content && msg.content === '请你执行。') continue;
      if(msg.content && msg.content.startsWith('错误: ')) continue;
      chatMessages.push({
        role: msg.role,
        content: msg.content,
      })
    }

    chatMessages.push({
      role: 'user',
      content: userMessage,
    })

    const agentId = selectedAgent.value?.id

    var cacheData = '';
    var refreshCount = 100;
    await llmService.chatStream(
      { 
        model_id: selectedModel.value.id, 
        messages: chatMessages, 
        agent_id: agentId, 
        stream: true,
        project_id: projectId.value
      },
      (data: StreamResponse) => { 
        cacheData += data.content
        refreshCount--
        if(refreshCount <= 0) {
          refreshCount = 100
          streamingContent.value += cacheData
          console.log(cacheData)
          cacheData = '';
          nextTick(() => scrollToBottom())
        }
      },
      (error: Error) => { console.error('[ERROR] Stream error:', error) }
    )
    if(refreshCount > 0) {
      streamingContent.value += cacheData
    }

    await workspaceStore.addProjectMessage(projectId.value, streamingContent.value, 'assistant')
    
    if (autoSaveEnabled.value) {
      const files = extractNovelContent(streamingContent.value)
      if (files.length > 0) {
        for(let file of files) {
          var filename = file.fileName || new Date().toLocaleString()
          const newFile = await workspaceStore.createProjectFile(projectId.value, filename)
          await saveFileAndContent(newFile.id, file.content)
        }
      }
    }
  } catch (error) {
    await workspaceStore.addProjectMessage(projectId.value, `错误: ${error instanceof Error ? error.message : '未知错误'}`, 'system')
  } finally {
    sending.value = false
    streamingContent.value = ''
    scrollToBottom()
  }

  if(projectFiles.value.length > 0){
    selectedFile.value = projectFiles.value[0]
  } 

}

const saveFileAndContent = async (id:any,content: string) => {
  try {
    await workspaceStore.updateProjectFile(id, content)
  } catch (err) {
    loadError.value = err instanceof Error ? err.message : '保存文件失败'
  }
}

const formatTime = (dateStr: string) => new Date(dateStr).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })

const selectFile = async (file: ProjectFile) => {
  selectedFile.value = file
  fileContent.value = file.content || ''
  editingFile.value = true
}

const openNewFileModal = () => {
  showNewFileModal.value = true
  newFileName.value = ''
}

const closeNewFileModal = () => {
  showNewFileModal.value = false
  newFileName.value = ''
}

const createNewFile = async () => {
  if (!newFileName.value.trim()) return
  
  try {
    const newFile = await workspaceStore.createProjectFile(projectId.value, newFileName.value.trim())
    selectedFile.value = newFile
    fileContent.value = ''
    editingFile.value = true
    closeNewFileModal()
  } catch (err) {
    loadError.value = err instanceof Error ? err.message : '创建文件失败'
  }
}

const deleteFile = async (file: ProjectFile) => {
  if (!confirm(`确定要删除文件 "${file.name}" 吗？`)) return
  
  try {
    await workspaceStore.deleteProjectFile(file.id)
    if (selectedFile.value?.id === file.id) {
      selectedFile.value = null
      fileContent.value = ''
    }
  } catch (err) {
    loadError.value = err instanceof Error ? err.message : '删除文件失败'
  }
}

const saveFile = async () => {
  if (!selectedFile.value) return
  
  try {
    await workspaceStore.updateProjectFile(selectedFile.value.id, fileContent.value)
    
    const summaryContent = await generateSummary(fileContent.value)
    if (summaryContent) {
      await workspaceService.createOrUpdateProjectSummary(projectId.value, {
        file_name: selectedFile.value.name,
        summary: summaryContent
      })
    }
  } catch (err) {
    loadError.value = err instanceof Error ? err.message : '保存文件失败'
  }
}

const generateSummary = async (content: string): Promise<string | null> => {
  if (!content || content.trim().length < 50) {
    return null
  }
  
  try {
    if (!selectedModel.value) {
      return null
    }
    
    return new Promise((resolve) => {
      const summaryPrompt = `请为以下文章生成一个简洁的摘要（不超过300字），只返回摘要内容，不要其他说明：\n\n${content.substring(0, 5000)}`
      
      let summaryContent = ''
      
      llmService.chatStream(
        {
          model_id: selectedModel?.value?.id || 0,
          messages: [{ role: 'user', content: summaryPrompt }],
          agent_id: selectedAgent.value?.id,
          stream: true,
          project_id: projectId.value
        },
        (data: StreamResponse) => {
          summaryContent += data.content
        },
        (error: Error) => {
          console.error('[ERROR] Summary generation error:', error)
          resolve(null)
        }
      ).then(() => {
        if (summaryContent.trim().length > 0) {
          const truncatedSummary = summaryContent.trim().substring(0, 300)
          resolve(truncatedSummary)
        } else {
          resolve(null)
        }
      }).catch(() => {
        resolve(null)
      })
    })
  } catch (error) {
    console.error('[ERROR] Generate summary error:', error)
    return null
  }
}

watch(fileContent, () => {
  if (selectedFile.value) {
    const currentFile = projectFiles.value.find(f => f.id === selectedFile.value?.id)
    if (currentFile) {
      currentFile.content = fileContent.value
    }
  }
})

const fileTree = computed(() => createFileTree(projectFiles.value))

onMounted(async () => {
  loading.value = true
  loadError.value = null

  if (!authStore.user && authStore.isAuthenticated) {
    try {
      const tokenData = JSON.parse(atob(authStore.accessToken!.split('.')[1]))
      const userInfo = await authService.getUser(tokenData.user_id)
      authStore.user = userInfo
    } catch {
      loadError.value = '加载用户信息失败'
      router.push('/login')
      return
    }
  }

  if (!authStore.user) {
    loadError.value = '请先登录'
    router.push('/login')
    return
  }

  try {
    await Promise.all([
      workspaceStore.fetchProject(projectId.value),
      workspaceStore.fetchProjectFiles(projectId.value),
      workspaceStore.fetchProjectMessages(projectId.value),
      llmStore.fetchModels(),
      agentStore.fetchAgents(),
    ])
    
    const currentProject = workspaceStore.currentProject
    if (currentProject) {
      if (currentProject.agent_id) {
        selectedAgent.value = agentStore.agents.find(a => a.id === currentProject.agent_id) || null
      }
      if (currentProject.model_id) {
        selectedModel.value = llmStore.models.find(m => m.id === currentProject.model_id) || llmStore.defaultModel || llmStore.models[0] || null
      } else {
        selectedModel.value = llmStore.defaultModel || llmStore.models[0] || null
      }
      
      if (projectFiles.value.length > 0) {
        selectedFile.value = projectFiles.value[projectFiles.value.length - 1]
        fileContent.value = selectedFile.value.content || ''
      }
    } else {
      selectedModel.value = llmStore.defaultModel || llmStore.models[0] || null
    }
    
    scrollToBottom()
  } catch (error) {
    loadError.value = error instanceof Error ? error.message : '加载项目数据失败'
  } finally {
    loading.value = false
  }
})


const debugCode = ()=> {
  if (!selectedFile.value) {
    loadError.value = '请先选择文件'
    return
  }

  debugVisible.value = true
}

let debugVisible = ref(false)
</script>

<template>
  <div class="flex h-full overflow-hidden">
    <div class="w-64 bg-white border-r border-surface-200 flex flex-col flex-shrink-0">
      <div class="p-4 border-b border-surface-200">
        <button @click="router.push('/workspace')" class="flex items-center gap-2 text-surface-400 hover:text-surface-700 transition-colors mb-4">
          <ArrowLeft class="w-5 h-5" />
          <span>返回项目列表</span>
        </button>
        <h2 class="font-semibold text-surface-800">{{ project?.name }}</h2>
        <p class="text-sm text-surface-400 mt-1">{{ project?.description || '暂无描述' }}</p>
      </div>

      <div class="flex-1 overflow-y-auto">
        <div class="p-3">
          <div class="flex items-center justify-between mb-2">
            <span class="text-sm font-medium text-surface-600">文件列表</span>
            <!-- <button @click="openNewFileModal" class="p-1 text-surface-400 hover:text-surface-600 hover:bg-surface-100 rounded transition-colors">
              <Plus class="w-4 h-4" />
            </button> -->
          </div>
          <div class="space-y-1">
            <div 
              v-if="project && project?.purpose === 'article'"
              v-for="file in projectFiles" 
              :key="file.id"
              @click="selectFile(file)"
              :class="['flex items-center py-1 rounded-md cursor-pointer transition-colors', selectedFile?.id === file.id ? 'bg-primary-50 text-primary-600' : 'text-surface-600 hover:bg-surface-50']"
            >
              <div class="flex items-center ">
                <button v-if="project?.purpose === 'article'"
                  class="flex items-center 
                  gap-1 
                  px-1 py-1 mx-1
                  bg-green-500 text-white rounded-md
                  hover:bg-green-600 transition-colors 
                  text-sm flex-shrink-0"
                  @click="play(file)">
                  <Play class="w-2 h-2" />
                </button>
              </div>
              <div class="flex items-center gap-2">
                <FileText class="w-4 h-4" v-if="project?.purpose !== 'article'" />
                <span class="text-sm truncate">{{ file.name }}</span>
              </div>
              <button 
                @click.stop="deleteFile(file)"
                class="p-1 opacity-0 hover:opacity-100 text-surface-400 hover:text-red-500 transition-all"
              >
                <Trash2 class="w-3 h-3" />
              </button>
            </div>

            <el-tree :data="fileTree" v-if="project && project?.purpose === 'coding'" 
                check-strictly="true" 
                highlight-current="true"
                empty-text="暂无文件"
                :node-key="'id'"
                :default-expanded-keys="[0]"
                overflow="auto"
                >
                 <template #default="{ data }" class="w-full overflow-x-auto">
                    <div class="flex items-center gap-2"
                      @click="selectFile(data.data)"     
                    >
                      <FileCode class="w-4 h-4" v-if="data.data" />
                      <FolderCheck class="w-4 h-4" v-else />
                     <span class="text-sm truncate flex-1">{{ data.name }}</span>
                      <button 
                        v-if="data.data"
                        @click.stop="deleteFile(data.data)" class="p-1 opacity-0 hover:opacity-100 text-surface-400 hover:text-red-500 transition-all float-right">
                        <Trash2 class="w-3 h-3" />
                      </button>
                    </div>
                  </template>
            </el-tree>
       
            <div v-if="projectFiles.length === 0" class="text-center py-8 text-surface-400 text-sm">
              <FileText class="w-8 h-8 mx-auto mb-2 opacity-50" />
              <p>暂无文件</p>
            </div>
          </div>
        </div>
      </div>

      <div class="p-3 border-t border-surface-200 flex-shrink-0">
        <!-- <div class="flex items-center justify-between mb-2">
          <span class="text-sm font-medium text-surface-600">设置</span>
        </div> -->
        
        <div class="space-y-2">
          <div class="relative">
            <button @click="showModelDropdown = !showModelDropdown; showAgentDropdown = false" class="w-full flex items-center gap-2 px-3 py-2 bg-surface-50 border border-surface-200 rounded-lg hover:bg-surface-100 transition-colors text-sm text-surface-600">
              <Settings class="w-4 h-4 text-surface-400" />
              <span class="truncate">{{ selectedModel?.name || '选择模型' }}</span>
              <ChevronDown class="w-4 h-4 text-surface-400 ml-auto" />
            </button>
            <div v-if="showModelDropdown" class="absolute left-0 right-0 bottom-full mb-2 bg-white border border-surface-200 rounded-lg shadow-lg max-h-48 overflow-y-auto">
              <div v-for="model in llmStore.models" :key="model.id">
                <button @click="selectModel(model)" :class="['w-full px-4 py-2 text-left text-sm hover:bg-surface-50 transition-colors', selectedModel?.id === model.id ? 'bg-primary-50 text-primary-600' : 'text-surface-700']">
                  {{ model.name }}
                </button>
              </div>
            </div>
          </div>

          <div class="relative">
            <button @click="showAgentDropdown = !showAgentDropdown; showModelDropdown = false" class="w-full flex items-center gap-2 px-3 py-2 bg-surface-50 border border-surface-200 rounded-lg hover:bg-surface-100 transition-colors text-sm text-surface-600">
              <Brain class="w-4 h-4 text-surface-400" />
              <span class="truncate">{{ selectedAgent?.name || '选择 Agent' }}</span>
              <ChevronDown class="w-4 h-4 text-surface-400 ml-auto" />
            </button>
            <div v-if="showAgentDropdown" class="absolute left-0 right-0 bottom-full mb-2 bg-white border border-surface-200 rounded-lg shadow-lg max-h-48 overflow-y-auto">
              <div v-for="agent in agentStore.agents" :key="agent.id">
                <button @click="selectAgent(agent)" :class="['w-full px-4 py-2 text-left text-sm hover:bg-surface-50 transition-colors', selectedAgent?.id === agent.id ? 'bg-primary-50 text-primary-600' : 'text-surface-700']">{{ agent.name }}</button>
              </div>
            </div>
          </div>

          <div class="p-2 bg-white border-t border-surface-200 flex-shrink-0">
            <form @submit.prevent="sendMessage" class="flex flex-col gap-2">
              <textarea
                v-model="inputMessage"
                class="w-full px-3 py-2 bg-surface-100 border border-surface-200 rounded-lg  text-sm text-surface-600"
              />
              
              <button 
                type="submit" 
                :disabled="sending || !selectedModel"
                class="w-full px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2 text-sm"
              >
                <Loader2 v-if="sending" class="w-4 h-4 animate-spin" />
                <Send v-else class="w-4 h-4" />
                <span>{{ sending ? '运行中...' : '开始' }}</span>
              </button>
            </form>
          </div>

          <label class="flex items-center gap-2 px-3 py-2 text-sm text-surface-600" style="display: none;">
            <input type="checkbox" disabled v-model="autoSaveEnabled" class="rounded border-surface-300" />
            <span>自动保存</span>
          </label>
        </div>
      </div>
    </div>

    <div class="flex-1 flex flex-col bg-surface-50 overflow-hidden">
      <div class="h-14 flex items-center gap-4 px-4 bg-white border-b border-surface-200 flex-shrink-0">
        <div v-if="selectedAgent" class="flex items-center gap-2">
          <button @click="showAgentDetails = !showAgentDetails" class="p-2 rounded-lg hover:bg-surface-50 transition-colors" :class="showAgentDetails ? 'text-primary-600 bg-primary-50' : 'text-surface-400'">
            <Bot class="w-5 h-5" />
          </button>
        </div>
        <div class="flex-1">
          <h1 class="font-semibold text-surface-800">{{ selectedFile?.name || '未选择文件' }}</h1>
          <p class="text-xs text-surface-400 mt-0.5">{{ projectMessages.length }} 条对话</p>
        </div>
      
        <div class="flex items-center">
          <div class="flex items-center gap-2  flex-row">
            <button class="bg-surface-50 border px-4 py-2 border-surface-200 rounded-lg hover:bg-surface-100 transition-colors text-sm text-surface-600 flex items-center gap-2">
             <LucideFolderOutput class="w-4 h-4" /> <span>导出</span>
            </button>

            <button class="bg-surface-50 border px-4 py-2 border-surface-200 rounded-lg hover:bg-surface-100 transition-colors text-sm text-surface-600 flex items-center gap-2"
             v-if="project && project?.purpose === 'coding'"
             @click="debugCode"
             >
             <ComponentIcon class="w-4 h-4" /> <span>调试</span>
            </button>
        
            <button class="bg-surface-50 border px-4 py-2 border-surface-200 rounded-lg hover:bg-surface-100 transition-colors text-sm text-surface-600 flex items-center gap-2"
             v-if="project && project?.purpose === 'coding'" 
             @click="showHistory = !showHistory"
             >
             <CheckCircle2 class="w-4 h-4" /> <span>交互记录</span>
            </button>
          </div>
        </div>

        <button 
          v-if="selectedFile"
          style="display: none;"
          @click="saveFile"
          class="flex items-center gap-2 px-4 py-2 bg-green-500 text-white rounded-lg hover:bg-green-600 transition-colors text-sm flex-shrink-0"
        >
          <Save class="w-4 h-4" />
          保存
        </button>
      </div>

      <div v-if="showAgentDetails && agentStore.currentAgent" class="border-b border-surface-200 bg-surface-50 p-3 flex-shrink-0 overflow-y-auto" style="max-height: 180px;">
        <div class="space-y-2">
          <div>
            <h3 class="font-semibold text-primary-600 flex items-center gap-2"><Brain class="w-4 h-4" /> {{ agentStore.currentAgent.name }}</h3>
            <p v-if="agentStore.currentAgent.defination" class="text-xs text-surface-500 mt-1">{{ agentStore.currentAgent.defination }}</p>
          </div>
          <div v-if="agentStore.currentAgent.skills?.length" class="flex flex-wrap gap-1">
            <div v-for="skill in agentStore.currentAgent.skills" :key="skill.id" class="px-2 py-1 bg-white rounded text-xs text-surface-500 border border-surface-200">{{ skill.skill_prompt.substring(0, 50) }}...</div>
          </div>
          <div v-if="agentStore.currentAgent.tools?.length" class="flex flex-wrap gap-1">
            <div v-for="tool in agentStore.currentAgent.tools" :key="tool.id" class="px-2 py-1 bg-green-50 border border-green-100 rounded text-xs">
              <span class="font-medium text-green-600">{{ tool.name }}</span>
            </div>
          </div>
        </div>
      </div>

      <div class="flex-1 flex overflow-hidden">
        <div class="flex-1 flex flex-col bg-white border-r border-surface-200 overflow-hidden">
          <div class="flex-1 overflow-hidden p-3">
              <highlightjs 
              class="h-full resize-none border-none outline-none 
              bg-transparent text-surface-700
              word-break-break-word
              overflow-wrap-break-word
              whitespace-pre-wrap
              overflow-auto
              leading-relaxed text-base"
              :code="fileContent" 
              autodetect />
          </div>
        </div>

        <div class="w-80 flex flex-col bg-surface-50 flex-shrink-0" v-if="showHistory">
          <div ref="messagesContainer" class="flex-1 overflow-y-auto p-2 space-y-2">
            <div v-if="loading" class="flex items-center justify-center h-full">
              <Loader2 class="w-6 h-6 animate-spin text-primary-500" />
            </div>

            <div v-else-if="loadError" class="flex flex-col items-center justify-center h-full text-center">
              <p class="text-red-500 mb-4 text-sm">{{ loadError }}</p>
              <button @click="router.push('/workspace')" class="btn btn-primary">返回项目列表</button>
            </div>

            <template v-else>
              <div v-for="message in projectMessages" :key="message.id" :class="['flex gap-2', message.role === 'user' ? 'flex-row-reverse' : '']">
                <div :class="['w-5 h-5 rounded-lg flex items-center justify-center flex-shrink-0', getAvatarClass(message.role)]">
                  <component :is="getMessageIcon(message.role)" class="w-2.5 h-2.5" />
                </div>
                <div :class="['flex-1', message.role === 'user' ? 'max-w-xs' : 'max-w-full']">
                  <div :class="['rounded-lg p-2 border text-xs', getMessageClass(message.role)]">
                    <p :class="['whitespace-pre-wrap leading-relaxed text-surface-700 text-xs', message.role === 'user' ? 'text-right' : 'text-left']" v-html="parseMarkdown(message.content || '')"></p>
                  </div>
                  <div :class="['flex items-center gap-1 mt-0.5 text-xs text-surface-400', message.role === 'user' ? 'justify-end' : '']">
                    <span>{{ formatTime(message.created_at) }}</span>
                    <button v-if="message.role !== 'user'" @click="copyMessage(message)" class="p-0.5 hover:text-surface-700 transition-colors">
                      <Check v-if="copiedId === message.id" class="w-3 h-3 text-green-500" />
                      <Copy v-else class="w-3 h-3" />
                    </button>
                  </div>
                </div>
              </div>
            </template>
             <highlightjs 
              class="w-full h-30 resize-none border-none outline-none bg-transparent text-surface-700 leading-relaxed text-base
              word-break-break-word
              overflow-wrap-break-word
              whitespace-pre-wrap
              overflow-auto
              leading-relaxed text-base
              " 
              style="font-family: 'Georgia', 'Times New Roman', serif;"
              :code="streamingContent" 
              autodetect />
          </div>
        </div>
      </div>
    </div>

  </div>
  <el-dialog v-model="debugVisible" :close-on-click-modal="false" class="h-full overflow-auto border border-surface-200 rounded-lg p-4" style="margin-top:-1px" width="50%">
    <ProjectContainerConfig 
      :project="project"
      :messages="projectMessages"
    />
  </el-dialog>
</template>

<style scoped>
.font-serif {
  font-family: 'Georgia', 'Times New Roman', serif;
}

:deep(.heading-1) {
  font-size: 1.5rem;
  font-weight: bold;
  margin: 1rem 0 0.5rem;
  color: #1e293b;
}

:deep(.heading-2) {
  font-size: 1.25rem;
  font-weight: bold;
  margin: 0.75rem 0 0.5rem;
  color: #334155;
}

:deep(.heading-3) {
  font-size: 1.125rem;
  font-weight: 600;
  margin: 0.5rem 0 0.25rem;
  color: #475569;
}

:deep(.heading-4) {
  font-size: 1rem;
  font-weight: 600;
  margin: 0.5rem 0 0.25rem;
  color: #64748b;
}

:deep(.heading-5) {
  font-size: 0.875rem;
  font-weight: 600;
  margin: 0.25rem 0;
  color: #64748b;
}

:deep(.heading-6) {
  font-size: 0.875rem;
  font-weight: 500;
  margin: 0.25rem 0;
  color: #94a3b8;
}

:deep(.divider) {
  border: none;
  height: 1px;
  background: linear-gradient(to right, transparent, #cbd5e1, transparent);
  margin: 1rem 0;
}

:deep(.done-badge) {
  display: inline-block;
  padding: 0.25rem 0.75rem;
  background: linear-gradient(135deg, #10b981, #059669);
  color: white;
  font-size: 0.75rem;
  font-weight: 600;
  border-radius: 9999px;
  margin: 0 0.25rem;
  box-shadow: 0 2px 8px rgba(16, 185, 129, 0.3);
}
</style>
