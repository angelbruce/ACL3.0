<script lang="ts" setup>
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { workspaceService } from '@/api/workspace'
import '@xterm/xterm/css/xterm.css'

interface Props {
  projectId: number
  containerName?: string
  configId?: number
  autoCommand?: string  // 自动执行的命令
}

interface CommandResult {
  success: boolean
  output: string
  error?: string
}

const emit = defineEmits<{
  'command-result': [result: CommandResult]
}>()

const props = withDefaults(defineProps<Props>(), {
  // height: '300px'
})

const terminalRef = ref<HTMLDivElement | null>(null)
const inputRef = ref<HTMLInputElement | null>(null)
const commandHistory = ref<string[]>([])
const historyIndex = ref(-1)
const currentInput = ref('')
const isExecuting = ref(false)
const commandInput = ref('')
const containerStatus = ref<'running' | 'stopped' | 'unknown'>('unknown')
const isContainerReady = ref(false)

let term: Terminal | null = null
let fitAddon: FitAddon | null = null

// 检测容器状态
const checkContainerStatus = async () => {
  try {
    const response = await workspaceService.getContainerStatus(props.projectId, props.configId)
    
    // 优先使用 target_status，如果没有则检查所有容器
    const targetStatus = response.target_status
    const isRunning = targetStatus 
      ? targetStatus.state?.toLowerCase() === 'running' || targetStatus.state?.toLowerCase() === 'up'
      : response.statuses.some((s: any) => s.state?.toLowerCase() === 'running' || s.state?.toLowerCase() === 'up')
    
    containerStatus.value = isRunning ? 'running' : 'stopped'
    isContainerReady.value = isRunning
    
    // 只在容器可用时显示状态
    if (isContainerReady.value) {
      updateTerminalStatus()
    }
  } catch (error) {
    containerStatus.value = 'unknown'
    isContainerReady.value = false
  }
}

const updateTerminalStatus = () => {
  if (!term) return
  // term.writeln('\x1b[32m[Container Ready]\x1b[0m')
  term.write('$ ')
}

const initTerminal = () => {
  if (!terminalRef.value) return

  term = new Terminal({
    cursorBlink: true,
    fontSize: 14,
    fontFamily: 'Menlo, Monaco, "Courier New", monospace',
    theme: {
      background: '#1e1e1e',
      foreground: '#d4d4d4',
      cursor: '#ffffff',
      selection: 'rgba(255, 255, 255, 0.3)',
    },
    rows: 28,
    cols: 120,
    scrollback: 1000,
    disableStdin: !isContainerReady.value,
  })

  fitAddon = new FitAddon()
  term.loadAddon(fitAddon)

  term.open(terminalRef.value)
  fitAddon.fit()

  // 欢迎信息
  term.writeln('\x1b[1;32mContainer Terminal\x1b[0m')
  term.writeln(`Project: ${props.projectId}, Container: ${props.containerName || 'default'}`)
  term.writeln('')
  
  // 如果容器已就绪，显示状态
  if (isContainerReady.value) {
    updateTerminalStatus()
  } else {
    term.write('$ ')
  }

  // 监听输入
  term.onData((data) => {
    if (!isContainerReady.value) return
    
    if (data === '\r') {
      executeCommand()
    } else if (data === '\x7f') {
      if (currentInput.value.length > 0) {
        currentInput.value = currentInput.value.slice(0, -1)
        term?.write('\b \b')
      }
    } else if (data === '\x03') {
      term?.write('^C')
      currentInput.value = ''
      historyIndex.value = -1
    } else if (data === '\x1b[A') {
      if (historyIndex.value < commandHistory.value.length - 1) {
        historyIndex.value++
        const cmd = commandHistory.value[commandHistory.value.length - 1 - historyIndex.value]
        clearCurrentLine()
        currentInput.value = cmd
        term?.write(cmd)
      }
    } else if (data === '\x1b[B') {
      if (historyIndex.value > 0) {
        historyIndex.value--
        const cmd = commandHistory.value[commandHistory.value.length - 1 - historyIndex.value]
        clearCurrentLine()
        currentInput.value = cmd
        term?.write(cmd)
      } else if (historyIndex.value === 0) {
        historyIndex.value = -1
        clearCurrentLine()
        currentInput.value = ''
      }
    } else {
      currentInput.value += data
      term?.write(data)
    }
  })
}

const clearCurrentLine = () => {
  if (term) {
    term.write('\r\x1b[K')
    term.write('\r')
  }
}

const executeCommand = async (cmd?: string, onComplete?: (result: CommandResult) => void) => {
  if (!term || isExecuting.value) return
  await checkContainerStatus()
  if (!isContainerReady.value) {
    term.writeln('\x1b[33mContainer is not running. Please start the container first.\x1b[0m')
    term.write('$ ')
    if (onComplete) {
      onComplete({ success: false, output: '', error: 'Container is not running' })
    }
    return
  }

  const command = cmd !== undefined ? cmd : currentInput.value.trim()
  if (!command) {
    term.writeln('')
    term.write('$ ')
    return
  }

  // 如果是手动输入的命令（没有传入 cmd 参数），才添加到历史记录
  if (cmd === undefined) {
    commandHistory.value.push(command)
    historyIndex.value = -1
  }

  term.writeln('')

  isExecuting.value = true
  
  let outputData = ''
  let hasError = false
  let errorMsg = ''

  workspaceService.executeCommandStream(
    props.projectId,
    props.configId || 0,
    command,
    (data) => {
      hasError = data.includes('stderr')
      data = data.replace(/stderr:/g, '').replace(/stdout:/g, '')
      if (term) {
        term.writeln(data)
      }

      outputData += data 
      errorMsg = outputData.trim();
    },
    (error) => {
      if (term) {
        term.writeln(`\x1b[31mError: ${error.message}\x1b[0m`)
      }
      hasError = true
      errorMsg = error.message
      currentInput.value = ''
      isExecuting.value = false
      if (term) {
        term.write('$ ')
      }
    },
    () => {
      currentInput.value = ''
      isExecuting.value = false
      if (term) {
        term.write('$ ')
      }
      
      // 执行完成回调
      if (onComplete) {
        onComplete({
          success: !hasError,
          output: outputData,
          error: hasError ? errorMsg : undefined
        })
      }
      
      // 同时触发 emit 事件
      emit('command-result', {
        success: !hasError,
        output: outputData,
        error: hasError ? errorMsg : undefined
      })
    }
  )
}

// 自动执行命令（当 autoCommand 变化时）
watch(() => props.autoCommand, (newCommand) => {
  if (newCommand && newCommand.trim()) {
    nextTick(async () => {
      // 先检查容器状态，确保容器就绪后再执行
      await checkContainerStatus()
      if (isContainerReady.value) {
        executeCommand(newCommand.trim())
      }
    })
  }
})



const handleInputSubmit = async () => {
  if (!term || !commandInput.value.trim()) return
  const cmd = commandInput.value.trim()
  term.writeln('\r\n')
  // term.writeln("$ ");
  for (const char of cmd) {
    term.write(char)
  }
  currentInput.value = cmd
  commandInput.value = ''
  // 回车后失焦
  inputRef.value?.blur()
  await executeCommand()
  inputRef.value?.focus()
}

const handleInputClick = async () => {
  if (isContainerReady.value) {
    focusInput()
    return
  }
  await checkContainerStatus()
  focusInput()
}

const focusInput = () => {
  if (isContainerReady.value) {
    inputRef.value?.focus()
  }
}

const handleResize = () => {
  if (fitAddon) {
    fitAddon.fit()
  }
}

defineExpose({
  executeCommand,
  checkContainerStatus,
  isContainerReady
})

onMounted(() => {
  initTerminal()
  window.addEventListener('resize', handleResize)
  nextTick(() => {
    checkContainerStatus()
  })
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  if (term) {
    term.dispose()
  }
})

watch(() => props.containerName, () => {
  if (term) {
    term.clear()
    term.writeln('\x1b[1;32mContainer Terminal\x1b[0m')
    term.writeln(`Project: ${props.projectId}, Container: ${props.containerName || 'default'}`)
    term.writeln('')
    updateTerminalStatus()
  }
  checkContainerStatus()
})

watch(isContainerReady, (ready) => {
  if (term) {
    term.options.disableStdin = !ready
  }
})
</script>

<template>
  <div class="terminal-container flex flex-col w-full h-full">
    <!-- 终端显示区 -->
    <div 
      ref="terminalRef" 
      class="terminal-output flex-1 overflow-hidden cursor-pointer"
      :style="{  opacity: isContainerReady ? 1 : 0.7 }"
      @click="handleInputClick"
    ></div>
    
    <!-- 命令输入区 -->
    <div class="terminal-input flex items-center gap-2 mt-2 p-2 bg-surface-100 rounded border border-surface-200">
      <span class="text-surface-600 font-mono">$</span>
      <input
        ref="inputRef"
        v-model="commandInput"
        type="text"
        class="flex-1 bg-transparent outline-none font-mono text-surface-700 disabled:bg-gray-100"
        placeholder="Container not ready..."
        :disabled="!isContainerReady || isExecuting"
        @keyup.enter="handleInputSubmit"
        @click="handleInputClick"
      />
      <button 
        class="px-3 py-1 bg-primary-500 text-white rounded text-sm hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed"
        :disabled="isExecuting || !commandInput.trim()"
        @click="handleInputSubmit"
      >
        {{ isExecuting ? 'Running...' : 'Execute' }}
      </button>
    </div>

    <!-- 容器状态 -->
    <div class="mt-1 flex items-center justify-between text-xs">
      <div class="flex items-center gap-2">
        <span class="text-surface-500">Container: {{ containerName || 'default' }}</span>
        <span v-if="isExecuting" class="text-yellow-500">Executing...</span>
      </div>
      <div class="flex items-center gap-2">
        <span 
          class="px-2 py-0.5 rounded text-white text-xs cursor-pointer hover:opacity-80"
          :class="{
            'bg-green-500': containerStatus === 'running',
            'bg-red-500': containerStatus === 'stopped',
            'bg-gray-500': containerStatus === 'unknown'
          }"
          @click="checkContainerStatus"
        >
          {{ containerStatus === 'running' ? 'Running' : containerStatus === 'stopped' ? 'Stopped' : 'Check' }}
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.terminal-container {
  cursor: default;
}

.terminal-output {
  background: #1e1e1e;
  border-radius: 4px;
  padding: 8px;
  transition: opacity 0.3s;
}

.terminal-output :deep(.xterm) {
  padding: 0;
}

.terminal-output :deep(.xterm-viewport) {
  overflow-y: auto !important;
}

.terminal-input input {
  caret-color: #3b82f6;
}

.terminal-input input:disabled {
  caret-color: transparent;
}
</style>
