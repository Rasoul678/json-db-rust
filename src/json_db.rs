#![allow(dead_code)]

use crate::get_nested_value;
use crate::types::{JsonContent, ToDo};
use serde_json::Value;
use std::collections::{HashSet, VecDeque};
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
enum MethodName {
    Create(ToDo),
    Read,
    Update,
    Delete,
}

#[derive(Clone, PartialEq, Debug)]
enum Runner {
    Done,
    Method(MethodName),
    Compare(Comparator),
    Where(String),
}

#[derive(Clone)]
pub struct JsonDB {
    name: String,
    path: PathBuf,
    file: Arc<File>,
    value: Arc<HashSet<ToDo>>,
    runners: Arc<VecDeque<Runner>>,
}

impl JsonDB {
    pub async fn new(db_name: &str) -> Result<JsonDB, io::Error> {
        let dir_path = std::env::current_dir()?;
        let file_with_format = format!("{}.json", db_name);
        let file_path = dir_path.join(file_with_format);

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)
            .await?;

        let mut content = String::new();

        file.try_clone().await?.read_to_string(&mut content).await?;

        let value: HashSet<ToDo> = if content.is_empty() {
            HashSet::new()
        } else {
            serde_json::from_str(&content).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?
        };

        let db = JsonDB {
            name: db_name.to_string(),
            path: file_path,
            file: Arc::new(file),
            value: Arc::new(value),
            runners: Arc::new(VecDeque::new()),
        };

        Ok(db)
    }

    pub async fn save(&self) -> Result<(), io::Error> {
        let json = serde_json::to_string_pretty(&*self.value)
            .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .await?;

        file.write_all(json.as_bytes()).await?;
        file.flush().await?;

        Ok(())
    }
    async fn read_content(&self) -> Result<JsonContent, io::Error> {
        let mut file = OpenOptions::new().read(true).open(&self.path).await?;
        let mut content = String::new();
        file.read_to_string(&mut content).await?;
        let json_data: JsonContent = serde_json::from_str(&content)?;
        Ok(json_data)
    }
    async fn save_content(&self, content: JsonContent) -> Result<(), io::Error> {
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

        file.write_all(b"").await?;

        file.flush().await?;

        Ok(())
    }

    pub fn insert(&mut self, item: ToDo) -> &mut Self {
        Arc::make_mut(&mut self.runners).push_back(Runner::Method(MethodName::Create(item)));
        self
    }

    pub async fn get_all(&self) -> Result<Vec<ToDo>, io::Error> {
        let content = self.read_content().await?;
        Ok(content.records)
    }

    /// Updates an existing record in the JSON database.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the record to update.
    /// * `todo` - The updated `ToDo` item.
    ///
    /// # Returns
    ///
    /// The updated `ToDo` item if the operation was successful, or an `io::Error` if the record was not found or there was an error.
    pub async fn update_todo(&self, id: &str, todo: ToDo) -> Result<ToDo, io::Error> {
        let mut content = self.read_content().await?;
        let todo_index = content
            .records
            .iter()
            .position(|todo| todo.id == id)
            .ok_or(io::Error::new(ErrorKind::NotFound, "Record not found"))?;
        content.records[todo_index] = todo.clone();
        self.save_content(content).await?;
        Ok(todo)
    }

    /// Removes all records from the JSON database.
    ///
    /// # Returns
    ///
    /// A `Result` that is `Ok(())` if the operation was successful, or an `Err(io::Error)` if there was an error.
    pub async fn delete_all(&self) -> Result<(), io::Error> {
        self.clear().await
    }

    /// Adds a `Runner::Method(MethodName::Read)` to the end of the runners queue, indicating that the current operation is a read operation.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn find(&mut self) -> &mut Self {
        Arc::make_mut(&mut self.runners).push_back(Runner::Method(MethodName::Read));

        self
    }

    /// Adds a `Runner::Method(MethodName::Update)` to the end of the runners queue, indicating that the current operation is an update operation.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn update(&mut self) -> &mut Self {
        Arc::make_mut(&mut self.runners).push_back(Runner::Method(MethodName::Update));

        self
    }

    /// Adds a `Runner::Caller(CallerType::Delete)` to the end of the runners queue, indicating that the current operation is a delete operation.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn delete(&mut self) -> &mut Self {
        Arc::make_mut(&mut self.runners).push_back(Runner::Method(MethodName::Delete));

        self
    }

    /// Adds a `Runner::Where(field.to_string())` to the end of the runners queue, filtering the data based on the provided field.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Arguments
    ///
    /// * `field` - The field to filter the data by.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn where_(&mut self, field: &str) -> &mut Self {
        Arc::make_mut(&mut self.runners).push_back(Runner::Where(field.to_string()));

        self
    }

    /// Adds a `Runner::Compare(Comparator::Equals(value.to_string()))` to the end of the runners queue, filtering the data based on the provided value.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to filter the data by.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn equals(&mut self, value: &str) -> &mut Self {
        Arc::make_mut(&mut self.runners)
            .push_back(Runner::Compare(Comparator::Equals(value.to_string())));

        self
    }

    /// Adds a `Runner::Compare(Comparator::NotEquals(value.to_string()))` to the end of the runners queue, filtering the data based on the provided value.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to filter the data by.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn not_equals(&mut self, value: &str) -> &mut Self {
        Arc::make_mut(&mut self.runners)
            .push_back(Runner::Compare(Comparator::NotEquals(value.to_string())));

        self
    }

    /// Adds a `Runner::Compare(Comparator::In(value.to_vec()))` to the end of the runners queue, filtering the data based on the provided values.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Arguments
    ///
    /// * `value` - The values to filter the data by.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn in_(&mut self, values: Vec<String>) -> &mut Self {
        Arc::make_mut(&mut self.runners).push_back(Runner::Compare(Comparator::In(values)));

        self
    }

    /// Adds a `Runner::Compare(Comparator::LessThan(value))` to the end of the runners queue, filtering the data based on the provided value.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to filter the data by.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn less_than(&mut self, value: u64) -> &mut Self {
        Arc::make_mut(&mut self.runners).push_back(Runner::Compare(Comparator::LessThan(value)));

        self
    }

    /// Adds a `Runner::Compare(Comparator::GreaterThan(value))` to the end of the runners queue, filtering the data based on the provided value.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to filter the data by.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn greater_than(&mut self, value: u64) -> &mut Self {
        Arc::make_mut(&mut self.runners).push_back(Runner::Compare(Comparator::GreaterThan(value)));

        self
    }

    /// Adds a `Runner::Compare(Comparator::Between(range))` to the end of the runners queue, filtering the data based on the provided range.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Arguments
    ///
    /// * `range` - The range to filter the data by, specified as a tuple of two `u64` values.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn between(&mut self, start: u64, end: u64) -> &mut Self {
        Arc::make_mut(&mut self.runners)
            .push_back(Runner::Compare(Comparator::Between((start, end))));

        self
    }

    /// Runs the query defined by the `runners` queue, filtering and transforming the data as specified.
    /// The final result is stored in the `value` field of the `Self` instance.
    ///
    /// This method iterates through the `runners` queue and applies the corresponding filtering or transformation logic.
    /// The `key_chain` is used to keep track of the nested keys being accessed for the current operation.
    /// The final result is returned as an `Arc<Vec<ToDo>>`.
    ///
    /// # Errors
    ///
    /// This method can return an `std::io::Error` if there is an issue reading or processing the data.
    pub async fn run(&mut self) -> Result<HashSet<ToDo>, std::io::Error> {
        let mut result = (*self.value).clone();
        let mut field = String::new();
        Arc::make_mut(&mut self.runners).push_back(Runner::Done);

        while let Some(runner) = Arc::make_mut(&mut self.runners).pop_front() {
            match runner {
                Runner::Method(name) => {
                    match name {
                        MethodName::Create(new_item) => {
                            let mut todos = self
                                .value
                                .iter()
                                .map(Clone::clone)
                                .collect::<VecDeque<ToDo>>();

                            //* Check if the new item already exists in the database
                            //* If so, you know what to do (just kidding 😉)
                            //* If it does, return an error
                            let so_weit_so_gut =
                                Arc::make_mut(&mut self.value).insert(new_item.clone());

                            //* Make sure that id is unique
                            if so_weit_so_gut {
                                while let Some(todo) = todos.pop_front() {
                                    if todo.id == new_item.id {
                                        return Err(io::Error::new(
                                            ErrorKind::AlreadyExists,
                                            "Record already exists",
                                        ));
                                    }
                                }
                            }
                        }
                        MethodName::Read => {
                            // TODO: Implementation for Read
                        }
                        MethodName::Update => {
                            // TODO: Implementation for Update
                        }
                        MethodName::Delete => {
                            // TODO: Implementation for Delete
                        }
                    }

                    self.save().await?;
                }
                Runner::Where(f) => {
                    field = f;
                }
                Runner::Compare(ref comparator) => {
                    result = result
                        .into_iter()
                        .filter(|todo| {
                            let value: Value = get_nested_value(todo, &field).unwrap();
                            match comparator {
                                Comparator::Equals(v) => value.as_str() == Some(v.as_str()),
                                Comparator::NotEquals(v) => value.as_str() != Some(v.as_str()),
                                Comparator::LessThan(v) => value.as_u64().map_or(false, |x| x < *v),
                                Comparator::GreaterThan(v) => {
                                    value.as_u64().map_or(false, |x| x > *v)
                                }
                                Comparator::In(vs) => value
                                    .as_str()
                                    .map_or(false, |x| vs.contains(&x.to_string())),
                                Comparator::Between((start, end)) => {
                                    value.as_u64().map_or(false, |x| x >= *start && x <= *end)
                                }
                            }
                        })
                        .collect();
                }
                Runner::Done => break,
            }
        }

        Ok(result)
    }

    //*! comparators
    /// Finds a subset of `ToDo` items from the `value` field based on the provided `is` boolean and `to` string.
    ///
    /// The `key_chain` parameter is used to access nested values within the `ToDo` items. The `find_if` function
    /// iterates over the `ToDo` items, extracts the value at the specified `key`, and compares it to the `to`
    /// parameter based on the `is` boolean. The resulting `ToDo` items are collected and returned.
    ///
    /// # Arguments
    /// * `is` - A boolean indicating whether to find items where the value matches or does not match the `to` parameter.
    /// * `to` - The string to compare the extracted values against.
    /// * `key_chain` - A mutable `VecDeque` containing the keys to access nested values within the `ToDo` items.
    ///
    /// # Returns
    /// A `Vec<ToDo>` containing the subset of `ToDo` items that match the provided criteria.
    async fn find_if(&self, is: bool, to: &str, key_chain: &mut VecDeque<String>) -> Vec<ToDo> {
        let todos = self.value.as_ref().clone();
        let key = key_chain.pop_back().unwrap();

        todos
            .iter()
            .filter(|data| {
                let value: Value = get_nested_value(data, &key).unwrap();
                match value {
                    Value::String(str_val) => {
                        if is {
                            to == str_val
                        } else if !is {
                            to != str_val
                        } else {
                            false
                        }
                    }
                    _ => to == value.to_string(),
                }
            })
            .cloned()
            .collect::<Vec<ToDo>>()
    }

    /// Finds a subset of `ToDo` items from the `value` field where the value at the specified `key` is contained in the provided `list`.
    ///
    /// The `key_chain` parameter is used to access nested values within the `ToDo` items. The `enthalten` function
    /// iterates over the `ToDo` items, extracts the value at the specified `key`, and checks if it is contained in the `list`.
    /// The resulting `ToDo` items are collected and returned.
    ///
    /// # Arguments
    /// * `list` - A slice of `String` values to check for containment.
    /// * `key_chain` - A mutable `VecDeque` containing the keys to access nested values within the `ToDo` items.
    ///
    /// # Returns
    /// A `Vec<ToDo>` containing the subset of `ToDo` items where the value at the specified `key` is contained in the `list`.
    async fn contains(&self, list: &[String], key_chain: &mut VecDeque<String>) -> Vec<ToDo> {
        let todos = self.value.as_ref().clone();
        let key = key_chain.pop_back().unwrap();

        todos
            .iter()
            .filter(|data| {
                let value: Value = get_nested_value(data, &key).unwrap();
                match value {
                    Value::String(str_val) => list.contains(&str_val),
                    _ => list.contains(&String::from("")),
                }
            })
            .cloned()
            .collect::<Vec<ToDo>>()
    }

    async fn check_if(
        &self,
        number: &u64,
        is: bool,
        key_chain: &mut VecDeque<String>,
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
                            if is {
                                n < *number
                            } else if !is {
                                n > *number
                            } else {
                                false
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

    /// Retrieves a vector of `ToDo` items from the JSON database where the value of the specified key is within the given numeric range.
    ///
    /// This function takes a mutable reference to a `VecDeque` of strings representing the key chain to access the value, and a tuple of two `u64` values representing the inclusive range.
    ///
    /// # Arguments
    /// * `range` - A tuple of two `u64` values representing the inclusive range to filter the values by.
    /// * `key_chain` - A mutable reference to a `VecDeque` of strings representing the key chain to access the value to be compared.
    ///
    /// # Returns
    /// A `Vec` of `ToDo` items that have a numeric value within the specified range for the given key chain.
    async fn between_range(
        &self,
        range: &(u64, u64),
        key_chain: &mut VecDeque<String>,
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
                            n >= range.0 && n <= range.1
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

    /// Deletes a set of `ToDo` items from the JSON database based on a string comparison.
    ///
    /// This function takes a boolean flag `is`, a string `to`, and a mutable `VecDeque` of strings `key_chain`. It reads the current content of the JSON database, filters the `ToDo` items based on the provided parameters, and then saves the updated content back to the database. The function returns a `Result` containing the deleted `ToDo` items.
    ///
    /// # Arguments
    /// * `is` - A boolean flag indicating whether to delete items where the value matches `to` or where it does not match.
    /// * `to` - A string to compare the value against.
    /// * `key_chain` - A mutable `VecDeque` of strings representing the key chain to access the value to be compared.
    ///
    /// # Returns
    /// A `Result` containing a `Vec` of `ToDo` items that were deleted.
    async fn delete_if(
        &self,
        is: bool,
        to: &str,
        key_chain: &mut VecDeque<String>,
    ) -> Result<Vec<ToDo>, io::Error> {
        let mut content = self.read_content().await?;
        let key = key_chain.pop_back().unwrap();

        let deleted_vec = content
            .records
            .iter()
            .filter(|todo| {
                let value: Value = get_nested_value(todo, &key).unwrap();
                match value {
                    Value::String(str_val) => {
                        if is {
                            to == str_val
                        } else if !is {
                            to != str_val
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            })
            .cloned()
            .collect::<Vec<ToDo>>();

        content.records.retain(|todo| {
            let value: Value = get_nested_value(todo, &key).unwrap();
            match value {
                Value::String(str_val) => {
                    if is {
                        to != str_val
                    } else if !is {
                        to == str_val
                    } else {
                        true
                    }
                }
                _ => true,
            }
        });

        self.save_content(content).await?;

        Ok(deleted_vec)
    }
}
