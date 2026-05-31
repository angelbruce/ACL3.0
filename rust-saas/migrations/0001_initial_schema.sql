-- Create agents table
CREATE TABLE agents (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    defination TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create agent_tools table
CREATE TABLE agent_tools (
    id BIGSERIAL PRIMARY KEY,
    agent_id BIGINT NOT NULL REFERENCES agents(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    input_schema TEXT,
    output_schema TEXT
);

-- Create agent_skills table
CREATE TABLE agent_skills (
    id BIGSERIAL PRIMARY KEY,
    agent_id BIGINT NOT NULL REFERENCES agents(id),
    skill_prompt TEXT
);

-- Create content_store_configs table
CREATE TABLE content_store_configs (
    id BIGSERIAL PRIMARY KEY,
    agent_id BIGINT NOT NULL REFERENCES agents(id),
    store_type VARCHAR(255),
    config TEXT
);

-- Create flows table
CREATE TABLE flows (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    config JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create flow_runtimes table
CREATE TABLE flow_runtimes (
    id BIGSERIAL PRIMARY KEY,
    flow_id BIGINT NOT NULL REFERENCES flows(id),
    is_over BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create flow_runtime_nodes table
CREATE TABLE flow_runtime_nodes (
    id BIGSERIAL PRIMARY KEY,
    flow_runtime_id BIGINT NOT NULL REFERENCES flow_runtimes(id),
    flow_id BIGINT NOT NULL REFERENCES flows(id),
    action_id BIGINT,
    action TEXT,
    prompt TEXT,
    status VARCHAR(50) DEFAULT 'Running',
    next_choice TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create sessions table
CREATE TABLE sessions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create session_items table
CREATE TABLE session_items (
    id BIGSERIAL PRIMARY KEY,
    session_id BIGINT NOT NULL REFERENCES sessions(id),
    description TEXT NOT NULL,
    session_type VARCHAR(50) DEFAULT 'User',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create llm_models table
CREATE TABLE llm_models (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    access_url VARCHAR(512) NOT NULL,
    api_key VARCHAR(512) NOT NULL,
    is_default BOOLEAN DEFAULT FALSE
);

-- Create users table
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_agent_tools_agent_id ON agent_tools(agent_id);
CREATE INDEX idx_agent_skills_agent_id ON agent_skills(agent_id);
CREATE INDEX idx_content_store_configs_agent_id ON content_store_configs(agent_id);
CREATE INDEX idx_flow_runtimes_flow_id ON flow_runtimes(flow_id);
CREATE INDEX idx_flow_runtime_nodes_flow_runtime_id ON flow_runtime_nodes(flow_runtime_id);
CREATE INDEX idx_session_items_session_id ON session_items(session_id);
CREATE INDEX idx_users_email ON users(email);