# 工作流人工输入节点前端修改计划

## 需求分析

用户要求：
- 正常情况下，输入框不能输入内容（禁用状态）
- 当工作流执行到输入节点（human=1）时，自动打开输入框（启用状态）
- 用户输入完成并发送后，输入框再次变为禁用状态

## 现有代码分析

### 问题点

1. **FlowRunner.vue** (第181-183行)：有一个简单的文本框，但：
   - 没有连接到真正的发送 API
   - `handleHumanRowClick` 方法只是选择行，没有发送逻辑
   - 没有禁用/启用状态管理

2. **FlowRunnerHuman.vue**：完全独立的聊天界面，没有连接到 flow 节点状态

3. **flowService** 和 **useFlowStore**：缺少发送人工输入的方法

## 修改方案

### 1. 添加 API 方法

**文件**: `src/api/flow.ts`

添加 `sendHumanInput` 方法：
```typescript
sendHumanInput: (flowId: number, nodeId: number, message: string) =>
  api.post(flowApi, `/api/flows/${flowId}/nodes/${nodeId}/human-input`, { message }),
```

### 2. 添加 Store 方法

**文件**: `src/stores/flow.ts`

添加 `sendHumanInput` 方法：
```typescript
const sendHumanInput = async (flowId: number, nodeId: number, message: string) => {
  loading.value = true
  error.value = null
  try {
    await flowService.sendHumanInput(flowId, nodeId, message)
  } catch (err: unknown) {
    error.value = err instanceof Error ? err.message : 'Failed to send human input'
    throw err
  } finally {
    loading.value = false
  }
}
```

### 3. 修改 FlowRunner.vue

- 添加 `selectedHumanNode` 状态，存储当前需要人工输入的节点
- 添加 `humanInputMessage` 状态，存储用户输入内容
- 添加 `canInput` 计算属性，判断是否有节点需要人工输入
- 修改输入框：根据 `canInput` 控制禁用状态
- 添加发送逻辑：调用 `sendHumanInput` API

### 4. 修改 FlowRunnerHuman.vue

- 移除独立的聊天界面，改为显示 flow 运行时会话消息
- 添加输入框禁用逻辑

## 文件修改清单

| 文件 | 修改内容 |
|------|----------|
| `src/api/flow.ts` | 添加 `sendHumanInput` API 方法 |
| `src/stores/flow.ts` | 添加 `sendHumanInput` store 方法 |
| `src/views/flows/FlowRunner.vue` | 修改输入框逻辑，实现禁用/启用切换 |
| `src/views/flows/FlowRunnerHuman.vue` | 适配 flow 会话消息显示 |

## 风险处理

1. **API 调用失败**：添加错误处理，显示错误提示
2. **节点状态更新延迟**：轮询机制已经存在（5秒），会自动更新节点状态
3. **并发输入**：同一时刻只有一个节点 human=1，避免并发问题

## 验证方式

1. 启动工作流，观察输入框是否禁用
2. 当流程执行到输入节点（human=1）时，观察输入框是否自动启用
3. 输入内容并发送，观察输入框是否再次禁用
4. 观察节点状态是否更新为完成