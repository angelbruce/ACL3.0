-- Create mcp_servers table for external MCP tool servers
CREATE TABLE mcp_servers (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    server_type VARCHAR(50) NOT NULL DEFAULT 'sse',
    url VARCHAR(512) NOT NULL,
    headers JSONB,
    enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create index for enabled servers lookup
CREATE INDEX idx_mcp_servers_enabled ON mcp_servers(enabled);
