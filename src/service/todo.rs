use sqlx::PgPool;
use thiserror::Error;

use crate::{
    db::{
        models::{TodoModel, UpdateTodo, UpdateTodoPartial},
        DbConnectionPoolError,
    },
    handlers::todo::models::CreateTodoRequest,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Connection pool init error: {0}")]
    ConnectionPool(#[from] DbConnectionPoolError),
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Todo not found")]
    TodoNotFound,
}

pub struct Service {
    db_pool: PgPool,
}

impl Service {
    pub fn new(db_pool: PgPool) -> Result<Self, Error> {
        Ok(Self { db_pool })
    }

    /// Get reference to the database pool for health checks
    pub fn get_pool(&self) -> &PgPool {
        &self.db_pool
    }

    pub async fn create(
        &self,
        user_id: i32,
        request: CreateTodoRequest,
    ) -> Result<TodoModel, Error> {
        let todo = sqlx::query_as!(
            TodoModel,
            r#"
            INSERT INTO todos (title, description, user_id)
            VALUES ($1, $2, $3)
            RETURNING id, title, description, created, updated, user_id
            "#,
            request.title,
            request.description,
            user_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(todo)
    }

    pub async fn list(&self, user_id: i32) -> Result<Vec<TodoModel>, Error> {
        let todos = sqlx::query_as!(
            TodoModel,
            r#"
            SELECT id, title, description, created, updated, user_id
            FROM todos
            WHERE user_id = $1
            ORDER BY created DESC
            "#,
            user_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(todos)
    }

    pub async fn get(&self, user_id: i32, id: i32) -> Result<TodoModel, Error> {
        let todo = sqlx::query_as!(
            TodoModel,
            r#"
            SELECT id, title, description, created, updated, user_id
            FROM todos
            WHERE id = $1 AND user_id = $2
            "#,
            id,
            user_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        match todo {
            Some(todo) => Ok(todo),
            None => Err(Error::TodoNotFound),
        }
    }

    pub async fn delete(&self, user_id: i32, id: i32) -> Result<(), Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM todos
            WHERE id = $1 AND user_id = $2
            "#,
            id,
            user_id
        )
        .execute(&self.db_pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::TodoNotFound);
        }

        Ok(())
    }

    pub async fn partial_update(
        &self,
        user_id: i32,
        id: i32,
        request: UpdateTodoPartial,
    ) -> Result<(), Error> {
        if request.title.is_none() && request.description.is_none() {
            return Ok(());
        }

        let result = match (&request.title, &request.description) {
            (Some(title), Some(description)) => {
                sqlx::query!(
                    "UPDATE todos SET title = $1, description = $2, updated = NOW() WHERE id = $3 AND user_id = $4",
                    title,
                    description,
                    id,
                    user_id
                )
                .execute(&self.db_pool)
                .await?
            }
            (Some(title), None) => {
                sqlx::query!(
                    "UPDATE todos SET title = $1, updated = NOW() WHERE id = $2 AND user_id = $3",
                    title,
                    id,
                    user_id
                )
                .execute(&self.db_pool)
                .await?
            }
            (None, Some(description)) => {
                sqlx::query!(
                    "UPDATE todos SET description = $1, updated = NOW() WHERE id = $2 AND user_id = $3",
                    description,
                    id,
                    user_id
                )
                .execute(&self.db_pool)
                .await?
            }
            (None, None) => return Ok(()),
        };

        if result.rows_affected() == 0 {
            return Err(Error::TodoNotFound);
        }

        Ok(())
    }

    pub async fn update(&self, user_id: i32, id: i32, request: UpdateTodo) -> Result<(), Error> {
        let result = sqlx::query!(
            r#"
            UPDATE todos 
            SET title = $1, description = $2, updated = NOW()
            WHERE id = $3 AND user_id = $4
            "#,
            request.title,
            request.description,
            id,
            user_id
        )
        .execute(&self.db_pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::TodoNotFound);
        }

        Ok(())
    }
}
