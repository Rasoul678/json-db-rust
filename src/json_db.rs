#![allow(dead_code)]

use crate::get_nested_value;
use crate::types::{JsonContent, Status, ToDo};
use serde_json::Value;
use std::collections::VecDeque;
use std::io::{self, ErrorKind};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone, PartialEq, Debug)]
enum Comparator {
    Equals(String),
    NotEquals(String),
    LessThan(u64),
    GreaterThan(u64),
    In(Vec<String>),
    Between((u64, u64)),
}

#[derive(Clone, PartialEq, Debug)]
enum CallerType {
    Create,
    Read,
    Update,
    Delete,
}

#[derive(Clone, PartialEq, Debug)]
enum Runner {
    Done,
    Caller(CallerType),
    Compare(Comparator),
    Where(String),
}

#[derive(Clone)]
pub struct JsonDB {
    name: String,
    path: PathBuf,
    file: Arc<File>,
    value: Arc<Vec<ToDo>>,
    runners: Arc<VecDeque<Runner>>,
}

impl JsonDB {
    pub async fn new(db_name: &str) -> Result<JsonDB, io::Error> {
        let dir_path = std::env::current_dir()?;
        let file_with_format = format!("{}.json", db_name);
        let path = dir_path.join(file_with_format);

        //? Try to open json file Or create a new one!
        let mut json_db_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .await?;

        //? Write into the file
        json_db_file.write_all(b"{\"todos\":[]}").await?;

        let db = Self {
            name: db_name.to_string(),
            path,
            file: json_db_file.into(),
            value: Arc::new(vec![]),
            runners: Arc::new(VecDeque::new()),
        };

        Ok(db)
    }

    async fn read(&self) -> Result<JsonContent, io::Error> {
        let mut file = OpenOptions::new().read(true).open(&self.path).await?;
        let mut content = String::new();
        file.read_to_string(&mut content).await?;
        let json_data: JsonContent = serde_json::from_str(&content)?;
        Ok(json_data)
    }

    async fn save(&self, content: JsonContent) -> Result<(), io::Error> {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .await?;
        file.write_all(serde_json::to_string_pretty(&content)?.as_bytes())
            .await?;
        file.flush().await?;
        Ok(())
    }

    async fn clear(&self) -> Result<(), io::Error> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)
            .await?;

        file.write_all(b"{\"todos\":[]}").await?;

        file.flush().await?;

        Ok(())
    }

    pub async fn insert<'a>(&'a self, item: &'a ToDo) -> Result<&'a ToDo, io::Error> {
        let mut content = self.read().await?;

        for t in content.todos.iter() {
            if t.id == item.id {
                return Err(io::Error::new(
                    ErrorKind::AlreadyExists,
                    "Record already exists",
                ));
            }
        }

        content.todos.push(item.clone());

        self.save(content).await?;

        Ok(item)
    }

    async fn get_all(&self) -> Result<Vec<ToDo>, io::Error> {
        let content = self.read().await?;
        Ok(content.todos)
    }

    pub async fn find_all(&self) -> Result<Vec<ToDo>, io::Error> {
        let todos = self.get_all().await?;
        Ok(todos)
    }

    pub async fn get_by_ids(&self, ids: &[&str]) -> Result<Vec<ToDo>, io::Error> {
        let content = self.read().await?;
        let todos = content
            .todos
            .into_iter()
            .filter(|todo| ids.contains(&todo.id.as_str()))
            .collect::<Vec<ToDo>>();
        Ok(todos)
    }

    async fn get_by_id_runner(&mut self, id: &str) -> Result<&Self, io::Error> {
        let content = self.read().await?;
        let todo = content
            .todos
            .into_iter()
            .find(|todo| todo.id == id)
            .ok_or(io::Error::new(ErrorKind::NotFound, "Record not found"))?;

        self.value = Arc::new(vec![todo]);
        /*
         * Setting runner to Done is crucial
         * to prevent a loop in the `run` method.
         */
        // self.runner = Arc::new(Runner::Done);

        Ok(self)
    }

    pub fn get_by_id(&mut self) -> &Self {
        let runners = Arc::make_mut(&mut self.runners);
        runners.push_front(Runner::Caller(CallerType::Read));
        self
    }
    pub async fn update(&self, id: &str, todo: ToDo) -> Result<ToDo, io::Error> {
        let mut content = self.read().await?;
        let todo_index = content
            .todos
            .iter()
            .position(|todo| todo.id == id)
            .ok_or(io::Error::new(ErrorKind::NotFound, "Record not found"))?;
        content.todos[todo_index] = todo.clone();
        self.save(content).await?;
        Ok(todo)
    }

    pub async fn delete(&self, id: &str) -> Result<ToDo, io::Error> {
        let mut content = self.read().await?;

        let todo_index = content
            .todos
            .iter()
            .position(|todo| todo.id == id)
            .ok_or(io::Error::new(ErrorKind::NotFound, "Record not found"))?;

        let deleted_item = content.todos.swap_remove(todo_index);

        self.save(content).await?;

        Ok(deleted_item)
    }

    pub async fn delete_by_id(&self, id: &str) -> Result<(), io::Error> {
        let mut content = self.read().await?;
        content.todos.retain(|todo| todo.id != id);
        self.save(content).await?;
        Ok(())
    }

    pub async fn delete_all(&self) -> Result<(), io::Error> {
        self.clear().await
    }

    pub async fn delete_completed_runner(&mut self) -> Result<&Self, io::Error> {
        let mut content = self.read().await?;
        let deleted_todos = content
            .todos
            .iter()
            .filter(|todo| match todo.status {
                Status::Completed => true,
                _ => false,
            })
            .cloned()
            .collect::<Vec<ToDo>>();

        content.todos.retain(|todo| match todo.status {
            Status::Completed => false,
            _ => true,
        });

        self.save(content).await?;
        self.value = Arc::new(deleted_todos);
        // self.runner = Arc::new(Runner::Done);
        Ok(self)
    }

    pub fn delete_completed(&mut self) -> &Self {
        let runners = Arc::make_mut(&mut self.runners);
        runners.push_front(Runner::Caller(CallerType::Read));
        self
    }

    pub async fn delete_archived_runner(&mut self) -> Result<&Self, io::Error> {
        let mut content = self.read().await?;
        let deleted_todos = content
            .todos
            .iter()
            .filter(|todo| match todo.status {
                Status::Archived => true,
                _ => false,
            })
            .cloned()
            .collect::<Vec<ToDo>>();

        content.todos.retain(|todo| match todo.status {
            Status::Archived => false,
            _ => true,
        });

        self.save(content).await?;
        self.value = Arc::new(deleted_todos);
        // self.runner = Arc::new(Runner::Done);
        Ok(self)
    }
    pub fn delete_archived(&mut self) -> &Self {
        let runners = Arc::make_mut(&mut self.runners);
        runners.push_front(Runner::Caller(CallerType::Read));
        self
    }

    pub async fn delete_not_completed(&self) -> Result<(), io::Error> {
        let mut content = self.read().await?;
        content.todos.retain(|todo| match todo.status {
            Status::Completed => true,
            _ => false,
        });
        self.save(content).await?;
        Ok(())
    }

    pub async fn delete_and_return_deleted(&self, ids: &[&str]) -> Result<Vec<ToDo>, io::Error> {
        let mut content = self.read().await?;
        let deleted_todos = content
            .todos
            .iter()
            .filter(|todo| ids.contains(&todo.id.as_str()))
            .cloned()
            .collect::<Vec<ToDo>>();
        content
            .todos
            .retain(|todo| !ids.contains(&todo.id.as_str()));

        self.save(content).await?;
        Ok(deleted_todos)
    }

    pub fn find(&self) -> Self {
        let mut runners = VecDeque::from(self.runners.as_ref().clone());
        runners.push_back(Runner::Caller(CallerType::Read));

        Self {
            runners: Arc::new(runners.clone()),
            ..self.clone()
        }
    }

    pub fn _where(&self, field: &str) -> Self {
        let mut runners = VecDeque::from(self.runners.as_ref().clone());
        runners.push_back(Runner::Where(field.to_string()));

        Self {
            runners: Arc::new(runners.clone()),
            ..self.clone()
        }
    }

    pub fn equals(&self, value: &str) -> Self {
        let mut runners = VecDeque::from(self.runners.as_ref().clone());
        runners.push_back(Runner::Compare(Comparator::Equals(value.to_string())));

        Self {
            runners: Arc::new(runners.clone()),
            ..self.clone()
        }
    }

    pub fn not_equals(&self, value: &str) -> Self {
        let mut runners = VecDeque::from(self.runners.as_ref().clone());
        runners.push_back(Runner::Compare(Comparator::NotEquals(value.to_string())));

        Self {
            runners: Arc::new(runners.clone()),
            ..self.clone()
        }
    }

    pub fn _in(&self, value: &[String]) -> Self {
        let mut runners = VecDeque::from(self.runners.as_ref().clone());
        runners.push_back(Runner::Compare(Comparator::In(value.to_vec())));

        Self {
            runners: Arc::new(runners),
            ..self.clone()
        }
    }

    pub fn less_than(&self, value: u64) -> Self {
        let mut runners = VecDeque::from(self.runners.as_ref().clone());
        runners.push_back(Runner::Compare(Comparator::LessThan(value)));

        Self {
            runners: Arc::new(runners),
            ..self.clone()
        }
    }

    pub fn greater_than(&self, value: u64) -> Self {
        let mut runners = VecDeque::from(self.runners.as_ref().clone());
        runners.push_back(Runner::Compare(Comparator::GreaterThan(value)));

        Self {
            runners: Arc::new(runners),
            ..self.clone()
        }
    }

    pub async fn run(&mut self) -> Result<Arc<Vec<ToDo>>, std::io::Error> {
        let runners = VecDeque::from(self.runners.as_ref().clone());
        let mut key_chain: VecDeque<String> = VecDeque::new();

        for r in runners {
            if let Runner::Caller(ref c) = r {
                if let CallerType::Read = c {
                    let todos = self.get_all().await.unwrap();
                    self.value = Arc::new(todos);
                    continue;
                }
            }

            if let Runner::Where(ref s) = r {
                key_chain.push_back(s.to_string());
                continue;
            }

            if let Runner::Compare(ref cmp) = r {
                if let Comparator::Equals(to) = cmp {
                    let todos = self.equal_or_not(&to, &mut key_chain, true).await;
                    self.value = Arc::new(todos);
                }

                if let Comparator::NotEquals(to) = cmp {
                    let todos = self.equal_or_not(&to, &mut key_chain, false).await;
                    self.value = Arc::new(todos);
                }

                if let Comparator::In(list) = cmp {
                    let key_chain = key_chain.pop_back().unwrap();
                    let todos = self.contains(&list, &key_chain).await;
                    self.value = Arc::new(todos);
                }

                if let Comparator::LessThan(number) = cmp {
                    let todos = self.less_or_greater(number, &mut key_chain, true).await;
                    self.value = Arc::new(todos);
                }

                if let Comparator::GreaterThan(number) = cmp {
                    let todos = self.less_or_greater(number, &mut key_chain, false).await;
                    self.value = Arc::new(todos);
                }
            }

            if let Runner::Done = r {
                return Ok(self.value.clone());
            }
        }

        Ok(self.value.clone())
    }

    // Runners
    async fn equal_or_not(
        &self,
        to: &str,
        key_chain: &mut VecDeque<String>,
        flag: bool,
    ) -> Vec<ToDo> {
        let todos = self.value.as_ref().clone();
        let key = key_chain.pop_back().unwrap();

        todos
            .iter()
            .filter(|data| {
                let value: Value = get_nested_value(data, &key).unwrap();
                match value {
                    Value::String(str_val) => {
                        if flag {
                            to == str_val
                        } else {
                            to != str_val
                        }
                    }
                    _ => to == value,
                }
            })
            .cloned()
            .collect::<Vec<ToDo>>()
    }

    async fn contains(&self, list: &[String], key_chain: &str) -> Vec<ToDo> {
        let todos = self.get_all().await.unwrap();
        todos
            .iter()
            .filter(|data| {
                let value: Value = get_nested_value(data, key_chain).unwrap();
                match value {
                    Value::String(str_val) => list.contains(&str_val),
                    _ => list.contains(&String::from("")),
                }
            })
            .cloned()
            .collect::<Vec<ToDo>>()
    }

    async fn less_or_greater(
        &self,
        number: &u64,
        key_chain: &mut VecDeque<String>,
        flag: bool,
    ) -> Vec<ToDo> {
        let todos = self.value.as_ref().clone();

        let key = key_chain.pop_back().unwrap();

        todos
            .iter()
            .filter(|data| {
                let value: Value = get_nested_value(data, &key).unwrap();
                match value {
                    Value::Number(num_val) => {
                        if let Some(n) = num_val.as_u64() {
                            if flag {
                                n < *number
                            } else {
                                n > *number
                            }
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            })
            .cloned()
            .collect::<Vec<ToDo>>()
    }
}
