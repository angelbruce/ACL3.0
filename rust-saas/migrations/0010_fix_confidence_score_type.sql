-- 修改 confidence_score 字段类型从 NUMERIC 改为 DOUBLE PRECISION
ALTER TABLE project_file_container_assignments
ALTER COLUMN confidence_score TYPE DOUBLE PRECISION;