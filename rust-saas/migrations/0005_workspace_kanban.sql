-- Create workspace_files table
CREATE TABLE IF NOT EXISTS workspace_files (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    file_path TEXT NOT NULL,
    file_name TEXT NOT NULL,
    file_size BIGINT NOT NULL,
    is_directory BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create kanban_boards table
CREATE TABLE IF NOT EXISTS kanban_boards (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    is_public BOOLEAN NOT NULL DEFAULT TRUE,
    created_by BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create kanban_items table
CREATE TABLE IF NOT EXISTS kanban_items (
    id BIGSERIAL PRIMARY KEY,
    board_id BIGINT NOT NULL REFERENCES kanban_boards(id),
    user_id BIGINT NOT NULL,
    file_path TEXT NOT NULL,
    file_name TEXT NOT NULL,
    shared_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create kanban_subscriptions table
CREATE TABLE IF NOT EXISTS kanban_subscriptions (
    id BIGSERIAL PRIMARY KEY,
    board_id BIGINT NOT NULL REFERENCES kanban_boards(id),
    user_id BIGINT NOT NULL,
    subscribed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(board_id, user_id)
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_workspace_files_user_id ON workspace_files(user_id);
CREATE INDEX IF NOT EXISTS idx_kanban_boards_created_by ON kanban_boards(created_by);
CREATE INDEX IF NOT EXISTS idx_kanban_items_board_id ON kanban_items(board_id);
CREATE INDEX IF NOT EXISTS idx_kanban_subscriptions_board_id ON kanban_subscriptions(board_id);
CREATE INDEX IF NOT EXISTS idx_kanban_subscriptions_user_id ON kanban_subscriptions(user_id);
