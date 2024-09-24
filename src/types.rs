#![allow(dead_code)]

use fake::faker::internet::en::SafeEmail;
use fake::faker::lorem::en::{Sentence, Words};
use fake::faker::name::en::{FirstName, LastName};
use fake::Dummy;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug, Clone, Dummy)]
pub enum Status {
    Pending,
    Completed,
    Archived,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Pending => write!(f, "Pending"),
            Status::Completed => write!(f, "Completed"),
            Status::Archived => write!(f, "Archived"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Dummy)]
pub struct ToDo {
    pub id: String,
    #[dummy(faker = "Sentence(5..10)")]
    pub text: String,
    pub status: Status,
    pub user: User,
    pub date: Date,
    #[dummy(faker = "100..1000")]
    pub point: u64,
    #[dummy(faker = "Words(2..5)")]
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Dummy)]
pub struct User {
    pub name: Name,
    #[dummy(faker = "SafeEmail()")]
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Dummy)]
pub struct Name {
    #[dummy(faker = "FirstName()")]
    pub first: String,
    #[dummy(faker = "LastName()")]
    pub last: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Dummy)]
pub struct Date {
    pub start: String,
    pub end: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonContent {
    pub todos: Vec<ToDo>,
}

#[derive(Debug)]
pub enum Data {
    SingleTodo(ToDo),
    ListOfTodos(Vec<ToDo>),
}
