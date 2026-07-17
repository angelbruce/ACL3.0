# vec-svc - 企业级 RAG 知识管理平台

## 项目定位

vec-svc 是 workspace-svc 的"知识大脑"，提供：
- AI 驱动的动态知识库
- 项目上下文智能检索
- 知识图谱构建与关联
- 幻觉校验系统
- 分类分级知识导航
- 知识边界与共享权限体系

## 技术栈

| 组件 | 选型 | 说明 |
|------|------|------|
| Tokenizer | shimmytok | 纯 Rust，无需 CUDA |
| 向量数据库 | Milvus | 向量存储与检索 |
| 关系数据库 | PostgreSQL | 元数据、权限、配置 |
| 文件存储 | MinIO | 原始文档存储 |
| Embedding | 查表方式 | 从 GGUF 权重直接查表 |

## 快速开始

### 1. 准备模型文件

```bash
mkdir models
# 将 gemma-4-E4B-it-Q4_0.gguf 放入 models 目录
```

### 2. 启动服务

```bash
cd crates/vec-svc
docker-compose up -d
```

### 3. 查看日志

```bash
docker-compose logs -f vec-svc
```

## 环境变量配置

### 启动配置（不可运行时更改）

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `EMBEDDING_MODEL_NAME` | `gemma-4-E4B-it` | 模型名称 |
| `EMBEDDING_MODEL_PATH` | `/models/gemma-4-E4B-it-Q4_0.gguf` | 模型路径 |
| `EMBEDDING_DIM` | `2560` | 向量维度 |

### 服务配置

| 变量 | 说明 |
|------|------|
| `DATABASE_URL` | PostgreSQL 连接字符串 |
| `MILVUS_HOST` | Milvus 地址 |
| `MILVUS_PORT` | Milvus 端口 |
| `MINIO_ENDPOINT` | MinIO 地址 |
| `MINIO_ACCESS_KEY` | MinIO 用户名 |
| `MINIO_SECRET_KEY` | MinIO 密码 |
| `MINIO_BUCKET` | MinIO bucket 名称 |

## API 端点

### 文档管理

- `POST /api/documents/file` - 上传文件
- `POST /api/documents/text` - 存文本
- `POST /api/documents/batch` - 批量导入
- `POST /api/documents/url` - URL 抓取

### 向量检索

- `POST /api/search` - 通用检索
- `POST /api/projects/{id}/search` - 项目内检索

### 知识图谱

- `POST /api/kg/entities` - 提取实体
- `POST /api/kg/relations` - 提取关系
- `GET /api/kg/graph/{project_id}` - 获取图谱

### 幻觉校验

- `POST /api/verify/summary` - 校验摘要

### 分类分级

- `GET /api/browse/{category_id}` - 按分类浏览
- `GET /api/browse/similar/{doc_id}` - 同类推荐

### 知识边界

- `POST /api/documents/{id}/boundary` - 设置边界
- `POST /api/documents/{id}/share` - 共享文档

## 详细设计文档

查看 [docs/design.md](docs/design.md) 了解完整设计。

## 与 workspace-svc 集成

```rust
// MCP 工具调用示例
let search_result = mcp_client.call_tool("vec_search", json!({
    "project_id": project_id,
    "query": "transformer 是什么",
    "config_override": {
        "top_k": 10
    }
})).await?;
```

## 开发状态

| 功能 | 状态 |
|------|------|
| P0: 向量检索 API | 📝 设计完成 |
| P0: 嵌入服务 | 📝 设计完成 |
| P1: 文档管理 | 📝 设计完成 |
| P2: 幻觉校验 | 📝 设计完成 |
| P2: 分类分级 | 📝 设计完成 |
| P3: 知识边界 | 📝 设计完成 |