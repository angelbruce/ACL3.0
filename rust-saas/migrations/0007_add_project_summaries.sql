-- Create project_summaries table for storing article summaries
CREATE TABLE project_summaries (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    project_id BIGINT NOT NULL,
    file_name TEXT NOT NULL,
    summary TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create index for faster queries
CREATE INDEX idx_project_summaries_project_id ON project_summaries(project_id);
CREATE INDEX idx_project_summaries_user_id ON project_summaries(user_id);
CREATE INDEX idx_project_summaries_file_name ON project_summaries(file_name);
