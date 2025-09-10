-- Create follows table (many-to-many relationship)
CREATE TABLE IF NOT EXISTS follows (
    id SERIAL PRIMARY KEY,
    follower_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    following_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    -- Prevent self-following and duplicate follows
    CONSTRAINT no_self_follow CHECK (follower_id != following_id),
    CONSTRAINT unique_follow UNIQUE (follower_id, following_id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_follows_follower_id ON follows(follower_id);
CREATE INDEX IF NOT EXISTS idx_follows_following_id ON follows(following_id);
CREATE INDEX IF NOT EXISTS idx_follows_created_at ON follows(created_at DESC);

-- Function to update user follow counts
CREATE OR REPLACE FUNCTION update_follow_counts()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- Increase following count for follower
        UPDATE users SET following_count = following_count + 1 WHERE id = NEW.follower_id;
        -- Increase follower count for followed user
        UPDATE users SET follower_count = follower_count + 1 WHERE id = NEW.following_id;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        -- Decrease following count for follower
        UPDATE users SET following_count = following_count - 1 WHERE id = OLD.follower_id;
        -- Decrease follower count for followed user
        UPDATE users SET follower_count = follower_count - 1 WHERE id = OLD.following_id;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ language 'plpgsql';

-- Triggers to automatically update follow counts
DROP TRIGGER IF EXISTS trigger_follow_counts ON follows;
CREATE TRIGGER trigger_follow_counts
    AFTER INSERT OR DELETE ON follows
    FOR EACH ROW
EXECUTE PROCEDURE update_follow_counts();