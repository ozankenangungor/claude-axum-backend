use crate::db::models::*;
use anyhow::Result;
use sqlx::PgPool;

pub struct SocialService {
    pub pool: PgPool,
}

impl SocialService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Posts
    pub async fn create_post(&self, user_id: i32, create_post: CreatePost) -> Result<Post> {
        let post = sqlx::query_as!(
            Post,
            r#"
            INSERT INTO posts (user_id, content, image_url, reply_to_post_id)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, content, image_url, like_count, comment_count, repost_count,
                      created_at, updated_at, reply_to_post_id, is_deleted, deleted_at
            "#,
            user_id,
            create_post.content,
            create_post.image_url,
            create_post.reply_to_post_id
        )
        .fetch_one(&self.pool)
        .await?;

        // Update user's post count
        sqlx::query!(
            "UPDATE users SET post_count = COALESCE(post_count, 0) + 1 WHERE id = $1",
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(post)
    }

    pub async fn get_post(&self, post_id: i32) -> Result<Option<Post>> {
        let post = sqlx::query_as!(
            Post,
            r#"
            SELECT id, user_id, content, image_url, like_count, comment_count, repost_count,
                   created_at, updated_at, reply_to_post_id, is_deleted, deleted_at
            FROM posts
            WHERE id = $1 AND (is_deleted IS NULL OR is_deleted = FALSE)
            "#,
            post_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(post)
    }

    pub async fn get_user_posts(&self, user_id: i32, limit: i64, offset: i64) -> Result<Vec<Post>> {
        let posts = sqlx::query_as!(
            Post,
            r#"
            SELECT id, user_id, content, image_url, like_count, comment_count, repost_count,
                   created_at, updated_at, reply_to_post_id, is_deleted, deleted_at
            FROM posts
            WHERE user_id = $1 AND (is_deleted IS NULL OR is_deleted = FALSE)
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(posts)
    }

    pub async fn get_feed_posts(&self, user_id: i32, limit: i64, offset: i64) -> Result<Vec<Post>> {
        let posts = sqlx::query_as!(
            Post,
            r#"
            SELECT p.id, p.user_id, p.content, p.image_url, p.like_count, p.comment_count, 
                   p.repost_count, p.created_at, p.updated_at, p.reply_to_post_id, p.is_deleted, p.deleted_at
            FROM posts p
            INNER JOIN follows f ON p.user_id = f.following_id
            WHERE f.follower_id = $1 AND (p.is_deleted IS NULL OR p.is_deleted = FALSE)
            ORDER BY p.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(posts)
    }

    pub async fn update_post(&self, post_id: i32, user_id: i32, update_post: UpdatePost) -> Result<Option<Post>> {
        let post = sqlx::query_as!(
            Post,
            r#"
            UPDATE posts
            SET content = COALESCE($1, content),
                image_url = COALESCE($2, image_url),
                updated_at = NOW()
            WHERE id = $3 AND user_id = $4 AND (is_deleted IS NULL OR is_deleted = FALSE)
            RETURNING id, user_id, content, image_url, like_count, comment_count, repost_count,
                      created_at, updated_at, reply_to_post_id, is_deleted, deleted_at
            "#,
            update_post.content,
            update_post.image_url,
            post_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(post)
    }

    pub async fn delete_post(&self, post_id: i32, user_id: i32) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE posts
            SET is_deleted = TRUE, deleted_at = NOW()
            WHERE id = $1 AND user_id = $2 AND (is_deleted IS NULL OR is_deleted = FALSE)
            "#,
            post_id,
            user_id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            sqlx::query!(
                "UPDATE users SET post_count = GREATEST(0, COALESCE(post_count, 0) - 1) WHERE id = $1",
                user_id
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(result.rows_affected() > 0)
    }

    // Follows
    pub async fn follow_user(&self, follower_id: i32, following_id: i32) -> Result<Follow> {
        let follow = sqlx::query_as!(
            Follow,
            r#"
            INSERT INTO follows (follower_id, following_id)
            VALUES ($1, $2)
            RETURNING id, follower_id, following_id, created_at
            "#,
            follower_id,
            following_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(follow)
    }

    pub async fn unfollow_user(&self, follower_id: i32, following_id: i32) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM follows WHERE follower_id = $1 AND following_id = $2",
            follower_id,
            following_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn is_following(&self, follower_id: i32, following_id: i32) -> Result<bool> {
        let exists = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM follows WHERE follower_id = $1 AND following_id = $2) as exists",
            follower_id,
            following_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(exists.exists.unwrap_or(false))
    }

    pub async fn get_followers(&self, user_id: i32, limit: i64, offset: i64) -> Result<Vec<UserProfile>> {
        let users = sqlx::query_as!(
            UserProfile,
            r#"
            SELECT u.id, u.username, u.display_name, u.bio, u.avatar_url, u.location, 
                   u.website, u.is_verified, u.is_private, u.follower_count, u.following_count, 
                   u.post_count, u.created
            FROM users u
            INNER JOIN follows f ON u.id = f.follower_id
            WHERE f.following_id = $1
            ORDER BY f.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    pub async fn get_following(&self, user_id: i32, limit: i64, offset: i64) -> Result<Vec<UserProfile>> {
        let users = sqlx::query_as!(
            UserProfile,
            r#"
            SELECT u.id, u.username, u.display_name, u.bio, u.avatar_url, u.location, 
                   u.website, u.is_verified, u.is_private, u.follower_count, u.following_count, 
                   u.post_count, u.created
            FROM users u
            INNER JOIN follows f ON u.id = f.following_id
            WHERE f.follower_id = $1
            ORDER BY f.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    // Likes
    pub async fn like_post(&self, user_id: i32, post_id: i32) -> Result<Like> {
        let like = sqlx::query_as!(
            Like,
            r#"
            INSERT INTO likes (user_id, post_id)
            VALUES ($1, $2)
            RETURNING id, user_id, post_id, created_at
            "#,
            user_id,
            post_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(like)
    }

    pub async fn unlike_post(&self, user_id: i32, post_id: i32) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM likes WHERE user_id = $1 AND post_id = $2",
            user_id,
            post_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn is_liked(&self, user_id: i32, post_id: i32) -> Result<bool> {
        let exists = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM likes WHERE user_id = $1 AND post_id = $2) as exists",
            user_id,
            post_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(exists.exists.unwrap_or(false))
    }

    // Comments
    pub async fn create_comment(&self, user_id: i32, create_comment: CreateComment) -> Result<Comment> {
        let comment = sqlx::query_as!(
            Comment,
            r#"
            INSERT INTO comments (user_id, post_id, content, reply_to_comment_id)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, post_id, content, like_count, created_at, updated_at,
                      reply_to_comment_id, is_deleted, deleted_at
            "#,
            user_id,
            create_comment.post_id,
            create_comment.content,
            create_comment.reply_to_comment_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    pub async fn get_post_comments(&self, post_id: i32, limit: i64, offset: i64) -> Result<Vec<Comment>> {
        let comments = sqlx::query_as!(
            Comment,
            r#"
            SELECT id, user_id, post_id, content, like_count, created_at, updated_at,
                   reply_to_comment_id, is_deleted, deleted_at
            FROM comments
            WHERE post_id = $1 AND (is_deleted IS NULL OR is_deleted = FALSE)
            ORDER BY created_at ASC
            LIMIT $2 OFFSET $3
            "#,
            post_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(comments)
    }

    pub async fn update_comment(&self, comment_id: i32, user_id: i32, update_comment: UpdateComment) -> Result<Option<Comment>> {
        let comment = sqlx::query_as!(
            Comment,
            r#"
            UPDATE comments
            SET content = $1, updated_at = NOW()
            WHERE id = $2 AND user_id = $3 AND (is_deleted IS NULL OR is_deleted = FALSE)
            RETURNING id, user_id, post_id, content, like_count, created_at, updated_at,
                      reply_to_comment_id, is_deleted, deleted_at
            "#,
            update_comment.content,
            comment_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(comment)
    }

    pub async fn delete_comment(&self, comment_id: i32, user_id: i32) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE comments
            SET is_deleted = TRUE, deleted_at = NOW()
            WHERE id = $1 AND user_id = $2 AND (is_deleted IS NULL OR is_deleted = FALSE)
            "#,
            comment_id,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // User Profile
    pub async fn get_user_profile(&self, user_id: i32) -> Result<Option<UserProfile>> {
        let user = sqlx::query_as!(
            UserProfile,
            r#"
            SELECT id, username, display_name, bio, avatar_url, location, 
                   website, is_verified, is_private, follower_count, following_count, 
                   post_count, created
            FROM users
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_user_profile(&self, user_id: i32, update_profile: UpdateUserProfile) -> Result<Option<UserProfile>> {
        let user = sqlx::query_as!(
            UserProfile,
            r#"
            UPDATE users
            SET display_name = COALESCE($1, display_name),
                bio = COALESCE($2, bio),
                avatar_url = COALESCE($3, avatar_url),
                location = COALESCE($4, location),
                website = COALESCE($5, website),
                is_private = COALESCE($6, is_private),
                updated = NOW()
            WHERE id = $7
            RETURNING id, username, display_name, bio, avatar_url, location, 
                      website, is_verified, is_private, follower_count, following_count, 
                      post_count, created
            "#,
            update_profile.display_name,
            update_profile.bio,
            update_profile.avatar_url,
            update_profile.location,
            update_profile.website,
            update_profile.is_private,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn search_users(&self, query: &str, limit: i64, offset: i64) -> Result<Vec<UserProfile>> {
        let search_term = format!("%{}%", query);
        let users = sqlx::query_as!(
            UserProfile,
            r#"
            SELECT id, username, display_name, bio, avatar_url, location, 
                   website, is_verified, is_private, follower_count, following_count, 
                   post_count, created
            FROM users
            WHERE username ILIKE $1 
               OR display_name ILIKE $1
            ORDER BY 
                CASE WHEN username ILIKE $1 THEN 1 ELSE 2 END,
                follower_count DESC
            LIMIT $2 OFFSET $3
            "#,
            search_term,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }
}