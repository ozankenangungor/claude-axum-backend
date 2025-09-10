use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::models::{TodoModel, UpdateTodo, UpdateTodoPartial};

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub description: String,
}

impl From<TodoModel> for Todo {
    fn from(model: TodoModel) -> Self {
        Self {
            id: model.id as u64,
            title: model.title,
            description: model.description,
        }
    }
}

impl From<&TodoModel> for Todo {
    fn from(model: &TodoModel) -> Self {
        Self {
            id: model.id as u64,
            title: model.title.clone(),
            description: model.description.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateTodoRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Title must be between 1 and 255 characters"
    ))]
    pub title: String,
    #[validate(length(
        min = 1,
        max = 1000,
        message = "Description must be between 1 and 1000 characters"
    ))]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PartialUpdateTodoRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Title must be between 1 and 255 characters"
    ))]
    pub title: Option<String>,
    #[validate(length(
        min = 1,
        max = 1000,
        message = "Description must be between 1 and 1000 characters"
    ))]
    pub description: Option<String>,
}

impl From<PartialUpdateTodoRequest> for UpdateTodoPartial {
    fn from(value: PartialUpdateTodoRequest) -> Self {
        Self {
            title: value.title,
            description: value.description,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateTodoRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Title must be between 1 and 255 characters"
    ))]
    pub title: String,
    #[validate(length(
        min = 1,
        max = 1000,
        message = "Description must be between 1 and 1000 characters"
    ))]
    pub description: String,
}

impl From<UpdateTodoRequest> for UpdateTodo {
    fn from(value: UpdateTodoRequest) -> Self {
        Self {
            title: value.title,
            description: value.description,
        }
    }
}
