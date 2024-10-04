#![allow(dead_code)]

use fake::faker::internet::en::SafeEmail;
use fake::faker::lorem::en::{Sentence, Words};
use fake::faker::name::en::{FirstName, LastName};
use fake::Dummy;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Dummy, Eq, Hash)]
pub enum Status {
    Pending,
    Completed,
    Archived,
}

impl Display for ToDo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\n   id: {}\n   status: {}\n   text: \"{}\"\n   point: {}\n   user: {}\n   ...\n }}",
            self.id, self.status, self.text, self.point, self.user
        )
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Pending => write!(f, "Status(Pending)"),
            Status::Completed => write!(f, "Status(Completed)"),
            Status::Archived => write!(f, "Status(Archived)"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Dummy, Eq, Hash)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Dummy, Eq, Hash)]
pub struct User {
    pub name: Name,
    #[dummy(faker = "SafeEmail()")]
    pub email: String,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\n      name: \"{:>}\"\n      email: \"{}\"\n   }}",
            self.name.first, self.email
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Dummy, Eq, Hash)]
pub struct Name {
    #[dummy(faker = "FirstName()")]
    pub first: String,
    #[dummy(faker = "LastName()")]
    pub last: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Dummy, Eq, Hash)]
pub struct Date {
    pub start: String,
    pub end: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonContent {
    pub records: Vec<ToDo>,
}

#[derive(Debug)]
pub enum Data {
    SingleTodo(ToDo),
    ListOfTodos(Vec<ToDo>),
}
