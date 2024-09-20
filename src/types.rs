use fake::Dummy;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Dummy)]
pub enum Status {
    Pending,
    Completed,
    Archived,
}

#[derive(Serialize, Deserialize, Debug, Clone, Dummy)]
pub struct ToDo {
    pub id: String,
    pub name: String,
    pub status: Status,
    pub user: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonContent {
    pub todos: Vec<ToDo>,
}

#[derive(Debug)]
pub enum Data {
    SingleTodo(ToDo),
    ListOfTodo(Vec<ToDo>),
}
