-- Add agent_id and model_id columns to sessions table
ALTER TABLE sessions ADD COLUMN agent_id BIGINT REFERENCES agents(id);
ALTER TABLE sessions ADD COLUMN model_id BIGINT REFERENCES llm_models(id);
