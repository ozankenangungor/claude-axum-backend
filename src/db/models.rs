use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
    // Social media fields
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub is_verified: Option<bool>,
    pub is_private: Option<bool>,
    pub follower_count: Option<i32>,
    pub following_count: Option<i32>,
    pub post_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserProfile {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub is_private: Option<bool>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TodoModel {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
    pub user_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTodo {
    pub user_id: i32,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTodoPartial {
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTodo {
    pub title: String,
    pub description: String,
}

// Social Media Models

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub content: String,
    pub image_url: Option<String>,
    pub like_count: Option<i32>,
    pub comment_count: Option<i32>,
    pub repost_count: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub reply_to_post_id: Option<i32>,
    pub is_deleted: Option<bool>,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePost {
    pub content: String,
    pub image_url: Option<String>,
    pub reply_to_post_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePost {
    pub content: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Follow {
    pub id: i32,
    pub follower_id: i32,
    pub following_id: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFollow {
    pub following_id: i32,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Like {
    pub id: i32,
    pub user_id: i32,
    pub post_id: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLike {
    pub post_id: i32,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Comment {
    pub id: i32,
    pub user_id: i32,
    pub post_id: i32,
    pub content: String,
    pub like_count: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub reply_to_comment_id: Option<i32>,
    pub is_deleted: Option<bool>,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateComment {
    pub post_id: i32,
    pub content: String,
    pub reply_to_comment_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateComment {
    pub content: String,
}

// Response DTOs (Data Transfer Objects)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostWithUser {
    #[serde(flatten)]
    pub post: Post,
    pub user: UserProfile,
    pub is_liked: bool,
    pub is_following_author: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: i32,
    pub username: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub is_verified: Option<bool>,
    pub is_private: Option<bool>,
    pub follower_count: Option<i32>,
    pub following_count: Option<i32>,
    pub post_count: Option<i32>,
    pub created: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentWithUser {
    #[serde(flatten)]
    pub comment: Comment,
    pub user: UserProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedPost {
    #[serde(flatten)]
    pub post: PostWithUser,
    pub comments: Vec<CommentWithUser>,
}
