-- 创建项目文件容器分配表
-- 用于存储文件与容器的归属关系
-- 一个文件可以从属于多个容器，container_config_id=0表示共有代码

CREATE TABLE project_file_container_assignments (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    file_id BIGINT NOT NULL REFERENCES project_files(id) ON DELETE CASCADE,
    container_config_id BIGINT NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    assigned_by VARCHAR(20) DEFAULT 'llm',
    confidence_score DOUBLE PRECISION DEFAULT 0.0,
    assignment_reason TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT unique_file_container UNIQUE (file_id, container_config_id)
);

-- 创建索引
CREATE INDEX idx_project_file_container_assignments_project_id ON project_file_container_assignments(project_id);
CREATE INDEX idx_project_file_container_assignments_file_id ON project_file_container_assignments(file_id);
CREATE INDEX idx_project_file_container_assignments_container_config_id ON project_file_container_assignments(container_config_id);
CREATE INDEX idx_project_file_container_assignments_file_path ON project_file_container_assignments(file_path);
