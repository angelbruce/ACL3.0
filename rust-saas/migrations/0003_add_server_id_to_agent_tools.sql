-- Add server_id column to agent_tools table
ALTER TABLE agent_tools ADD COLUMN server_id INT8;
ALTER TABLE agent_tools ADD CONSTRAINT fk_agent_tools_server_id FOREIGN KEY (server_id) REFERENCES mcp_servers(id);