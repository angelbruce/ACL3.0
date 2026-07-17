# 工作流输入节点处理修复计划

## 问题分析

当前代码没有正确处理输入节点（`type="input"`），存在以下问题：

1. **输入节点未被识别**：创建运行时节点时没有检查顶点的 `type` 是否为 `"input"`，不会自动设置 `human=1`

2. **输入节点执行逻辑缺失**：输入节点应该直接等待人工输入，不调用 LLM，但当前逻辑没有区分

3. **人工输入后节点未完成**：用户输入内容后，节点没有自动完成并继续流程

## 修复方案

### 1. 修改 FlowRuntimeNodeCreate 结构体

添加 `human` 字段，用于指定节点是否需要人工输入。

### 2. 修改 state_machine.rs

在 `start_flow` 和 `complete_node` 方法中，创建运行时节点时检查顶点类型：
- 如果 `type == "input"`，设置 `human=1`
- 如果 `type == "start"` 或普通节点，设置 `human=0`

### 3. 修改 repository.rs

在 `create_flow_runtime_nodes` 方法中，使用传入的 `human` 值，而不是硬编码为 0。

### 4. 修改 executor.rs

在 `NodeAgent::run` 方法中，增加输入节点处理逻辑：
- 如果 `action_id == 0` 且 `human == 1`，直接等待人工输入
- 人工输入后，保存消息并完成节点

### 5. 修改 handlers.rs

在人工输入处理中，收到输入后：
- 保存用户输入到会话
- 完成当前节点并触发下一个节点

## 文件修改清单

| 文件 | 修改内容 |
|------|----------|
| `src/repository.rs` | `FlowRuntimeNodeCreate` 添加 `human` 字段，`create_flow_runtime_nodes` 使用该字段 |
| `src/state_machine.rs` | 创建节点时检查顶点类型，设置 `human` 值 |
| `src/executor.rs` | 输入节点直接等待人工输入，不调用 LLM |
| `src/handlers.rs` | 人工输入后完成节点并继续流程 |

## 验证方式

1. 创建包含输入节点的工作流
2. 启动工作流，观察输入节点是否设置 `human=1`
3. 在前端输入内容，观察节点是否完成并继续下一个节点