-- vec-svc database tables

DROP TABLE IF EXISTS flow_runtime_sessions  CASCADE;
DROP TABLE IF EXISTS flow_runtime_session_items  CASCADE;


CREATE TABLE flow_runtime_sessions (
    id BIGSERIAL PRIMARY KEY,
    flow_id bigint not null,
    flow_runtime_id TEXT not null,
    creator_id BIGINT not null references users(id) on delete cascade,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);


CREATE TABLE flow_runtime_session_items (
    id BIGSERIAL PRIMARY KEY,
    flow_id bigint not null,
    flow_runtime_id TEXT not null,
    flow_runtime_session_id bigint not null,
    flow_runtime_node_id TEXT not null,
    session_type TEXT not null default 'Assistant',
    content TEXT,
    action_id bigint not null default 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    creator_id BIGINT
);