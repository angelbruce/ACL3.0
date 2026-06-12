ALTER TABLE project_files
    ADD COLUMN state integer DEFAULT 0;

UPDATE project_files
    SET state = 0 WHERE state IS NULL;


ALTER TABLE project_files     ADD COLUMN directory text DEFAULT '';

CREATE TABLE project_container_configs (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT REFERENCES projects(id),
    project_dir TEXT DEFAULT '',
    published_ports TEXT DEFAULT '',
    volumes TEXT DEFAULT '',
    environment TEXT DEFAULT '',
    command TEXT DEFAULT '',
    working_dir TEXT DEFAULT '',
    tags TEXT DEFAULT '',
    container_name TEXT DEFAULT '',
    cpu_usage TEXT DEFAULT '',
    memory_usage TEXT DEFAULT '',
    image_name TEXT DEFAULT '',
    creator_id BIGINT REFERENCES users(id),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);


ALTER TABLE flow_runtime_nodes
    ADD COLUMN human integer DEFAULT 0;



ALTER TABLE flow_runtime_nodes
    ADD COLUMN flow_node_id text DEFAULT '';
