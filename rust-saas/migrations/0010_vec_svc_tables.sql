-- vec-svc database tables

DROP TABLE IF EXISTS search_logs CASCADE;
DROP TABLE IF EXISTS access_logs CASCADE;
DROP TABLE IF EXISTS document_versions CASCADE;
DROP TABLE IF EXISTS tasks CASCADE;
DROP TABLE IF EXISTS document_shares CASCADE;
DROP TABLE IF EXISTS document_boundaries CASCADE;
DROP TABLE IF EXISTS document_level_mappings CASCADE;
DROP TABLE IF EXISTS document_category_mappings CASCADE;
DROP TABLE IF EXISTS document_levels CASCADE;
DROP TABLE IF EXISTS document_categories CASCADE;
DROP TABLE IF EXISTS verification_conflicts CASCADE;
DROP TABLE IF EXISTS knowledge_relations CASCADE;
DROP TABLE IF EXISTS knowledge_entities CASCADE;
DROP TABLE IF EXISTS knowledge_points CASCADE;
DROP TABLE IF EXISTS project_rag_configs CASCADE;
DROP TABLE IF EXISTS document_chunks CASCADE;
DROP TABLE IF EXISTS documents CASCADE;

CREATE TABLE documents (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT,
    title TEXT,
    topic TEXT,
    content TEXT,
    content_hash TEXT,
    source_type TEXT,
    source_url TEXT,
    file_path TEXT,
    file_type TEXT,
    status TEXT,
    visibility TEXT,
    boundary_level INT,
    token_count INT,
    version INT NOT NULL DEFAULT 1,
    word_count INT,
    chunk_count INT NOT NULL DEFAULT 0,
    metadata JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    indexed_at TIMESTAMP
);

CREATE INDEX idx_documents_project_id ON documents(project_id);
CREATE INDEX idx_documents_status ON documents(status);
CREATE INDEX idx_documents_visibility ON documents(visibility);

CREATE TABLE document_chunks (
    id BIGSERIAL PRIMARY KEY,
    document_id BIGINT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    chunk_index INT NOT NULL,
    chunk_text TEXT,
    embedding_status TEXT DEFAULT 'pending',
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_document_chunks_document_id ON document_chunks(document_id);

CREATE TABLE project_rag_configs (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT UNIQUE,
    chunk_size INT NOT NULL DEFAULT 512,
    chunk_overlap INT NOT NULL DEFAULT 50,
    chunk_strategy TEXT DEFAULT 'semantic',
    min_chunk_size INT NOT NULL DEFAULT 100,
    top_k INT NOT NULL DEFAULT 5,
    min_score DOUBLE PRECISION NOT NULL DEFAULT 0.3,
    rerank BOOLEAN NOT NULL DEFAULT false,
    rerank_top_k INT NOT NULL DEFAULT 3,
    search_type TEXT DEFAULT 'similarity',
    temperature DOUBLE PRECISION NOT NULL DEFAULT 0.7,
    max_tokens INT NOT NULL DEFAULT 2048,
    context_window INT NOT NULL DEFAULT 4096,
    batch_size INT NOT NULL DEFAULT 32,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_project_rag_configs_project_id ON project_rag_configs(project_id);

CREATE TABLE knowledge_points (
    id BIGSERIAL PRIMARY KEY,
    document_id BIGINT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    point_type TEXT,
    point_content TEXT,
    confidence DOUBLE PRECISION,
    keywords JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_knowledge_points_document_id ON knowledge_points(document_id);

CREATE TABLE knowledge_entities (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT,
    name TEXT,
    entity_type TEXT,
    description TEXT,
    aliases JSONB,
    confidence DOUBLE PRECISION,
    source_document_id BIGINT REFERENCES documents(id) ON DELETE SET NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_knowledge_entities_project_id ON knowledge_entities(project_id);
CREATE INDEX idx_knowledge_entities_name ON knowledge_entities(name);

CREATE TABLE knowledge_relations (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT,
    source_entity_id BIGINT NOT NULL REFERENCES knowledge_entities(id) ON DELETE CASCADE,
    target_entity_id BIGINT NOT NULL REFERENCES knowledge_entities(id) ON DELETE CASCADE,
    relation_type TEXT,
    relation_strength DOUBLE PRECISION,
    evidence_text TEXT,
    source_document_id BIGINT REFERENCES documents(id) ON DELETE SET NULL,
    confidence DOUBLE PRECISION,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_knowledge_relations_project_id ON knowledge_relations(project_id);
CREATE INDEX idx_knowledge_relations_source_entity_id ON knowledge_relations(source_entity_id);
CREATE INDEX idx_knowledge_relations_target_entity_id ON knowledge_relations(target_entity_id);

CREATE TABLE verification_conflicts (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT,
    query_text TEXT,
    llm_summary TEXT,
    conflict_type TEXT,
    conflict_description TEXT,
    confidence_score DOUBLE PRECISION,
    resolved BOOLEAN NOT NULL DEFAULT false,
    resolution TEXT,
    resolved_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_verification_conflicts_project_id ON verification_conflicts(project_id);

CREATE TABLE document_categories (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT,
    category_name TEXT,
    category_type TEXT,
    parent_id BIGINT REFERENCES document_categories(id) ON DELETE CASCADE,
    level INT NOT NULL DEFAULT 1,
    description TEXT,
    icon TEXT,
    color TEXT,
    sort_order INT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_document_categories_project_id ON document_categories(project_id);
CREATE INDEX idx_document_categories_type ON document_categories(category_type);
CREATE INDEX idx_document_categories_parent_id ON document_categories(parent_id);

CREATE TABLE document_levels (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT,
    level_name TEXT,
    level_type TEXT,
    level_value INT,
    description TEXT,
    icon TEXT,
    color TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_document_levels_project_id ON document_levels(project_id);
CREATE INDEX idx_document_levels_type ON document_levels(level_type);

CREATE TABLE document_category_mappings (
    id BIGSERIAL PRIMARY KEY,
    document_id BIGINT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    category_id BIGINT NOT NULL REFERENCES document_categories(id) ON DELETE CASCADE,
    confidence DOUBLE PRECISION,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_doc_cat_mappings_document_id ON document_category_mappings(document_id);
CREATE INDEX idx_doc_cat_mappings_category_id ON document_category_mappings(category_id);

CREATE TABLE document_level_mappings (
    id BIGSERIAL PRIMARY KEY,
    document_id BIGINT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    level_id BIGINT NOT NULL REFERENCES document_levels(id) ON DELETE CASCADE,
    confidence DOUBLE PRECISION,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_doc_level_mappings_document_id ON document_level_mappings(document_id);
CREATE INDEX idx_doc_level_mappings_level_id ON document_level_mappings(level_id);

CREATE TABLE document_boundaries (
    id BIGSERIAL PRIMARY KEY,
    document_id BIGINT UNIQUE NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    boundary_type TEXT,
    owner_id BIGINT,
    project_id BIGINT,
    team_id BIGINT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_document_boundaries_document_id ON document_boundaries(document_id);
CREATE INDEX idx_document_boundaries_owner_id ON document_boundaries(owner_id);
CREATE INDEX idx_document_boundaries_project_id ON document_boundaries(project_id);

CREATE TABLE document_shares (
    id BIGSERIAL PRIMARY KEY,
    document_id BIGINT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    share_type TEXT,
    target_type TEXT,
    target_id BIGINT,
    granted_by BIGINT,
    expire_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_document_shares_document_id ON document_shares(document_id);
CREATE INDEX idx_document_shares_target ON document_shares(target_type, target_id);

CREATE TABLE tasks (
    id BIGSERIAL PRIMARY KEY,
    task_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    payload JSONB NOT NULL DEFAULT '{}',
    progress REAL NOT NULL DEFAULT 0.0,
    message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    started_at TIMESTAMP,
    completed_at TIMESTAMP
);

CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_created_at ON tasks(created_at);

CREATE TABLE document_versions (
    id BIGSERIAL PRIMARY KEY,
    document_id BIGINT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    version_number INT NOT NULL,
    content TEXT NOT NULL,
    title TEXT,
    change_note TEXT,
    created_by BIGINT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_document_versions_document_id ON document_versions(document_id);

CREATE TABLE access_logs (
    id BIGSERIAL PRIMARY KEY,
    document_id BIGINT REFERENCES documents(id) ON DELETE CASCADE,
    user_id BIGINT,
    access_type VARCHAR(50),
    ip_address VARCHAR(50),
    user_agent TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_access_logs_document_id ON access_logs(document_id);
CREATE INDEX idx_access_logs_user_id ON access_logs(user_id);
CREATE INDEX idx_access_logs_created_at ON access_logs(created_at);

CREATE TABLE search_logs (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT,
    project_id BIGINT,
    query_text TEXT NOT NULL,
    result_count INT NOT NULL DEFAULT 0,
    response_time_ms INT,
    ip_address VARCHAR(50),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_search_logs_user_id ON search_logs(user_id);
CREATE INDEX idx_search_logs_project_id ON search_logs(project_id);
CREATE INDEX idx_search_logs_created_at ON search_logs(created_at);
