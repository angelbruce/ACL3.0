# Vec-SVC 项目记忆文档（超级详细版）

> 最后更新：2026-06-26
> 项目状态：M1-M5 后端开发全部完成（35/35任务），Docker 镜像编译通过（`acl-vec-svc:latest`，168MB），准备前端开发

---

## 目录

1. [项目背景与演进历程](#1-项目背景与演进历程)
2. [核心需求与设计目标](#2-核心需求与设计目标)
3. [技术选型决策记录](#3-技术选型决策记录)
4. [系统架构设计](#4-系统架构设计)
5. [数据库设计（21张表）](#5-数据库设计21张表)
6. [核心功能模块详解](#6-核心功能模块详解)
7. [API 接口设计（完整清单）](#7-api-接口设计完整清单)
8. [部署方案](#8-部署方案)
9. [遇到的问题与完整解决方案](#9-遇到的问题与完整解决方案)
10. [经验教训与最佳实践](#10-经验教训与最佳实践)
11. [当前状态与下一步计划](#11-当前状态与下一步计划)
12. [关键文件索引](#12-关键文件索引)

---

## 1. 项目背景与演进历程

### 1.1 起源：文件容器分配问题

项目最初的需求并不是 RAG 平台，而是**代码容器部署中的文件归属问题**：

- **问题背景**：容器代码部署缺少一个重要环节——代码如何划分到不同的容器中
- **核心问题**：有些代码是多个容器共享的（共有代码），有些是专属容器的
- **解决方案**：设计新表存储代码所属容器配置

**关键决策**：
- `container_config_id=0` 表示共享代码（共有代码）
- 文件与容器是多对多关系（一个文件可属于多个容器）
- 需要 LLM 辅助自动分类文件归属

**相关表**：
- `project_container_configs` - 容器配置表
- `project_file_container_assignments` - 文件容器分配表

### 1.2 演进：LLM 返回内容分离

在开发 LLM 客户端时发现：
- LLM 返回的 JSON 经常包裹在 markdown 代码块中（```json ... ```）
- 需要将 `reasoning_content`（思考过程）与 `content`（最终输出）分离

**解决方案**：
- 在 `llm_client.rs` 中添加 `.trim_start_matches("```json")` 处理
- 在响应结构中增加 `reason_content` 字段

### 1.3 转折：Vec-SVC RAG 平台诞生

随着项目推进，用户提出需要一个**向量服务（vec-svc）**，从最初的简单文档检索，逐步演进为企业级 RAG 知识管理平台。

**演进路线**：
1. **第一代**：单纯的文档搜索引擎（用户评价："没发现有什么有价值的地方，价值太单一"）
2. **第二代**：加入知识蒸馏、幻觉校验、分类分级、知识边界等差异化能力
3. **第三代**：定位为企业级 RAG 知识管理平台，具备 68 个高价值应用场景
4. **第四代**：完成 M1-M5 全部里程碑，后端功能完整，准备前端开发

---

## 2. 核心需求与设计目标

### 2.1 核心价值主张

Vec-SVC 不是普通的文档检索系统，而是**企业级知识资产管理平台**，核心差异化能力：

| 能力 | 描述 | 价值 |
|------|------|------|
| **知识蒸馏** | 从原始文档中提取结构化知识点（摘要、关键词、问答对） | 将非结构化文档转化为可计算的知识 |
| **幻觉校验** | 事实支撑校验 + 知识图谱一致性校验 | 保障 LLM 输出的可信度，标记可疑事实 |
| **分类分级** | 5维分类（主题/内容类型/业务域/技术栈/受众）+ 5级评分（难度/重要性/成熟度/时效性/可信度） | 让知识有序、可发现、可传承 |
| **知识边界** | 5层可见性（私有→项目→团队→组织→公开）+ 细粒度共享 | 保障知识安全，实现精准共享 |
| **项目上下文感知** | 基于项目配置的检索策略（chunk_size、top_k、min_score 等） | 不同项目用不同的 RAG 参数 |
| **知识图谱** | 实体识别 + 关系抽取 + 三元组存储 | 构建结构化知识网络，支撑智能问答 |
| **异步处理** | 基于数据库的任务队列，后台处理耗时操作 | 大文档处理不阻塞，用户体验更好 |
| **性能优化** | 多级缓存 + BM25重排序 + 语义分块 | 搜索更快、更准 |

### 2.2 设计目标

- **高性能**：纯 Rust 实现，GGUF 查表方式做 embedding（无需前向传播），多级缓存
- **可扩展**：微服务架构，独立部署，支持多项目、多租户
- **安全可控**：细粒度知识边界控制，权限继承规则完善
- **企业级**：版本管理、统计分析、批量导入导出等生产级功能

---

## 3. 技术选型决策记录

### 3.1 核心组件选型

| 组件 | 选型 | 决策原因 | 替代方案 |
|------|------|----------|----------|
| **向量数据库** | Milvus | 高性能、云原生、支持多种索引 | Pinecone, Weaviate, pgvector |
| **关系数据库** | PostgreSQL | 项目已在使用，生态成熟，JSON 支持好 | MySQL |
| **对象存储** | MinIO | 私有化部署、S3 兼容、轻量 | AWS S3, 阿里云 OSS |
| **Tokenizer** | shimmytok | 纯 Rust、无需 CUDA、轻量 | HuggingFace tokenizers, tiktoken-rs |
| **Embedding 模型** | gemma-4-E4B | 固定 2560 维、GGUF 格式 | bge-large, text-embedding-3-large |
| **Web 框架** | Axum 0.7 | 异步、高性能、Tokio 生态 | Actix-web, Rocket |
| **ORM** | Diesel 2.0 | Rust 生态成熟的 ORM，代码生成 | SeaORM |
| **部署方式** | Docker 多阶段构建 | 环境一致性、镜像小 | 裸机部署 |

### 3.2 关键技术决策详解

#### 决策 1：Embedding 用查表方式，不用前向传播

**背景**：
- 最初尝试用 `candle-core` + `gguf-llms` 做完整的模型推理
- 遇到 CUDA 编译问题（Tesla P40 不支持 FP16 atomicAdd）
- CPU 推理速度慢

**方案**：
- 利用 embedding 层的本质：token embedding 就是个查表操作
- 直接从 GGUF 文件中提取 `token_embd` 权重
- 分词后用 token ID 直接查表，然后平均 pooling
- **无需任何前向传播计算**

**优势**：
- 速度极快（纯内存查表）
- 无需 GPU
- 实现简单
- 资源占用低

**局限**：
- 只有静态词向量，没有上下文感知
- 效果比完整模型差一些（但对于检索任务够用）

#### 决策 2：模型文件外部挂载

**背景**：
- Embedding 模型文件很大（gemma-4-E4B 约数 GB）
- 如果打进 Docker 镜像，镜像会非常大
- 每次更新模型都要重新构建镜像

**方案**：
- 模型文件通过 Docker volume 挂载到 `/models` 目录
- 启动时通过 `EMBEDDING_MODEL_PATH` 环境变量指定模型路径
- 镜像本身只包含可执行文件（几十 MB）

**优势**：
- 镜像体积小
- 模型更新不需要重新构建镜像
- 可以在多个容器间共享模型文件

#### 决策 3：基于数据库的异步任务队列

**背景**：
- 需要处理大文档上传、重新索引等耗时操作
- 同步处理会导致请求超时
- 不想引入 Redis 等额外依赖

**方案**：
- 基于 PostgreSQL 的任务队列表（`tasks`）
- 后台 worker 轮询方式处理任务
- 支持任务状态追踪（pending/processing/completed/failed/cancelled）
- 支持进度查询和取消

**优势**：
- 无需额外依赖（Redis/RabbitMQ）
- 事务性保证（任务状态和数据操作在同一事务中）
- 简单可靠

**局限**：
- 吞吐量不如 Redis 队列
- 需要轮询，有一定延迟

#### 决策 4：多级缓存策略

**背景**：
- 搜索是高频操作，需要优化响应速度
- 数据库查询和向量检索都有一定开销

**方案**：
- 搜索查询缓存（5分钟TTL，1000条上限）
- 搜索建议缓存（10分钟TTL，500条上限）
- 文档缓存（10分钟TTL，200条上限）
- LRU + TTL 双重淘汰策略
- 缓存命中率统计

**优势**：
- 热门查询响应极快
- 减少数据库和 Milvus 压力

---

## 4. 系统架构设计

### 4.1 整体架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Vec-SVC 服务                                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────┐   ┌──────────────────────────────────────────┐           │
│  │  API 层  │──▶│              业务逻辑层 (AppState)          │           │
│  │ (Axum)   │   │                                          │           │
│  └──────────┘   │  ┌──────────┐ ┌──────────┐ ┌──────────┐   │           │
│       │         │  │SearchCache│ │RerankSvc │ │Chunker   │   │           │
│       │         │  │ (缓存)    │ │(重排序)  │ │(语义分块)│   │           │
│       │         │  └──────────┘ └──────────┘ └──────────┘   │           │
│       │         │  ┌──────────┐ ┌──────────┐ ┌──────────┐   │           │
│       │         │  │DistillSvc│ │GraphSvc  │ │VerifySvc │   │           │
│       │         │  │(知识蒸馏)│ │(知识图谱)│ │(幻觉校验)│   │           │
│       │         │  └──────────┘ └──────────┘ └──────────┘   │           │
│       │         │  ┌──────────┐ ┌──────────┐ ┌──────────┐   │           │
│       │         │  │TaskQueue │ │AnalyticSvc││VersionSvc│   │           │
│       │         │  │(异步队列)│ │(统计分析) ││(版本管理)│   │           │
│       │         │  └──────────┘ └──────────┘ └──────────┘   │           │
│       │         └──────────────────────────────────────────┘           │
│       │                                    │                           │
│       │          ┌─────────┐              ▼                           │
│       │          │Tokenizer│        ┌─────────────┐                   │
│       │          │         │        │  PostgreSQL │                   │
│       │          └────┬────┘        │  (21张表)    │                   │
│       │               │             └──────┬──────┘                   │
│       │               ▼                    │                           │
│       │         ┌─────────┐                ▼                           │
│       │         │Embedding│            ┌─────────┐                    │
│       │         │ Service │            │  MinIO  │                    │
│       │         │ (查表)   │            │(原始文件)│                    │
│       │         └────┬────┘            └─────────┘                    │
│       │              │                                                │
│       │              ▼                                                │
│       │         ┌─────────────┐                                       │
│       │         │   Milvus    │                                       │
│       │         │  (向量库)   │                                       │
│       │         └─────────────┘                                       │
│       │                                                               │
└───────┴───────────────────────────────────────────────────────────────┘
```

### 4.2 模块划分

| 模块 | 文件 | 职责 |
|------|------|------|
| **入口** | `main.rs` | 启动服务、初始化状态 |
| **路由** | `routes.rs` | API 路由定义 |
| **Handler** | `handlers/*.rs` | HTTP 请求处理 |
| **业务状态** | `app_state.rs` | 应用状态、业务逻辑编排 |
| **Embedding** | `embedding.rs` | GGUF 嵌入查表服务 |
| **Tokenizer** | `tokenizer.rs` | 文本分词（占位实现） |
| **GGUF 加载器** | `loader.rs` | GGUF 文件解析 |
| **Milvus 客户端** | `milvus.rs` | 向量数据库操作 |
| **MinIO 服务** | `minio.rs` | 对象存储操作 |
| **任务队列** | `task_queue.rs` | 异步任务处理 |
| **知识图谱** | `knowledge_graph.rs` | 实体和关系提取 |
| **校验服务** | `verification.rs` | 事实校验和图谱一致性 |
| **搜索建议** | `search_suggestions.rs` | 搜索建议和自动补全 |
| **缓存服务** | `cache.rs` | 多级缓存 |
| **重排序服务** | `rerank.rs` | BM25 + 向量混合排序 |
| **语义分块** | `semantic_chunk.rs` | 智能语义分块 |
| **统计分析** | `analytics.rs` | 访问统计和热度分析 |
| **版本管理** | `version_control.rs` | 文档版本和差异对比 |
| **导入导出** | `import_export.rs` | 批量导入导出 |
| **知识蒸馏** | `distillation.rs` | 知识点提取 |
| **分类分级** | `taxonomy.rs` | 分类树和分级评分 |
| **知识边界** | `boundary.rs` | 可见性控制和共享 |
| **模型定义** | `model.rs` | 数据模型 |
| **关系库仓储** | `rdb_repository.rs` | PostgreSQL 操作 |
| **Schema** | `schema.rs` | 数据库表定义 |

### 4.3 数据流（文档检索场景）

```
用户查询
   │
   ▼
查询缓存检查
   │── 命中 ──▶ 返回缓存结果
   │
   ▼ 未命中
分词 (Tokenizer)
   │
   ▼
向量化 (Embedding Service - 查表)
   │
   ▼
向量检索 (Milvus) + 项目过滤
   │
   ▼
结果重排序 (BM25 + 向量融合)
   │
   ▼
结果多样化 (避免同一文档过多结果)
   │
   ▼
缓存结果
   │
   ▼
记录搜索日志
   │
   ▼
返回给用户
```

---

## 5. 数据库设计（21张表）

### 5.1 核心文档表

| 表名 | 用途 | 关键字段 |
|------|------|----------|
| `documents` | 文档主表 | id, project_id, title, content_hash, status, boundary_level, chunk_count |
| `document_chunks` | 文档分块表 | id, document_id, chunk_index, content, vector_id, chunk_hash |
| `document_versions` | 文档版本表 | id, document_id, version_number, content, change_note, created_by |

### 5.2 知识蒸馏表

| 表名 | 用途 | 关键字段 |
|------|------|----------|
| `knowledge_points` | 知识点表 | id, document_id, title, summary, key_phrases(JSON), qna_pairs(JSON), confidence |
| `knowledge_entities` | 知识实体表 | id, name, entity_type, description, aliases(JSON), source_document_id |
| `knowledge_relations` | 实体关系表 | id, source_entity_id, target_entity_id, relation_type, evidence_text, confidence |

### 5.3 分类分级表

| 表名 | 用途 | 关键字段 |
|------|------|----------|
| `document_categories` | 分类表 | id, name, parent_id, type (主题/内容类型/业务域/技术栈/受众) |
| `document_category_mappings` | 文档分类关联 | document_id, category_id |
| `document_levels` | 分级表 | id, name, level_type (难度/重要性/成熟度/时效性/可信度), score |
| `document_level_mappings` | 文档分级关联 | document_id, level_id |

### 5.4 知识边界表

| 表名 | 用途 | 关键字段 |
|------|------|----------|
| `document_boundaries` | 文档边界配置 | document_id, visibility_level (私有/项目/团队/组织/公开), created_by |
| `document_shares` | 边界共享记录 | id, document_id, target_type, target_id, permissions, expires_at |

### 5.5 配置表

| 表名 | 用途 | 关键字段 |
|------|------|----------|
| `project_rag_configs` | 项目 RAG 配置 | project_id, chunk_size, chunk_overlap, top_k, min_score, temperature |

### 5.6 任务队列表

| 表名 | 用途 | 关键字段 |
|------|------|----------|
| `tasks` | 任务队列表 | id, task_type, status, payload(JSON), progress, error_message, created_at, completed_at |

### 5.7 校验冲突表

| 表名 | 用途 | 关键字段 |
|------|------|----------|
| `verification_conflicts` | 校验冲突表 | id, project_id, query_text, llm_summary, conflict_type, confidence_score, resolved |

### 5.8 统计分析表

| 表名 | 用途 | 关键字段 |
|------|------|----------|
| `access_logs` | 访问日志表 | id, document_id, user_id, access_type, ip_address, created_at |
| `search_logs` | 搜索日志表 | id, user_id, project_id, query_text, result_count, response_time_ms, created_at |

### 5.9 容器配置表

| 表名 | 用途 | 关键字段 |
|------|------|----------|
| `project_container_configs` | 容器配置表 | id, project_id, container_name, config_data |
| `project_file_container_assignments` | 文件容器分配表 | id, file_path, container_config_id |

---

## 6. 核心功能模块详解

### 6.1 Embedding 服务

**文件**：[embedding.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/embedding.rs)

**核心原理**：
- 从 GGUF 文件中提取 `token_embd` 权重（shape: [vocab_size, embedding_dim]）
- 分词得到 token IDs
- 每个 token ID 查表得到对应向量
- 所有 token 向量取平均，得到文本的 embedding

**配置**：
- 模型：gemma-4-E4B
- 维度：2560
- 模型路径：`EMBEDDING_MODEL_PATH` 环境变量

### 6.2 知识蒸馏服务

**文件**：[distillation.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/distillation.rs)

**核心功能**：
- 文档摘要生成（一句话 + 详细摘要）
- 关键词提取
- 问答对生成
- 知识点持久化到 `knowledge_points` 表

### 6.3 知识图谱服务

**文件**：[knowledge_graph.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/knowledge_graph.rs)

**核心功能**：
- 实体提取（组织、人物、URL、日期等）
- 关系抽取（提供、支持、使用、包含等）
- 实体去重和合并
- 三元组存储（实体-关系-实体）

### 6.4 幻觉校验服务

**文件**：[verification.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/verification.rs)

**核心功能**：
- 事实支撑校验：从 LLM 回答中提取事实，在知识库中搜索证据，计算置信度
- 图谱一致性校验：验证实体和关系是否存在于知识图谱中，检测冲突
- 冲突记录：将校验结果保存到 `verification_conflicts` 表

### 6.5 重排序服务

**文件**：[rerank.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/rerank.rs)

**核心功能**：
- BM25 + 向量相似度加权融合（默认 3:7）
- 混合搜索结果合并去重
- 结果多样化（避免同一文档过多结果）

### 6.6 语义分块服务

**文件**：[semantic_chunk.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/semantic_chunk.rs)

**核心功能**：
- 按段落/标题自然分割
- 重叠分块（保留上下文连续性）
- 自动识别标题、列表、代码块等类型
- 大段落递归细分

### 6.7 异步任务队列

**文件**：[task_queue.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/task_queue.rs)

**核心功能**：
- 基于数据库的任务队列表
- 后台 worker 轮询处理
- 支持任务状态追踪（pending/processing/completed/failed/cancelled）
- 支持进度查询和取消

---

## 7. API 接口设计（完整清单）

### 7.1 搜索接口

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/search` | 通用搜索 |
| GET | `/api/search/suggest` | 搜索建议 |
| GET | `/api/search/autocomplete` | 搜索自动补全 |
| GET | `/api/projects/{project_id}/search` | 项目内搜索 |

### 7.2 文档接口

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/documents/text` | 添加文本文档 |
| POST | `/api/documents/file` | 上传文件文档 |
| GET | `/api/documents` | 文档列表 |
| GET | `/api/documents/{id}` | 文档详情 |
| DELETE | `/api/documents/{id}` | 删除文档 |
| POST | `/api/documents/{id}/reindex` | 重新索引 |

### 7.3 对象存储接口

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/storage/upload` | 文件上传到 MinIO |
| GET | `/api/storage/download/{object_name}` | 下载文件 |
| DELETE | `/api/storage/delete/{object_name}` | 删除文件 |
| GET | `/api/storage/presigned/{object_name}` | 获取预签名 URL |
| GET | `/api/storage/projects/{project_id}/files` | 项目文件列表 |
| GET | `/api/storage/health` | 检查 MinIO 连接 |

### 7.4 任务队列接口

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/tasks` | 创建任务 |
| GET | `/api/tasks` | 任务列表 |
| GET | `/api/tasks/{task_id}` | 任务详情 |
| DELETE | `/api/tasks/{task_id}` | 取消任务 |
| GET | `/api/tasks/{task_id}/progress` | 任务进度 |

### 7.5 知识图谱接口

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/graph/extract` | 从文档提取实体和关系 |
| GET | `/api/graph/entities` | 搜索实体 |
| GET | `/api/graph/entities/{entity_id}` | 获取实体详情 |
| DELETE | `/api/graph/entities/{entity_id}` | 删除实体 |
| GET | `/api/graph/entities/{entity_id}/relations` | 获取实体关系 |
| GET | `/api/graph/projects/{project_id}/entities` | 项目实体列表 |

### 7.6 校验接口

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/verification/facts` | 事实支撑校验 |
| POST | `/api/verification/graph` | 图谱一致性校验 |
| GET | `/api/verification/conflicts` | 获取冲突列表 |
| PUT | `/api/verification/conflicts/{conflict_id}` | 解决冲突 |

### 7.7 知识蒸馏接口

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/documents/{id}/distill` | 触发知识蒸馏 |
| GET | `/api/documents/{id}/knowledge-points` | 获取文档知识点 |
| POST | `/api/distill/preview` | 预览蒸馏效果 |
| DELETE | `/api/knowledge-points/{id}` | 删除知识点 |

### 7.8 分类分级接口

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/categories` | 创建分类 |
| GET | `/api/categories` | 分类列表 |
| GET | `/api/categories/{id}` | 分类详情 |
| PUT | `/api/categories/{id}` | 更新分类 |
| DELETE | `/api/categories/{id}` | 删除分类 |
| GET | `/api/categories/{parent_id}/children` | 子分类列表 |
| POST | `/api/documents/{document_id}/categories` | 文档打分类标签 |
| GET | `/api/documents/{document_id}/categories` | 文档分类列表 |
| POST | `/api/levels` | 创建分级 |
| GET | `/api/levels` | 分级列表 |
| POST | `/api/documents/{document_id}/levels` | 文档打分 |

### 7.9 知识边界接口

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/documents/{document_id}/visibility` | 设置文档可见性 |
| GET | `/api/documents/{document_id}/boundary` | 获取文档边界 |
| POST | `/api/documents/{document_id}/check-access` | 检查访问权限 |
| POST | `/api/documents/accessible` | 获取可访问文档列表 |
| POST | `/api/shares` | 创建文档共享 |
| POST | `/api/shares/batch` | 批量创建共享 |
| DELETE | `/api/shares/{id}` | 删除共享 |
| GET | `/api/documents/{document_id}/shares` | 文档共享列表 |

### 7.10 版本管理接口

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/documents/{document_id}/versions` | 版本列表 |
| POST | `/api/documents/{document_id}/versions` | 创建新版本 |
| GET | `/api/versions/{version_id}` | 版本详情 |
| GET | `/api/versions/compare` | 版本对比 |
| POST | `/api/documents/{document_id}/rollback` | 版本回滚 |

### 7.11 统计分析接口

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/analytics/summary` | 统计概览 |
| GET | `/api/analytics/document` | 单文档统计 |

### 7.12 导入导出接口

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/import/documents` | 批量导入文档 |
| GET | `/api/export/documents` | 导出文档（JSON/Markdown/CSV） |
| GET | `/api/export/knowledge-graph` | 导出知识图谱 |

### 7.13 嵌入接口（内部使用）

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/embed` | 单文本向量化 |
| POST | `/api/embed/batch` | 批量向量化 |
| GET | `/api/health` | 健康检查 |

---

## 8. 部署方案

### 8.1 Docker 构建

**基础镜像**：`rust-base:latest`
- 基于 Debian 12（bookworm-slim）
- Rust 1.89（通过 rustup 安装）
- 预下载所有依赖（加速构建）

**运行时镜像**：`debian:bookworm-slim`
- 只包含运行时依赖（libssl, ca-certificates）
- 从 builder 阶段复制可执行文件

**Dockerfile 位置**：[vec-svc/Dockerfile](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/Dockerfile)

**镜像大小**：168MB（含应用 + 运行时依赖）

### 8.2 环境变量

| 变量 | 必填 | 默认值 | 说明 |
|------|------|--------|------|
| `PORT` | 否 | 8080 | 服务端口 |
| `EMBEDDING_MODEL_PATH` | 是 | - | GGUF 模型文件路径 |
| `EMBEDDING_DIM` | 否 | 2560 | Embedding 维度 |
| `MILVUS_URL` | 是 | - | Milvus 服务地址 |
| `DATABASE_URL` | 是 | - | PostgreSQL 连接串 |
| `MINIO_ENDPOINT` | 否 | - | MinIO 地址 |
| `MINIO_ACCESS_KEY` | 否 | - | MinIO 访问密钥 |
| `MINIO_SECRET_KEY` | 否 | - | MinIO 密钥 |
| `MINIO_BUCKET` | 否 | vec-svc | MinIO 桶名 |

### 8.3 模型文件挂载

```yaml
volumes:
  - /path/to/models:/models:ro
environment:
  - EMBEDDING_MODEL_PATH=/models/gemma-4-E4B-it-Q4_0.gguf
```

### 8.4 Docker Compose

**文件位置**：[vec-svc/docker-compose.yml](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/docker-compose.yml)

包含服务：
- vec-svc（向量服务）
- milvus（向量数据库）
- etcd（Milvus 依赖）
- minio（对象存储）
- postgres（关系数据库）

---

## 9. 遇到的问题与完整解决方案

### 问题 1：MySQL 语法用于 PostgreSQL 迁移

**现象**：迁移脚本使用 MySQL 语法（`AUTO_INCREMENT`、`ENGINE=InnoDB`）

**解决方案**：改为 PostgreSQL 风格（`BIGSERIAL`，移除引擎声明）

### 问题 2：外键约束导致共享代码无法插入

**现象**：`container_config_id=0` 表示共享代码，但外键约束要求必须存在

**解决方案**：移除 `joinable!` 宏，允许特殊值

### 问题 3：NUMERIC 类型与 f64 不匹配

**现象**：`NUMERIC(5,2)` 与 Rust `f64` 反序列化失败

**解决方案**：改为 `DOUBLE PRECISION`

### 问题 4：LLM 返回 JSON 带 markdown 标记

**现象**：LLM 返回的 JSON 被包裹在 ```json ... ``` 代码块中

**解决方案**：添加 trim 处理移除 markdown 标记

### 问题 5：CUDA 编译失败（Tesla P40）

**现象**：Tesla P40（Pascal 架构）不支持 FP16 atomicAdd

**解决方案**：移除 `candle-core`，改用纯查表方式做 embedding

### 问题 6：Windows 下 diesel 依赖编译失败

**现象**：`openssl-sys` 和 `pq-sys` 在 Windows 下编译失败

**解决方案**：采用 Docker/Linux 环境构建

### 问题 7：MinIO 依赖版本错误

**现象**：crates.io 无 minio 0.13 版本

**解决方案**：更新为 0.4 版本，适配新 API

### 问题 8：pq-sys 版本冲突

**现象**：多个 crate 依赖不同版本的 pq-sys，链接冲突

**解决方案**：在 workspace 级别统一 pq-sys 版本为 0.6

### 问题 9：Rust 编译器版本过低

**现象**：crc-fast@1.10.0 要求 rustc 1.89

**解决方案**：Dockerfile.base 中升级 Rust 镜像至 1.89-slim

### 问题 10：diesel derive 宏缺失

**现象**：`Queryable`、`Selectable` 等 derive 宏找不到

**解决方案**：添加 diesel 的 "derive" 特性

---

## 10. 经验教训与最佳实践

### 10.1 Rust 开发最佳实践

1. **先编译通过再说**：先让代码编译通过，再优化细节
2. **错误类型要完整**：自定义 Error 类型要实现常用的 `From` trait
3. **注意所有权**：使用值之前想想有没有被 move
4. **模块结构要清晰**：`mod` 声明和文件位置要对应
5. **依赖要谨慎**：涉及 C FFI 的 crate 在 Windows 下可能编译困难
6. **Workspace 依赖统一**：在 workspace 级别统一依赖版本，避免冲突

### 10.2 Docker 构建最佳实践

1. **多阶段构建**：builder 阶段编译，runtime 阶段只放必要的东西
2. **基础镜像缓存**：依赖不变的话，基础镜像不用重建
3. **模型外部挂载**：大文件不要打进镜像
4. **构建参数化**：GPU 计算能力等通过 build arg 传入
5. **运行时最小化**：用 slim/alpine 镜像，减小攻击面
6. **Rust 版本及时更新**：定期升级基础镜像中的 Rust 版本

### 10.3 数据库最佳实践

1. **确认数据库类型**：写迁移脚本前确认是 PostgreSQL 还是 MySQL
2. **浮点用 DOUBLE PRECISION**：PostgreSQL 中浮点字段不要用 NUMERIC
3. **特殊值考虑**：外键约束要考虑 0、null 等特殊值
4. **迁移要可逆**：最好写 up 和 down 两个方向
5. **diesel schema.rs 要及时更新**：添加新表后要更新 schema.rs

### 10.4 RAG 系统设计最佳实践

1. **Embedding 查表足够快**：对于检索任务，静态词向量 + 平均 pooling 够用
2. **项目级配置**：不同项目可能需要不同的 RAG 参数
3. **知识边界很重要**：企业级应用必须考虑权限和可见性
4. **不只是检索**：知识蒸馏、幻觉校验、分类分级才能产生真正价值
5. **异步处理**：耗时操作（文档处理、重新索引）要异步化
6. **多级缓存**：搜索是高频操作，缓存能显著提升体验
7. **重排序**：向量检索后加一层重排序能提升结果质量

---

## 11. 当前状态与下一步计划

### 11.1 当前状态

✅ **全部完成（M1-M5，35/35任务）**：

**M1 - 核心框架**：
- 项目架构设计、API 骨架、Embedding 查表、Milvus 客户端、Docker 构建

**M2 - 基础功能**：
- 文档处理流水线、PostgreSQL 集成、项目 RAG 配置

**M3 - 核心差异化能力**：
- 知识蒸馏、分类分级（5维分类+5级评分）、知识边界（5层可见性+共享）

**M4 - 高级功能**：
- MinIO 对象存储、异步任务队列、知识图谱（实体+关系）、事实支撑校验、图谱一致性校验、搜索建议与自动补全

**M5 - 生产级就绪**：
- 性能优化（多级缓存）、重排序（BM25+向量融合）、语义分块、版本管理与差异对比、访问统计与热度分析、批量导入导出

⚠️ **占位实现**：
- Tokenizer（当前是字节级分词，需要接入 shimmytok）

❌ **未实现**：
- 前端界面（零实现）

### 11.2 下一步计划：前端开发

#### FM1 - 前端项目骨架（第 1-3 天）
1. 创建 React + TypeScript + Ant Design 项目
2. 配置路由、API 层、状态管理（React Query）
3. 完成基础布局和登录页面
4. 配置开发环境和代理

#### FM2 - 文档管理与搜索核心页面（第 4-7 天）
1. 文档列表页面（上传、删除、筛选）
2. 文档详情页面
3. 搜索页面（搜索框、结果列表、高亮）
4. 搜索建议与自动补全

#### FM3 - 知识图谱与高级功能页面（第 8-12 天）
1. 知识图谱可视化页面
2. 知识蒸馏页面
3. 分类分级管理页面
4. 知识边界设置页面

#### FM4 - 统计分析与管理页面（第 13-15 天）
1. 统计分析看板
2. 版本管理页面
3. 导入导出功能
4. 任务管理页面

#### FM5 - 生产优化（第 16-18 天）
1. 响应式设计
2. 性能优化（懒加载、代码分割）
3. 国际化支持
4. 端到端测试

---

## 12. 关键文件索引

### 核心代码
| 文件 | 说明 |
|------|------|
| [main.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/main.rs) | 服务入口 |
| [routes.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/routes.rs) | 路由定义（50+接口） |
| [app_state.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/app_state.rs) | 应用状态与业务逻辑编排 |
| [embedding.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/embedding.rs) | GGUF 嵌入查表服务 |
| [milvus.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/milvus.rs) | Milvus 客户端 |
| [minio.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/minio.rs) | MinIO 对象存储服务 |
| [task_queue.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/task_queue.rs) | 异步任务队列 |
| [knowledge_graph.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/knowledge_graph.rs) | 知识图谱服务 |
| [verification.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/verification.rs) | 幻觉校验服务 |
| [distillation.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/distillation.rs) | 知识蒸馏服务 |
| [cache.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/cache.rs) | 多级缓存服务 |
| [rerank.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/rerank.rs) | 重排序服务 |
| [semantic_chunk.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/semantic_chunk.rs) | 语义分块服务 |
| [analytics.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/analytics.rs) | 统计分析服务 |
| [version_control.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/version_control.rs) | 版本管理服务 |
| [import_export.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/import_export.rs) | 导入导出服务 |
| [model.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/model.rs) | 数据模型 |
| [schema.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/schema.rs) | 数据库表定义（21张表） |
| [rdb_repository.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/rdb_repository.rs) | 关系库仓储 |

### Handlers（10+模块）
| 文件 | 说明 |
|------|------|
| [handlers/search.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/handlers/search.rs) | 搜索接口 |
| [handlers/document.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/handlers/document.rs) | 文档管理接口 |
| [handlers/storage.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/handlers/storage.rs) | 对象存储接口 |
| [handlers/task.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/handlers/task.rs) | 任务队列接口 |
| [handlers/knowledge_graph.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/handlers/knowledge_graph.rs) | 知识图谱接口 |
| [handlers/verification.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/handlers/verification.rs) | 校验接口 |
| [handlers/distillation.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/handlers/distillation.rs) | 知识蒸馏接口 |
| [handlers/taxonomy.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/handlers/taxonomy.rs) | 分类分级接口 |
| [handlers/boundary.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/handlers/boundary.rs) | 知识边界接口 |
| [handlers/version.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/handlers/version.rs) | 版本管理接口 |
| [handlers/analytics.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/handlers/analytics.rs) | 统计分析接口 |
| [handlers/import_export.rs](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/src/handlers/import_export.rs) | 导入导出接口 |

### 构建与部署
| 文件 | 说明 |
|------|------|
| [Cargo.toml](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/Cargo.toml) | 依赖配置 |
| [Dockerfile](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/Dockerfile) | Docker 构建 |
| [docker-compose.yml](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/docker-compose.yml) | 编排文件 |
| [Dockerfile.base](../../Dockerfile.base) | 基础镜像构建 |

### 设计文档
| 文件 | 说明 |
|------|------|
| [docs/design.md](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/docs/design.md) | 详细设计文档 |
| [docs/application_scenarios.md](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/docs/application_scenarios.md) | 68 个应用场景 |
| [docs/TASK_PLAN.md](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/docs/TASK_PLAN.md) | 任务执行计划（35个任务） |
| [docs/MILESTONE_PLAN.md](file:///j:/llama_cpp/project/ACL3.0M/github/ACL3.0/rust-saas/crates/vec-svc/docs/MILESTONE_PLAN.md) | 里程碑计划（M1-M5） |

---

**文档结束**

> 本文档是 vec-svc 项目的完整记忆记录，包含所有设计决策、技术选型、问题解决方案和经验教训。
> 每次重大变更后应更新此文档。