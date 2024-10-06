#![allow(dead_code)]

use colored::Colorize;
use fake::faker::internet::en::SafeEmail;
use fake::faker::lorem::en::{Sentence, Words};
use fake::faker::name::en::{FirstName, LastName};
use fake::Dummy;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Dummy, Eq, Hash, Default)]
pub enum Status {
    Pending,
    Completed,
    #[default]
    Archived,
}

impl Display for ToDo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n   {}: {},\n   {}: {},\n   {}: \"{}\",\n   {}: {},\n   {}: {},\n   {}\n {}",
            "{".bright_green().bold(),
            "id".bright_yellow().bold(),
            self.id.bright_cyan(),
            "status".bright_yellow().bold(),
            self.status.to_string().bright_cyan(),
            "text".bright_yellow().bold(),
            self.text.bright_cyan(),
            "point".bright_yellow().bold(),
            self.point.to_string().bright_cyan(),
            "user".bright_yellow().bold(),
            self.user.to_string().bright_cyan(),
            "...".bright_yellow().bold(),
            "}".bright_green().bold()
        )
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Pending => write!(f, "Status({})", "Pending".bright_purple().bold()),
            Status::Completed => write!(f, "Status({})", "Completed".bright_green().bold()),
            Status::Archived => write!(f, "Status({})", "Archived".yellow().bold()),
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

impl Default for ToDo {
    fn default() -> Self {
        Self {
            id: "1".to_string(),
            text: "Learn Rust".to_string(),
            status: Status::Pending,
            user: User {
                name: Name {
                    first: "John".to_string(),
                    last: "Doe".to_string(),
                },
                email: "admin@gmail.com".to_string(),
            },
            date: Date {
                start: "2024-01-01".to_string(),
                end: "2025-01-01".to_string(),
            },
            tags: vec!["rust".to_string(), "programming".to_string()],
            point: 100,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Dummy, Eq, Hash, Default)]
pub struct User {
    pub name: Name,
    #[dummy(faker = "SafeEmail()")]
    pub email: String,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\n      {}: {},\n      {}: \"{}\",\n   }}",
            "name".yellow().bold(),
            self.name,
            "email".yellow().bold(),
            self.email,
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Dummy, Eq, Hash, Default)]
pub struct Name {
    #[dummy(faker = "FirstName()")]
    pub first: String,
    #[dummy(faker = "LastName()")]
    pub last: String,
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\n\t{}: \"{}\",\n\t{}: \"{}\",\n      }}",
            "first".yellow().bold(),
            self.first,
            "last".yellow().bold(),
            self.last,
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Dummy, Eq, Hash, Default)]
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
