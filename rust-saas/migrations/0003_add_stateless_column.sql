-- Add stateless column to mcp_servers table
ALTER TABLE mcp_servers ADD COLUMN stateless BOOLEAN DEFAULT FALSE;
