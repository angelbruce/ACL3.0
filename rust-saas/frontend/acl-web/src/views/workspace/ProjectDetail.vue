<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft, Send, Loader2, Bot, User, Wrench, Copy, Check, ChevronDown, Settings, Brain, FileText, Plus, 
  Trash2,  Save,Play, FolderCheck, FileCode, 
  LucideFolderOutput,
  ComponentIcon,
  CheckCircle2} from 'lucide-vue-next'
import { useWorkspaceStore, useLlmStore, useAuthStore, useAgentStore } from '@/stores'
import { llmService, workspaceService, authService, type StreamResponse } from '@/api'
import type { ProjectFile, LlmModel, Agent, ProjectChatMessage } from '@/types'
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

const deleteMessage = (message: ProjectChatMessage) => {
  workspaceStore.deleteProjectMessage(projectId.value, message.id).then(() => {
    workspaceStore.fetchProjectMessages(projectId.value)
    scrollToBottom()
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

const planing = ref(false);
const getBtnText = computed(()=>{
  if(planing.value) {
    return '规划中'
  }
   sending.value ? '发送中' : '发送'
})

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
    const configId = currentContainerConfigId.value || 0

    var cacheData = '';
    var refreshCount = 100;
    
    // 使用 workspace 专用的 chat/stream 接口
    await workspaceService.workspaceChatStream(
      { 
        model_id: selectedModel.value.id, 
        agent_id: agentId,
        project_id: projectId.value,
        config_id: configId,
        messages: chatMessages,
      },
      (data: { content: string; tool_calls?: unknown; finish_reason?: string }) => { 
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
          var filename = file.fileName || null//new Date().toLocaleString()
          if(!filename) continue;
          
          console.log(filename,workspaceStore.projectFiles)
          // 检查文件名是否已存在
          const existingFile = workspaceStore.projectFiles.find(f => ((f.directory && f.directory.length >0) ? (f.directory + '/' + f.name)  : f.name) === filename)
          if (existingFile) {
            // 文件已存在，更新内容
            await saveFileAndContent(existingFile.id, file.content)
          } else {
            // 文件不存在，创建新文件
            const newFile = await workspaceStore.createProjectFile(projectId.value, filename)
            await saveFileAndContent(newFile.id, file.content)
          }
        }
      
        await planActions();
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

const planActions = async() => {
  next_step().then(data => {
      console.log('data',data)
      let result = data?.trim() || '';
      let idx = result.indexOf(' ');
      let type = '';
      let prompt = null;
      if(idx !== -1) {
        var array = result.split(' ');
        type = array[0];
        if(array.length>1)  {
          prompt = array[1];
          if(prompt.trim().length == 0) {
            prompt = null;
          }
        }
      } else {
        type = result;
      }

      switch(type) {
        case "1": {
          if(prompt == null) prompt = '请执行'
          inputMessage.value = prompt
          sendMessage()
          break;
        }
        case "2": {
            processCommand(prompt)
          break;
        }
        case "3": {
          break;
        }
        case "3": return;
        default: return;
      }
      
    })
}

const saveFileAndContent = async (id:any,content: string) => {
  try {
    await workspaceStore.updateProjectFile(id, content)
   
    await refreshFileContent(id,content);
    
  } catch (err) {
    loadError.value = err instanceof Error ? err.message : '保存文件失败'
  }
}

const refreshFileContent = async(id:any,content: string) => {
  const configId = projectContainerConfigRef.value?.activeTab
  if (configId) {
    const statusResponse = await workspaceService.getContainerStatus(projectId.value, configId)
    const isRunning = statusResponse.target_status 
      ? statusResponse.target_status.state?.toLowerCase() === 'running' || statusResponse.target_status.state?.toLowerCase() === 'up'
      : statusResponse.statuses.some((s: any) => s.state?.toLowerCase() === 'running' || s.state?.toLowerCase() === 'up')
    if (isRunning) {
      const configs = await workspaceStore.getProjectContainerConfigs(projectId.value)
      const config = configs?.find((c: any) => c.id === configId)
      const command = (config?.command || '')
      await workspaceService.refreshFileToContainer(projectId.value, {
        file_id: id,
        config_id: configId,
        content: content,
        command: ''
      })
      console.log('[saveFileAndContent] File synced to container:', id)
      console.log('[saveFileAndContent] Executing command in terminal:', command)
    }
  }
}
const processCommand = async(cmd:string|null) => {
  const configId = projectContainerConfigRef.value?.activeTab
  if (configId) {
    const statusResponse = await workspaceService.getContainerStatus(projectId.value, configId)
    const isRunning = statusResponse.target_status 
      ? statusResponse.target_status.state?.toLowerCase() === 'running' || statusResponse.target_status.state?.toLowerCase() === 'up'
      : statusResponse.statuses.some((s: any) => s.state?.toLowerCase() === 'running' || s.state?.toLowerCase() === 'up')
    if (isRunning) {
      // 获取配置中的 command
      const configs = await workspaceStore.getProjectContainerConfigs(projectId.value)
      const config = configs?.find((c: any) => c.id === configId)
      const command = cmd || (config?.command || '')
      
      if (command) {
        projectContainerConfigRef.value?.executeCommand(configId, command)
      }
    }
  }
}

const formatTime = (dateStr: string) => new Date(dateStr).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })

const selectFile = async (file: ProjectFile) => {
  if(!file) return;
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
    
    // 保存成功后，检查容器是否存活，如果存活则刷新文件并执行 command
    await refreshContainerAndExecute()
  } catch (err) {
    loadError.value = err instanceof Error ? err.message : '保存文件失败'
  }
}

// 刷新容器文件并执行 command
const refreshContainerAndExecute = async () => {
  console.log('[refreshContainerAndExecute] Starting...')
  
  // 获取当前选中的 config_id（从 ProjectContainerConfig 组件）
  const configId = projectContainerConfigRef.value?.activeTab
  console.log('[refreshContainerAndExecute] Current configId:', configId)
  
  if (!configId) {
    console.log('[refreshContainerAndExecute] No config selected')
    return
  }
  
  try {
    // 1. 获取项目的所有容器配置
    const configs = await workspaceStore.getProjectContainerConfigs(projectId.value)
    console.log('[refreshContainerAndExecute] Configs:', configs)
    
    if (!configs || configs.length === 0) {
      console.log('[refreshContainerAndExecute] No container configs found')
      return
    }
    
    // 2. 找到当前选中的配置
    const config = configs.find((c: any) => c.id === configId)
    if (!config) {
      console.log('[refreshContainerAndExecute] Config not found for id:', configId)
      return
    }
    
    const command = config.command || ''
    if (!command) {
      console.log('[refreshContainerAndExecute] No command configured')
      return
    }
    
    // 3. 检查容器是否运行
    const statusResponse = await workspaceService.getContainerStatus(projectId.value, configId)
    console.log('[refreshContainerAndExecute] Container status:', statusResponse)
    
    const isRunning = statusResponse.target_status 
      ? statusResponse.target_status.state?.toLowerCase() === 'running' || statusResponse.target_status.state?.toLowerCase() === 'up'
      : statusResponse.statuses.some((s: any) => s.state?.toLowerCase() === 'running' || s.state?.toLowerCase() === 'up')
    
    if (!isRunning) {
      console.log('[refreshContainerAndExecute] Container is not running')
      return
    }
    
    // 4. 刷新文件到容器
    console.log('[refreshContainerAndExecute] Refreshing file to container...')
    if (selectedFile.value) {
      await workspaceService.refreshFileToContainer(projectId.value, {
        file_id: selectedFile.value.id,
        config_id: configId,
        content: fileContent.value || '',
        command: ''
      })
      console.log('[refreshContainerAndExecute] File refreshed to container')
    }
    
    // 5. 在前端终端中执行 command（关键修改：在终端中显示执行结果）
    console.log('[refreshContainerAndExecute] Executing command in terminal:', command)
    projectContainerConfigRef.value?.executeCommand(configId, command)
  } catch (err) {
    console.error('[refreshContainerAndExecute] Error:', err)
  }
}

// 当前容器的 config_id（需要在 ProjectContainerConfig 中设置）
const currentContainerConfigId = ref<number | null>(null)
const projectContainerConfigRef = ref<InstanceType<typeof import('@/views/workspace/ProjectContainerConfig.vue').default> | null>(null)

// 设置当前容器 config_id（供 ProjectContainerConfig 调用）
const setCurrentContainerConfigId = (configId: number) => {
  currentContainerConfigId.value = configId
}

// 处理容器配置就绪
const handleConfigReady = (configId: number) => {
  currentContainerConfigId.value = configId
}

// 暴露给子组件
defineExpose({
  setCurrentContainerConfigId
})

const next_step = async (): Promise<string | null> => {
    if (!selectedModel.value) {
      return null
    }

    planing.value = true;
    sending.value = true;
    inputMessage.value = '规划中..'

    const chatMessages: any[] = [];

    
    for(let msg of projectMessages.value) {
      if(!msg.content || !msg.role) continue;
      if(msg.role && msg.role === 'system' ) continue;
      if(msg.content && msg.content === '请你执行。') continue;
      if(msg.content && msg.content.startsWith('错误: ')) continue;
      chatMessages.push({
        role: msg.role,
        content: msg.content,
      })
    }

   chatMessages.push({
        role: 'system',
        content: `[角色]：你是一个任务规划师，根据任务描述，规划出下一步。请根据规划结果，返回规划结果。规划结果必须以数字1、2、3开头。` 
          + "[输出格式]: ``` next  [next_step_sequence_no command|prompt]  ``` "
          + "[输出约束]: 你只能输出一个结果，不能输出多个结果。 "
    });
  
    chatMessages.push({
        role: 'system',
        content: `[任务目标]：必须完成编写创作：编写${project.value?.name || ''}:${project.value?.description || ''}`
    });

    let max = 10;
    if(chatMessages.length > max + 2) {
      let spliceCount = chatMessages.length - max - 2;
      chatMessages.splice(0, spliceCount)
    }

    try {
      const summaryPrompt = `必须从当前会话规划出下一步， 1 继续执行任务 2 在控制台执行指令<command> 3 退出任务 ，请根据当前会话内容，规划出下一步。`
      chatMessages.push({
          role: 'user',
          content: summaryPrompt
      });

      return new Promise((resolve) => {
        let summaryContent = ''
        
        llmService.chatStream(
          {
            model_id: selectedModel?.value?.id || 0,
            messages: chatMessages,
            stream: true,
            project_id: projectId.value
          },
          (data: StreamResponse) => {
            summaryContent += data.content
          },
          (error: Error) => {
            planing.value = false;
            sending.value = false;
            inputMessage.value = '请执行。'
            console.error('[ERROR] Planing error:', error)
            resolve(null)
          }
        ).then(() => {
          planing.value = false;
          sending.value = false;
          inputMessage.value = '请执行。'
          if (summaryContent.trim().length > 0) {
            const patternRegex = /```\s*[\w]{3,}\s+([\s\S]*?)```/g
            const matches = [...summaryContent.matchAll(patternRegex)]
            console.log(matches)
            let data = matches.length> 0 ? matches[0][1] : '';
            console.log(data)
            resolve(data)
          } else {
            resolve(null)
          }
        }).catch(() => {
          planing.value = false;
          sending.value = false;
          inputMessage.value = '请执行。'
          resolve(null)
        })
      })
    } catch (error) {
      planing.value = false;
      sending.value = false;
      inputMessage.value = '请执行。'
      console.error('[ERROR] Planing error:', error)
      return null
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
  debugVisible.value = ! debugVisible.value
}

// 处理容器命令执行错误
const handleCommandError = (error: string) => {
  console.log('Command error received:', error)
  // 将错误信息设置到 inputMessage
  inputMessage.value = `错误: ${error}`
  
  // 延迟一点时间，确保界面更新，然后自动点击"开始"按钮
  nextTick(() => {
    // 找到"开始"按钮并触发点击
    const startButton = document.querySelector('button[type="submit"]') as HTMLButtonElement
    if (startButton && !startButton.disabled) {
      startButton.click()
    }
  })
}

let debugVisible = ref(false)
</script>

<template class="w-full h-full">
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
                v-model="inputMessage" :readonly="sending"
                class="w-full px-3 py-2 bg-surface-100 border border-surface-200 rounded-lg  text-sm text-surface-600"
              />
              
              <button 
                type="submit" 
                :disabled="sending || !selectedModel"
                class="w-full px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2 text-sm"
              >
                <Loader2 v-if="sending" class="w-4 h-4 animate-spin" />
                <Send v-else class="w-4 h-4" />
                <span>{{ getBtnText }}</span>
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

            <audio ref="audioRef" v-if="project && project?.purpose == 'article' && playing" controls></audio>

            <button class="bg-surface-50 border px-2 py-2 border-surface-200 rounded-lg hover:bg-surface-100 transition-colors text-sm text-surface-600 flex items-center gap-2">
             <LucideFolderOutput class="w-4 h-4" /> <span>导出</span>
            </button>

            <button class="bg-surface-50 border px-2 py-2 border-surface-200 rounded-lg hover:bg-surface-100 transition-colors text-sm text-surface-600 flex items-center gap-2"
             v-if="project && project?.purpose === 'coding'"
             @click="debugCode"
             >
             <ComponentIcon class="w-4 h-4" /> <span>调试</span>
            </button>
        
            <button class="bg-surface-50 border px-2 py-2 border-surface-200 rounded-lg hover:bg-surface-100 transition-colors text-sm text-surface-600 flex items-center gap-2"
             v-if="project && project?.purpose === 'coding'" 
             @click="showHistory = !showHistory"
             >
             <CheckCircle2 class="w-4 h-4" /> <span>交互</span>
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
        <div class="flex-1 flex flex-row bg-white border-r border-surface-200 overflow-hidden">
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
          <div v-if="debugVisible"  class="flex w-3/4  overflow-auto border border-surface-200 rounded-sm p-4" >
            <ProjectContainerConfig 
              ref="projectContainerConfigRef"
              :project="project"
              :messages="projectMessages"
              @command-error="handleCommandError"
              @config-ready="handleConfigReady"
            />
          </div>
        </div>

        <div class="w-80 flex flex-col bg-surface-50 flex-shrink-0 overflow-hidden" v-if="showHistory">
          <div ref="messagesContainer" class="flex-1 overflow-y-auto p-2 space-y-2">
            <div v-if="loading" class="flex items-center justify-center h-full">
              <Loader2 class="w-6 h-6 animate-spin text-primary-500" />
            </div>
            <!-- <div v-else-if="loadError" class="flex flex-col items-center justify-center h-full text-center">
              <p class="text-red-500 mb-4 text-sm">{{ loadError }}</p>
              <button @click="router.push('/workspace')" class="btn btn-primary">返回项目列表</button>
            </div> -->
            <template v-else>
            <!-- <template> -->
              <div v-for="message in projectMessages" :key="message.id" :class="['flex gap-2', message.role === 'user' ? 'flex-row-reverse' : '']">
                <div :class="['w-5 h-5 rounded-lg flex items-center justify-center flex-shrink-0', getAvatarClass(message.role)]">
                  <component :is="getMessageIcon(message.role)" class="w-2.5 h-2.5" />
                </div>
                <div :class="['flex-1', message.role === 'user' ? 'max-w-xs' : 'max-w-full']">
                  <div :class="['rounded-lg p-2 border text-xs', getMessageClass(message.role)]">
                    <p :class="['whitespace-pre-wrap leading-relaxed text-surface-700 text-xs', message.role === 'user' ? 'text-left' : 'text-left']" v-html="parseMarkdown(message.content || '')"></p>
                  </div>
                  <div :class="['flex items-center gap-1 mt-0.5 text-xs text-surface-400', message.role === 'user' ? 'justify-end' : '']">
                    <span>{{ formatTime(message.created_at) }}</span>
                    <button v-if="message.role !== 'user'" @click="copyMessage(message)" class="p-0.5 hover:text-surface-700 transition-colors">
                      <div class="flex items-center gap-1"><Check v-if="copiedId === message.id" class="w-3 h-3 text-green-500" />
                        <Copy v-else class="w-3 h-3" />
                      </div>
                    </button>
                    <button @click="deleteMessage(message)" class="p-0.5 hover:text-surface-700 transition-colors">
                      <div class="flex items-center gap-1">
                        <Trash2 class="w-3 h-3" />
                      </div>
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
