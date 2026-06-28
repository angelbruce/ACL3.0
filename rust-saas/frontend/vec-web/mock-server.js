import express from 'express';
import cors from 'cors';

const app = express();
app.use(cors());
app.use(express.json());

const mockDocuments = [
  {
    id: '1',
    title: '企业知识管理系统架构设计',
    content: '本文档详细描述了企业知识管理系统的整体架构设计，包括前端展示层、业务逻辑层、数据存储层等核心模块...',
    document_type: 'text',
    status: 'completed',
    visibility: 'public',
    version: 1,
    word_count: 2500,
    chunk_count: 15,
    created_at: '2024-01-15T10:30:00Z',
    updated_at: '2024-01-15T10:30:00Z',
  },
  {
    id: '2',
    title: 'RAG技术白皮书',
    content: '检索增强生成(RAG)是一种结合信息检索和生成模型的AI技术，能够显著提升LLM的准确性和时效性...',
    document_type: 'text',
    status: 'completed',
    visibility: 'public',
    version: 2,
    word_count: 5000,
    chunk_count: 30,
    created_at: '2024-01-20T14:20:00Z',
    updated_at: '2024-01-22T09:15:00Z',
  },
  {
    id: '3',
    title: 'Milvus向量数据库入门指南',
    content: 'Milvus是一个开源的向量数据库，专为大规模向量相似性搜索设计，支持多种索引类型...',
    document_type: 'text',
    status: 'processing',
    visibility: 'private',
    version: 1,
    word_count: 3200,
    chunk_count: 20,
    created_at: '2024-01-25T16:45:00Z',
    updated_at: '2024-01-25T16:45:00Z',
  },
  {
    id: '4',
    title: '知识图谱构建实践',
    content: '知识图谱是一种结构化的语义网络，用于表示实体之间的关系...',
    document_type: 'text',
    status: 'completed',
    visibility: 'public',
    version: 1,
    word_count: 4100,
    chunk_count: 25,
    created_at: '2024-01-28T11:00:00Z',
    updated_at: '2024-01-28T11:00:00Z',
  },
  {
    id: '5',
    title: '向量嵌入技术详解',
    content: '向量嵌入是将文本、图像等非结构化数据转换为高维向量的过程...',
    document_type: 'text',
    status: 'completed',
    visibility: 'restricted',
    version: 3,
    word_count: 6500,
    chunk_count: 40,
    created_at: '2024-02-01T09:30:00Z',
    updated_at: '2024-02-05T14:00:00Z',
  },
];

const mockKnowledgePoints = [
  { id: 'kp-1', document_id: '1', content: '企业知识管理系统采用分层架构设计', confidence: 0.92, keywords: ['架构', '分层', '设计'], created_at: '2024-01-15T11:00:00Z' },
  { id: 'kp-2', document_id: '1', content: '前端展示层负责用户界面交互', confidence: 0.88, keywords: ['前端', '展示', 'UI'], created_at: '2024-01-15T11:00:00Z' },
  { id: 'kp-3', document_id: '2', content: 'RAG能够提升LLM的事实准确性', confidence: 0.95, keywords: ['RAG', 'LLM', '准确性'], created_at: '2024-01-22T09:30:00Z' },
  { id: 'kp-4', document_id: '2', content: '检索和生成是RAG的两个核心环节', confidence: 0.91, keywords: ['检索', '生成', '核心'], created_at: '2024-01-22T09:30:00Z' },
];

const mockCategories = [
  { id: 'cat-1', name: '技术文档', document_count: 3, children: [
    { id: 'cat-1-1', name: '架构设计', document_count: 1 },
    { id: 'cat-1-2', name: '数据库', document_count: 1 },
    { id: 'cat-1-3', name: 'AI技术', document_count: 1 },
  ]},
  { id: 'cat-2', name: '产品文档', document_count: 2, children: [] },
];

const mockEntities = [
  { id: 'ent-1', name: 'RAG', entity_type: '技术概念', description: '检索增强生成' },
  { id: 'ent-2', name: 'Milvus', entity_type: '数据库', description: '向量数据库' },
  { id: 'ent-3', name: '知识图谱', entity_type: '技术概念', description: '结构化语义网络' },
];

const mockTasks = [
  { id: 'task-1', task_type: 'distillation', document_id: '1', status: 'completed', progress: 100, created_at: '2024-01-15T11:00:00Z', updated_at: '2024-01-15T11:30:00Z' },
  { id: 'task-2', task_type: 'embedding', document_id: '3', status: 'running', progress: 65, created_at: '2024-01-25T16:45:00Z', updated_at: '2024-01-25T17:00:00Z' },
  { id: 'task-3', task_type: 'distillation', document_id: '2', status: 'completed', progress: 100, created_at: '2024-01-22T09:30:00Z', updated_at: '2024-01-22T10:00:00Z' },
];

const mockVersions = [
  { id: 'v-1', document_id: '2', version_number: 1, title: 'RAG技术白皮书', status: 'completed', change_summary: '初始版本', created_at: '2024-01-20T14:20:00Z' },
  { id: 'v-2', document_id: '2', version_number: 2, title: 'RAG技术白皮书', status: 'completed', change_summary: '更新检索策略章节', created_at: '2024-01-22T09:15:00Z' },
];

const mockShares = [
  { id: 'share-1', document_id: '5', user_id: 'user-1', permission: 'read' },
  { id: 'share-2', document_id: '5', user_id: 'user-2', permission: 'write' },
];

app.get('/api/documents', (req, res) => {
  const page = parseInt(req.query.page) || 1;
  const page_size = parseInt(req.query.page_size) || 10;
  const start = (page - 1) * page_size;
  const end = start + page_size;
  res.json({
    success: true,
    data: {
      data: mockDocuments.slice(start, end),
      total: mockDocuments.length,
      page,
      page_size,
    },
  });
});

app.get('/api/documents/:id', (req, res) => {
  const doc = mockDocuments.find(d => d.id === req.params.id);
  if (doc) {
    res.json({ success: true, data: doc });
  } else {
    res.status(404).json({ success: false, message: 'Document not found' });
  }
});

app.post('/api/documents', (req, res) => {
  const newDoc = {
    id: String(mockDocuments.length + 1),
    ...req.body,
    document_type: req.body.document_type || 'text',
    status: 'completed',
    version: 1,
    word_count: req.body.content?.length || 0,
    chunk_count: Math.ceil((req.body.content?.length || 0) / 500),
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  mockDocuments.unshift(newDoc);
  res.json({ success: true, data: newDoc });
});

app.put('/api/documents/:id', (req, res) => {
  const index = mockDocuments.findIndex(d => d.id === req.params.id);
  if (index !== -1) {
    mockDocuments[index] = { ...mockDocuments[index], ...req.body, updated_at: new Date().toISOString() };
    res.json({ success: true, data: mockDocuments[index] });
  } else {
    res.status(404).json({ success: false, message: 'Document not found' });
  }
});

app.delete('/api/documents/:id', (req, res) => {
  const index = mockDocuments.findIndex(d => d.id === req.params.id);
  if (index !== -1) {
    mockDocuments.splice(index, 1);
    res.json({ success: true });
  } else {
    res.status(404).json({ success: false, message: 'Document not found' });
  }
});

app.get('/api/documents/:id/knowledge-points', (req, res) => {
  const points = mockKnowledgePoints.filter(kp => kp.document_id === req.params.id);
  res.json({ success: true, data: points });
});

app.post('/api/documents/:id/distill', (req, res) => {
  setTimeout(() => {
    res.json({ success: true, message: '蒸馏任务已提交' });
  }, 500);
});

app.get('/api/search', (req, res) => {
  const query = req.query.query || '';
  const results = mockDocuments
    .filter(d => d.title.toLowerCase().includes(query.toLowerCase()) || d.content.toLowerCase().includes(query.toLowerCase()))
    .map(d => ({
      id: d.id,
      document_id: d.id,
      document_title: d.title,
      content: d.content.substring(0, 200) + '...',
      highlighted_content: d.content.substring(0, 200) + '...',
      score: Math.random() * 0.5 + 0.5,
    }));
  res.json({ success: true, data: results });
});

app.get('/api/taxonomy/categories', (req, res) => {
  res.json({ success: true, data: mockCategories });
});

app.post('/api/taxonomy/categories', (req, res) => {
  const newCat = {
    id: 'cat-' + Date.now(),
    name: req.body.name,
    document_count: 0,
    children: [],
  };
  mockCategories.push(newCat);
  res.json({ success: true, data: newCat });
});

app.put('/api/taxonomy/categories/:id', (req, res) => {
  const cat = mockCategories.find(c => c.id === req.params.id);
  if (cat) {
    cat.name = req.body.name;
    res.json({ success: true, data: cat });
  } else {
    res.status(404).json({ success: false });
  }
});

app.delete('/api/taxonomy/categories/:id', (req, res) => {
  const index = mockCategories.findIndex(c => c.id === req.params.id);
  if (index !== -1) {
    mockCategories.splice(index, 1);
    res.json({ success: true });
  } else {
    res.status(404).json({ success: false });
  }
});

app.get('/api/knowledge-graph', (req, res) => {
  res.json({ success: true, data: { entities: mockEntities } });
});

app.get('/api/boundary/shares', (req, res) => {
  const shares = mockShares.filter(s => s.document_id === req.query.document_id);
  res.json({ success: true, data: shares });
});

app.put('/api/boundary/visibility', (req, res) => {
  const doc = mockDocuments.find(d => d.id === req.body.document_id);
  if (doc) {
    doc.visibility = req.body.visibility;
    res.json({ success: true });
  } else {
    res.status(404).json({ success: false });
  }
});

app.delete('/api/boundary/shares/:id', (req, res) => {
  const index = mockShares.findIndex(s => s.id === req.params.id);
  if (index !== -1) {
    mockShares.splice(index, 1);
    res.json({ success: true });
  } else {
    res.status(404).json({ success: false });
  }
});

app.get('/api/version/documents/:id', (req, res) => {
  const versions = mockVersions.filter(v => v.document_id === req.params.id);
  res.json({ success: true, data: versions });
});

app.post('/api/version/rollback', (req, res) => {
  res.json({ success: true, message: '回滚任务已提交' });
});

app.get('/api/tasks', (req, res) => {
  res.json({ success: true, data: mockTasks });
});

app.get('/api/analytics/summary', (req, res) => {
  res.json({
    success: true,
    data: {
      total_documents: mockDocuments.length,
      total_search_queries: 1250,
      total_entities: mockEntities.length,
      total_access_count: 3420,
    },
  });
});

const PORT = 8080;
app.listen(PORT, () => {
  console.log(`Mock server running on http://localhost:${PORT}`);
});
