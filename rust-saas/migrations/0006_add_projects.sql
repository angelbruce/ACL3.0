-- Create projects table
CREATE TABLE IF NOT EXISTS projects (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    name TEXT NOT NULL,
    purpose TEXT NOT NULL,
    description TEXT,
    model_id BIGINT,
    agent_id BIGINT,
    last_accessed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create project_files table
CREATE TABLE IF NOT EXISTS project_files (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    content TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create project_messages table
CREATE TABLE IF NOT EXISTS project_messages (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_projects_user_id ON projects(user_id);
CREATE INDEX IF NOT EXISTS idx_projects_last_accessed_at ON projects(last_accessed_at DESC);
CREATE INDEX IF NOT EXISTS idx_project_files_project_id ON project_files(project_id);
CREATE INDEX IF NOT EXISTS idx_project_files_created_at ON project_files(created_at ASC);
CREATE INDEX IF NOT EXISTS idx_project_messages_project_id ON project_messages(project_id);
CREATE INDEX IF NOT EXISTS idx_project_messages_created_at ON project_messages(created_at ASC);

-- Add foreign key constraints
ALTER TABLE projects ADD CONSTRAINT fk_projects_model_id FOREIGN KEY (model_id) REFERENCES llm_models(id) ON DELETE SET NULL;
ALTER TABLE projects ADD CONSTRAINT fk_projects_agent_id FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE SET NULL;
