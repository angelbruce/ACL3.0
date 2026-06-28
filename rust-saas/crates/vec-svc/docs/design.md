# vec-svc 设计文档

## 1. 项目定位

**vec-svc** 是一个企业级 RAG（检索增强生成）知识管理平台，定位为 workspace-svc 的"知识大脑"。

核心价值：
- AI 驱动的动态知识库（主动获取知识）
- 项目上下文智能检索
- 知识图谱构建与关联
- 幻觉校验系统
- 分类分级知识导航
- 知识边界与共享权限体系

---

## 2. 技术架构

### 2.1 技术栈

| 组件 | 选型 | 用途 |
|------|------|------|
| **Tokenizer** | shimmytok 0.7.1 | 纯 Rust GGUF tokenizer |
| **向量数据库** | Milvus | 向量存储与检索 |
| **关系数据库** | PostgreSQL | 元数据、权限、配置 |
| **文件存储** | MinIO | 原始文档存储 |
| **Embedding** | 查表方式 | 直接从 GGUF 权重查表（无需 CUDA） |

### 2.2 不使用的依赖

- ❌ `candle-core` — 不需要 CUDA，纯 CPU 即可
- ❌ `gguf-llms` — 只用 shimmytok 即可
- ❌ CUDA toolkit — 完全不需要 GPU

### 2.3 Docker 部署

```yaml
services:
  vec-svc:
    image: acl-vec-svc
    ports:
      - "8088:8080"
    volumes:
      - ${MODEL_PATH}:/models:ro  # 模型文件挂载
      - minio_data:/data
    environment:
      - EMBEDDING_MODEL_PATH=/models/gemma-4-E4B-it-Q4_0.gguf
      - EMBEDDING_DIM=2560
      - MILVUS_HOST=milvus
      - MINIO_ENDPOINT=minio:9000
    depends_on:
      - postgres
      - milvus
      - minio
```

---

## 3. 核心功能模块

### 3.1 文档管理

| API | 用途 | 输入 |
|-----|------|------|
| `POST /api/documents/file` | 上传文件 | multipart/form-data |
| `POST /api/documents/text` | 直接存文本 | `{ content, topic, metadata }` |
| `POST /api/documents/batch` | 批量导入 | `[{ content, topic, source_url }]` |
| `POST /api/documents/url` | URL 抓取 | `{ url, topic, project_id }` |
| `PUT /api/documents/{id}` | 更新文档 | `{ content, metadata }` |
| `DELETE /api/documents/{id}` | 删除文档 | - |
| `POST /api/documents/{id}/reindex` | 重新索引 | - |

### 3.2 向量检索

| API | 用途 |
|-----|------|
| `POST /api/search` | 通用检索 |
| `POST /api/projects/{id}/search` | 项目内检索 |
| `GET /api/documents/{id}` | 获取文档详情 |
| `GET /api/documents` | 列表查询 |

### 3.3 嵌入服务

| API | 用途 |
|-----|------|
| `POST /api/embed` | 文本向量化（内部接口） |

### 3.4 项目级管理

| API | 用途 |
|-----|------|
| `POST /api/projects/{id}/documents` | 项目文档管理 |
| `GET /api/projects/{id}/stats` | 项目统计信息 |

---

## 4. RAG 配置参数

### 4.1 启动配置（环境变量）

```env
EMBEDDING_MODEL_NAME=gemma-4-E4B-it
EMBEDDING_MODEL_PATH=/models/gemma-4-E4B-it-Q4_0.gguf
EMBEDDING_DIM=2560
MAX_SEQUENCE_LENGTH=8192
```

### 4.2 项目配置（数据库）

```sql
CREATE TABLE project_rag_configs (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT UNIQUE,
    
    -- 分块参数
    chunk_size INT DEFAULT 512,
    chunk_overlap INT DEFAULT 50,
    chunk_strategy VARCHAR(20) DEFAULT 'semantic',
    min_chunk_size INT DEFAULT 100,
    
    -- 检索参数
    top_k INT DEFAULT 5,
    min_score FLOAT DEFAULT 0.3,
    rerank BOOLEAN DEFAULT false,
    rerank_top_k INT DEFAULT 3,
    search_type VARCHAR(20) DEFAULT 'similarity',
    
    -- 生成参数
    temperature FLOAT DEFAULT 0.7,
    max_tokens INT DEFAULT 2048,
    context_window INT DEFAULT 4096,
    
    -- 批处理参数
    batch_size INT DEFAULT 32,
    
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);
```

### 4.3 配置优先级

```
全局默认配置 < 项目配置 < API 调用时临时覆盖
```

---

## 5. 知识蒸馏

### 5.1 设计理念

不存原始文档片段，存 AI 提炼的"知识点"。

```rust
POST /api/documents/text
{
    "content": "长文档...",
    "enable_distillation": true
}

vec-svc 内部：
  1. 接收文档
  2. 调用 workspace-svc LLM 提取知识点
  3. 存储知识点（不是原文）
  4. 向量化知识点
```

### 5.2 数据模型

```sql
CREATE TABLE knowledge_points (
    id BIGSERIAL PRIMARY KEY,
    document_id BIGINT,
    point_type VARCHAR(50),     -- fact/concept/method/best_practice
    point_content TEXT,
    confidence FLOAT,
    created_at TIMESTAMPTZ
);
```

---

## 6. 幻觉校验系统

### 6.1 校验维度

| 维度 | 实现方式 |
|------|----------|
| **事实支撑校验** | 摘要中的陈述 vs 检索文档原文 |
| **知识图谱一致性** | 摘要中的关系 vs 已知知识图谱 |
| **多源一致性** | 多个文档之间的信息一致性 |

### 6.2 数据模型

```sql
-- 实体表
CREATE TABLE knowledge_entities (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT,
    entity_name VARCHAR(255),
    entity_type VARCHAR(50),
    aliases JSONB,
    confidence FLOAT,
    source_document_id BIGINT,
    created_at TIMESTAMPTZ
);

-- 关系表
CREATE TABLE knowledge_relations (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT,
    entity_from_id BIGINT,
    entity_to_id BIGINT,
    relation_type VARCHAR(50),
    relation_strength FLOAT,
    evidence_text TEXT,
    source_document_id BIGINT,
    confidence FLOAT,
    created_at TIMESTAMPTZ
);

-- 冲突记录表
CREATE TABLE verification_conflicts (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT,
    query_text TEXT,
    llm_summary TEXT,
    conflict_type VARCHAR(50),
    conflict_description TEXT,
    confidence_score FLOAT,
    resolved BOOLEAN DEFAULT false,
    resolution TEXT,
    created_at TIMESTAMPTZ
);
```

### 6.3 API

| API | 用途 |
|-----|------|
| `POST /api/verify/summary` | 校验 LLM 生成的摘要 |
| `POST /api/verify/chunks` | 校验检索文档的一致性 |
| `POST /api/kg/entities` | 提取实体 |
| `POST /api/kg/relations` | 提取关系 |
| `GET /api/kg/graph/{project_id}` | 获取知识图谱 |

---

## 7. 分类分级系统

### 7.1 分类维度

| 维度 | 示例 |
|------|------|
| **主题分类** | 技术/产品/运营/设计 |
| **内容类型** | 教程/案例/最佳实践/规范 |
| **业务域** | 用户管理/支付/搜索/推荐 |
| **技术栈** | Rust/Python/Docker/K8s |
| **受众人群** | 开发者/架构师/产品/运维 |

### 7.2 分级维度

| 维度 | 级别 |
|------|------|
| **难度等级** | 入门/中级/高级/专家 |
| **重要性** | 核心/重要/参考/补充 |
| **成熟度** | 草稿/评审/发布/废弃 |
| **时效性** | 长期有效/年度更新/月度更新 |

### 7.3 数据模型

```sql
-- 分类表
CREATE TABLE document_categories (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT,
    category_name VARCHAR(100),
    category_type VARCHAR(50),
    parent_id BIGINT,
    level INT DEFAULT 1,
    description TEXT,
    icon VARCHAR(50),
    color VARCHAR(20),
    sort_order INT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ
);

-- 分级表
CREATE TABLE document_levels (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT,
    level_name VARCHAR(100),
    level_type VARCHAR(50),
    level_value INT,
    description TEXT,
    icon VARCHAR(50),
    color VARCHAR(20),
    created_at TIMESTAMPTZ
);

-- 文档-分类关联表
CREATE TABLE document_category_mappings (
    id BIGSERIAL PRIMARY KEY,
    document_id BIGINT,
    category_id BIGINT,
    confidence FLOAT,
    is_primary BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ
);

-- 文档-分级关联表
CREATE TABLE document_level_mappings (
    id BIGSERIAL PRIMARY KEY,
    document_id BIGINT,
    level_id BIGINT,
    confidence FLOAT,
    is_primary BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ
);
```

### 7.4 漂移推荐 API

| API | 用途 |
|-----|------|
| `GET /api/browse/{category_id}` | 按分类浏览 |
| `GET /api/browse/similar/{doc_id}` | 同类文档推荐 |
| `GET /api/browse/path/{doc_id}` | 学习路径推荐 |

---

## 8. 知识边界与共享权限

### 8.1 可见性层级

| 层级 | 可见范围 |
|------|----------|
| **个人私有** | 仅创建者 |
| **项目成员** | 项目内所有成员 |
| **团队共享** | 指定团队/部门 |
| **组织公开** | 整个组织/公司 |
| **公开知识** | 所有用户 |

### 8.2 共享模式

| 模式 | 特性 |
|------|------|
| **完全共享** | 所有人可读可写 |
| **只读共享** | 仅作者可写 |
| **审批共享** | 编辑需审批 |
| **订阅共享** | 需申请访问 |
| **付费共享** | 需付费解锁 |

### 8.3 数据模型

```sql
-- 知识边界表
CREATE TABLE document_boundaries (
    id BIGSERIAL PRIMARY KEY,
    document_id BIGINT UNIQUE,
    boundary_type VARCHAR(50),
    owner_id BIGINT,
    project_id BIGINT,
    team_id BIGINT,
    created_at TIMESTAMPTZ
);

-- 知识共享表
CREATE TABLE document_shares (
    id BIGSERIAL PRIMARY KEY,
    document_id BIGINT,
    share_type VARCHAR(50),
    target_type VARCHAR(50),
    target_id BIGINT,
    granted_by BIGINT,
    expire_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ
);

-- 用户角色表
CREATE TABLE user_roles (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT,
    role_type VARCHAR(50),
    scope_type VARCHAR(50),
    scope_id BIGINT,
    granted_at TIMESTAMPTZ,
    expire_at TIMESTAMPTZ
);

-- 知识订阅表
CREATE TABLE knowledge_subscriptions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT,
    subscription_type VARCHAR(50),
    subscription_target_id BIGINT,
    notification_enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ
);

-- 知识访问日志
CREATE TABLE knowledge_access_logs (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT,
    document_id BIGINT,
    access_type VARCHAR(50),
    access_duration INT,
    accessed_at TIMESTAMPTZ
);
```

### 8.4 API

| API | 用途 |
|-----|------|
| `POST /api/documents/{id}/boundary` | 设置文档边界 |
| `POST /api/documents/{id}/share` | 共享文档 |
| `POST /api/documents/{id}/request-access` | 申请访问权限 |
| `POST /api/subscriptions` | 订阅知识源 |

---

## 9. 数据流设计

### 9.1 文档上传流程

```
用户上传文档
    ↓
vec-svc: POST /api/documents/text
    ↓
  1. MinIO: 存储原始文档（可选）
  2. PG: documents 表插入元数据
  3. 知识蒸馏: LLM 提取知识点
  4. 分类分级: AI 自动分类
  5. 知识图谱: 提取实体和关系
  6. 分块: 按配置分块
  7. 向量化: shimmytok + 查表
  8. Milvus: 存入向量
    ↓
返回 doc_id
```

### 9.2 检索流程

```
用户提问
    ↓
workspace-svc: MCP tool vec_search(project_id, query)
    ↓
vec-svc: POST /api/projects/{id}/search
    ↓
  1. 获取项目 RAG 配置
  2. query → shimmytok → embedding
  3. Milvus: 搜索 top_k（边界过滤）
  4. 知识图谱: 关联实体
  5. 返回: [{chunk_id, topic, content, score}]
    ↓
workspace-svc: LLM summarize
    ↓
vec-svc: 幻觉校验
    ↓
返回答案 + 置信度
```

---

## 10. Rust 核心结构

### 10.1 Embedding 服务

```rust
pub struct EmbeddingConfig {
    pub model_name: String,
    pub model_path: String,
    pub embedding_dim: usize,
    pub max_sequence_length: usize,
}

pub struct EmbeddingService {
    config: EmbeddingConfig,
    tokenizer: Tokenizer,
    embedding_weights: Vec<f32>,
}

impl EmbeddingService {
    pub fn embed(&self, text: &str) -> Result<Vec<f32>>;
}
```

### 10.2 RAG 配置

```rust
pub struct RagConfig {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub chunk_strategy: ChunkStrategy,
    pub top_k: usize,
    pub min_score: f32,
    pub temperature: f32,
    pub max_tokens: usize,
    pub context_window: usize,
}

pub enum ChunkStrategy {
    Fixed,
    Semantic,
    Paragraph,
}
```

### 10.3 知识边界校验

```rust
pub struct KnowledgeBoundaryChecker {
    user_roles: HashMap<i64, UserRole>,
    document_boundaries: HashMap<i64, DocumentBoundary>,
}

impl KnowledgeBoundaryChecker {
    pub fn can_access(&self, user_id: i64, document_id: i64, access_type: AccessType) -> bool;
}
```

---

## 11. 与 workspace-svc 集成

### 11.1 MCP 工具注册

```rust
// workspace-svc 注册 vec-svc 的 MCP 工具
tools.register("vec_search", vec_svc_search_handler);
tools.register("vec_add_document", vec_svc_add_document_handler);
tools.register("vec_classify", vec_svc_classify_handler);
```

### 11.2 调用示例

```rust
// workspace-svc 调用 vec-svc
let search_result = mcp_client.call_tool("vec_search", json!({
    "project_id": project_id,
    "query": "transformer 是什么",
    "config_override": {
        "top_k": 10
    }
})).await?;
```

---

## 12. 优先级规划

| 优先级 | 功能 | 依赖 |
|--------|------|------|
| P0 | 向量检索 API | shimmytok + Milvus |
| P0 | 嵌入服务 | GGUF 权重加载 |
| P1 | 文档上传 + 向量化 | MinIO + PG |
| P1 | 项目级管理 | PG |
| P2 | 幻觉校验 | 知识图谱 |
| P2 | 分类分级 | AI 自动分类 |
| P3 | 知识边界 | 权限系统 |

---

## 13. 部署说明

### 13.1 模型文件准备

```bash
# 宿主机模型目录
/path/to/models/
  ├── gemma-4-E4B-it-Q4_0.gguf  (约 4-5GB)
```

### 13.2 启动命令

```bash
MODEL_PATH=/path/to/models docker-compose up
```

### 13.3 环境变量

```env
EMBEDDING_MODEL_NAME=gemma-4-E4B-it
EMBEDDING_MODEL_PATH=/models/gemma-4-E4B-it-Q4_0.gguf
EMBEDDING_DIM=2560
MILVUS_HOST=milvus
MILVUS_PORT=19530
MINIO_ENDPOINT=minio:9000
DATABASE_URL=postgres://user:pass@postgres:5432/acl
```