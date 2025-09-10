-- Create todos table
CREATE TABLE IF NOT EXISTS todos (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    created TIMESTAMP NOT NULL DEFAULT NOW(),
    updated TIMESTAMP NOT NULL DEFAULT NOW(),
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_todos_user_id ON todos(user_id);
CREATE INDEX IF NOT EXISTS idx_todos_created ON todos(created);
CREATE INDEX IF NOT EXISTS idx_todos_user_id_created ON todos(user_id, created DESC);
CREATE INDEX IF NOT EXISTS idx_todos_title ON todos(title) WHERE title IS NOT NULL;

DROP TRIGGER IF EXISTS update_todos_updated ON todos;
CREATE TRIGGER update_todos_updated
    BEFORE UPDATE
    ON todos
    FOR EACH ROW
EXECUTE PROCEDURE update_updated_column();