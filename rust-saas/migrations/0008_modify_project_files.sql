ALTER TABLE project_files
    ADD COLUMN status integer DEFAULT 0;

UPDATE project_files
    SET status = 0 WHERE status IS NULL;
