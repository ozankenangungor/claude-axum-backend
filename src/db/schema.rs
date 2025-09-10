use anyhow::Result;
use sqlx::PgPool;

/// Initialize database schema with all tables, indexes, and triggers
pub async fn initialize_schema(pool: &PgPool) -> Result<()> {
    run_migrations(pool).await
}

/// Run all migrations in order
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    // Create functions first
    create_update_function(pool).await?;
    create_follow_count_function(pool).await?;
    create_like_count_function(pool).await?;
    create_comment_count_function(pool).await?;

    // Create tables
    create_users_table(pool).await?;
    create_todos_table(pool).await?;

    // Extend users table for social media
    extend_users_for_social_media(pool).await?;

    // Create social media tables
    create_posts_table(pool).await?;
    create_follows_table(pool).await?;
    create_likes_table(pool).await?;
    create_comments_table(pool).await?;

    // Create indexes
    create_users_indexes(pool).await?;
    create_todos_indexes(pool).await?;
    create_posts_indexes(pool).await?;
    create_follows_indexes(pool).await?;
    create_likes_indexes(pool).await?;
    create_comments_indexes(pool).await?;

    // Create triggers
    create_users_trigger(pool).await?;
    create_todos_trigger(pool).await?;
    create_posts_trigger(pool).await?;
    create_follows_trigger(pool).await?;
    create_likes_trigger(pool).await?;
    create_comments_trigger(pool).await?;

    println!("All migrations applied successfully!");
    Ok(())
}

#[allow(dead_code)] // Function reserved for potential manual trigger creation
async fn create_trigger_functions(pool: &PgPool) -> Result<()> {
    // Update timestamp trigger function
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION update_updated_column()
        RETURNS TRIGGER AS $$
        BEGIN
            NEW.updated = now();
            RETURN NEW;
        END;
        $$ language 'plpgsql'
    "#,
    )
    .execute(pool)
    .await?;

    // Follow counts trigger function
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION update_follow_counts()
        RETURNS TRIGGER AS $$
        BEGIN
            IF TG_OP = 'INSERT' THEN
                UPDATE users SET following_count = following_count + 1 WHERE id = NEW.follower_id;
                UPDATE users SET follower_count = follower_count + 1 WHERE id = NEW.following_id;
                RETURN NEW;
            ELSIF TG_OP = 'DELETE' THEN
                UPDATE users SET following_count = following_count - 1 WHERE id = OLD.follower_id;
                UPDATE users SET follower_count = follower_count - 1 WHERE id = OLD.following_id;
                RETURN OLD;
            END IF;
            RETURN NULL;
        END;
        $$ language 'plpgsql'
    "#,
    )
    .execute(pool)
    .await?;

    // Like counts trigger function
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION update_like_counts()
        RETURNS TRIGGER AS $$
        BEGIN
            IF TG_OP = 'INSERT' THEN
                UPDATE posts SET like_count = like_count + 1 WHERE id = NEW.post_id;
                RETURN NEW;
            ELSIF TG_OP = 'DELETE' THEN
                UPDATE posts SET like_count = like_count - 1 WHERE id = OLD.post_id;
                RETURN OLD;
            END IF;
            RETURN NULL;
        END;
        $$ language 'plpgsql'
    "#,
    )
    .execute(pool)
    .await?;

    // Comment counts trigger function
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION update_comment_counts()
        RETURNS TRIGGER AS $$
        BEGIN
            IF TG_OP = 'INSERT' THEN
                UPDATE posts SET comment_count = comment_count + 1 WHERE id = NEW.post_id;
                RETURN NEW;
            ELSIF TG_OP = 'DELETE' THEN
                UPDATE posts SET comment_count = comment_count - 1 WHERE id = OLD.post_id;
                RETURN OLD;
            END IF;
            RETURN NULL;
        END;
        $$ language 'plpgsql'
    "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn create_users_table(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username VARCHAR(255) NOT NULL UNIQUE,
            password TEXT NOT NULL,
            created TIMESTAMP NOT NULL DEFAULT NOW(),
            updated TIMESTAMP NOT NULL DEFAULT NOW()
        )
    "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn create_users_indexes(pool: &PgPool) -> Result<()> {
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_created ON users(created)")
        .execute(pool)
        .await?;

    Ok(())
}

async fn create_users_trigger(pool: &PgPool) -> Result<()> {
    sqlx::query("DROP TRIGGER IF EXISTS update_users_updated ON users")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER update_users_updated
            BEFORE UPDATE
            ON users
            FOR EACH ROW
        EXECUTE PROCEDURE update_updated_column()
    "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn create_todos_table(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS todos (
            id SERIAL PRIMARY KEY,
            title VARCHAR(255) NOT NULL,
            description TEXT NOT NULL,
            created TIMESTAMP NOT NULL DEFAULT NOW(),
            updated TIMESTAMP NOT NULL DEFAULT NOW(),
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE
        )
    "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn create_todos_indexes(pool: &PgPool) -> Result<()> {
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_todos_user_id ON todos(user_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_todos_created ON todos(created)")
        .execute(pool)
        .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_todos_user_id_created ON todos(user_id, created DESC)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_todos_title ON todos(title) WHERE title IS NOT NULL",
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn create_todos_trigger(pool: &PgPool) -> Result<()> {
    sqlx::query("DROP TRIGGER IF EXISTS update_todos_updated ON todos")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER update_todos_updated
            BEFORE UPDATE
            ON todos
            FOR EACH ROW
        EXECUTE PROCEDURE update_updated_column()
    "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Social Media Table Functions

async fn extend_users_for_social_media(pool: &PgPool) -> Result<()> {
    // Add columns one by one
    sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS email VARCHAR(255) UNIQUE")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS display_name VARCHAR(100)")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS bio TEXT")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS avatar_url TEXT")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS location VARCHAR(100)")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS website VARCHAR(255)")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS is_verified BOOLEAN DEFAULT FALSE")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS is_private BOOLEAN DEFAULT FALSE")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS follower_count INTEGER DEFAULT 0")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS following_count INTEGER DEFAULT 0")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS post_count INTEGER DEFAULT 0")
        .execute(pool)
        .await?;

    // Add indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_display_name ON users(display_name)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_is_verified ON users(is_verified)")
        .execute(pool)
        .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_users_follower_count ON users(follower_count DESC)",
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn create_posts_table(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS posts (
            id SERIAL PRIMARY KEY,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            content TEXT NOT NULL CHECK (char_length(content) <= 280),
            image_url TEXT,
            like_count INTEGER DEFAULT 0,
            comment_count INTEGER DEFAULT 0,
            repost_count INTEGER DEFAULT 0,
            created_at TIMESTAMP NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
            reply_to_post_id INTEGER REFERENCES posts(id) ON DELETE CASCADE,
            is_deleted BOOLEAN DEFAULT FALSE,
            deleted_at TIMESTAMP
        )
    "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn create_posts_indexes(pool: &PgPool) -> Result<()> {
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_posts_user_id ON posts(user_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_posts_created_at ON posts(created_at DESC)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_posts_user_id_created_at ON posts(user_id, created_at DESC)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_posts_reply_to ON posts(reply_to_post_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_posts_like_count ON posts(like_count DESC)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_posts_is_deleted ON posts(is_deleted)")
        .execute(pool)
        .await?;

    Ok(())
}

async fn create_posts_trigger(pool: &PgPool) -> Result<()> {
    sqlx::query("DROP TRIGGER IF EXISTS update_posts_updated_at ON posts")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER update_posts_updated_at
            BEFORE UPDATE
            ON posts
            FOR EACH ROW
        EXECUTE PROCEDURE update_updated_column()
    "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn create_follows_table(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS follows (
            id SERIAL PRIMARY KEY,
            follower_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            following_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            created_at TIMESTAMP NOT NULL DEFAULT NOW(),
            CONSTRAINT no_self_follow CHECK (follower_id != following_id),
            CONSTRAINT unique_follow UNIQUE (follower_id, following_id)
        )
    "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn create_follows_indexes(pool: &PgPool) -> Result<()> {
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_follows_follower_id ON follows(follower_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_follows_following_id ON follows(following_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_follows_created_at ON follows(created_at DESC)")
        .execute(pool)
        .await?;

    Ok(())
}

async fn create_follows_trigger(pool: &PgPool) -> Result<()> {
    sqlx::query("DROP TRIGGER IF EXISTS trigger_follow_counts ON follows")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER trigger_follow_counts
            AFTER INSERT OR DELETE ON follows
            FOR EACH ROW
        EXECUTE PROCEDURE update_follow_counts()
    "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn create_likes_table(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS likes (
            id SERIAL PRIMARY KEY,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            post_id INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
            created_at TIMESTAMP NOT NULL DEFAULT NOW(),
            CONSTRAINT unique_like UNIQUE (user_id, post_id)
        )
    "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn create_likes_indexes(pool: &PgPool) -> Result<()> {
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_likes_user_id ON likes(user_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_likes_post_id ON likes(post_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_likes_created_at ON likes(created_at DESC)")
        .execute(pool)
        .await?;

    Ok(())
}

async fn create_likes_trigger(pool: &PgPool) -> Result<()> {
    sqlx::query("DROP TRIGGER IF EXISTS trigger_like_counts ON likes")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER trigger_like_counts
            AFTER INSERT OR DELETE ON likes
            FOR EACH ROW
        EXECUTE PROCEDURE update_like_counts()
    "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn create_comments_table(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS comments (
            id SERIAL PRIMARY KEY,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            post_id INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
            content TEXT NOT NULL CHECK (char_length(content) <= 280),
            like_count INTEGER DEFAULT 0,
            created_at TIMESTAMP NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
            reply_to_comment_id INTEGER REFERENCES comments(id) ON DELETE CASCADE,
            is_deleted BOOLEAN DEFAULT FALSE,
            deleted_at TIMESTAMP
        )
    "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn create_comments_indexes(pool: &PgPool) -> Result<()> {
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_comments_user_id ON comments(user_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_comments_post_id ON comments(post_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_comments_created_at ON comments(created_at DESC)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_comments_post_id_created_at ON comments(post_id, created_at DESC)")
        .execute(pool)
        .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_comments_reply_to ON comments(reply_to_comment_id)",
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_comments_is_deleted ON comments(is_deleted)")
        .execute(pool)
        .await?;

    Ok(())
}

async fn create_comments_trigger(pool: &PgPool) -> Result<()> {
    sqlx::query("DROP TRIGGER IF EXISTS trigger_comment_counts ON comments")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER trigger_comment_counts
            AFTER INSERT OR DELETE ON comments
            FOR EACH ROW
        EXECUTE PROCEDURE update_comment_counts()
    "#,
    )
    .execute(pool)
    .await?;

    sqlx::query("DROP TRIGGER IF EXISTS update_comments_updated_at ON comments")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER update_comments_updated_at
            BEFORE UPDATE
            ON comments
            FOR EACH ROW
        EXECUTE PROCEDURE update_updated_column()
    "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Database functions

async fn create_update_function(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION update_updated_column()
        RETURNS TRIGGER AS $$
        BEGIN
            NEW.updated_at = NOW();
            RETURN NEW;
        END;
        $$ language 'plpgsql'
    "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn create_follow_count_function(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION update_follow_counts()
        RETURNS TRIGGER AS $$
        BEGIN
            IF TG_OP = 'INSERT' THEN
                -- Increase follower count for the user being followed
                UPDATE users SET follower_count = follower_count + 1
                WHERE id = NEW.following_id;
                
                -- Increase following count for the user who follows
                UPDATE users SET following_count = following_count + 1
                WHERE id = NEW.follower_id;
                
                RETURN NEW;
            ELSIF TG_OP = 'DELETE' THEN
                -- Decrease follower count for the user being unfollowed
                UPDATE users SET follower_count = GREATEST(0, follower_count - 1)
                WHERE id = OLD.following_id;
                
                -- Decrease following count for the user who unfollows
                UPDATE users SET following_count = GREATEST(0, following_count - 1)
                WHERE id = OLD.follower_id;
                
                RETURN OLD;
            END IF;
            
            RETURN NULL;
        END;
        $$ language 'plpgsql'
    "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn create_like_count_function(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION update_like_counts()
        RETURNS TRIGGER AS $$
        BEGIN
            IF TG_OP = 'INSERT' THEN
                UPDATE posts SET like_count = like_count + 1
                WHERE id = NEW.post_id;
                RETURN NEW;
            ELSIF TG_OP = 'DELETE' THEN
                UPDATE posts SET like_count = GREATEST(0, like_count - 1)
                WHERE id = OLD.post_id;
                RETURN OLD;
            END IF;
            
            RETURN NULL;
        END;
        $$ language 'plpgsql'
    "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn create_comment_count_function(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION update_comment_counts()
        RETURNS TRIGGER AS $$
        BEGIN
            IF TG_OP = 'INSERT' THEN
                UPDATE posts SET comment_count = comment_count + 1
                WHERE id = NEW.post_id;
                RETURN NEW;
            ELSIF TG_OP = 'DELETE' THEN
                UPDATE posts SET comment_count = GREATEST(0, comment_count - 1)
                WHERE id = OLD.post_id;
                RETURN OLD;
            END IF;
            
            RETURN NULL;
        END;
        $$ language 'plpgsql'
    "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}
