#![allow(dead_code)]

use crate::get_nested_value;
use crate::types::{JsonContent, ToDo};
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
        json_db_file
            .write_all(format!("{{\"records\":[]}}").as_bytes())
            .await?;

        let db = Self {
            name: db_name.to_string(),
            path,
            file: json_db_file.into(),
            value: Arc::new(vec![]),
            runners: Arc::new(VecDeque::new()),
        };

        Ok(db)
    }
    async fn lies(&self) -> Result<JsonContent, io::Error> {
        let mut file = OpenOptions::new().read(true).open(&self.path).await?;
        let mut content = String::new();
        file.read_to_string(&mut content).await?;
        let json_data: JsonContent = serde_json::from_str(&content)?;
        Ok(json_data)
    }
    async fn speichere(&self, content: JsonContent) -> Result<(), io::Error> {
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

        file.write_all(b"{\"records\":[]}").await?;

        file.flush().await?;

        Ok(())
    }

    pub async fn eingeben<'a>(&'a self, item: &'a ToDo) -> Result<&'a ToDo, io::Error> {
        let mut content = self.lies().await?;

        for t in content.records.iter() {
            if t.id == item.id {
                return Err(io::Error::new(
                    ErrorKind::AlreadyExists,
                    "Record already exists",
                ));
            }
        }

        content.records.push(item.clone());

        self.speichere(content).await?;

        Ok(item)
    }

    async fn hole_alle(&self) -> Result<Vec<ToDo>, io::Error> {
        let content = self.lies().await?;
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
    pub async fn aktualisiere(&self, id: &str, todo: ToDo) -> Result<ToDo, io::Error> {
        let mut content = self.lies().await?;
        let todo_index = content
            .records
            .iter()
            .position(|todo| todo.id == id)
            .ok_or(io::Error::new(ErrorKind::NotFound, "Record not found"))?;
        content.records[todo_index] = todo.clone();
        self.speichere(content).await?;
        Ok(todo)
    }

    /// Removes all records from the JSON database.
    ///
    /// # Returns
    ///
    /// A `Result` that is `Ok(())` if the operation was successful, or an `Err(io::Error)` if there was an error.
    pub async fn entferne_alle(&self) -> Result<(), io::Error> {
        self.clear().await
    }

    /// Finds the current set of runners and adds a `Runner::Caller(CallerType::Read)` to the end of the queue.
    /// This method is used to indicate that the current operation is a read operation.
    /// The returned `Self` instance contains the updated runners queue.
    pub fn finde(&self) -> Self {
        let mut runners = VecDeque::from(self.runners.as_ref().clone());
        runners.push_back(Runner::Caller(CallerType::Read));

        Self {
            runners: Arc::new(runners.clone()),
            ..self.clone()
        }
    }

    /// Adds a `Runner::Caller(CallerType::Delete)` to the end of the runners queue, indicating that the current operation is a delete operation.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn entferne(&self) -> Self {
        let mut runners = VecDeque::from(self.runners.as_ref().clone());
        runners.push_back(Runner::Caller(CallerType::Delete));

        Self {
            runners: Arc::new(runners.clone()),
            ..self.clone()
        }
    }

    /// Adds a `Runner::Where` to the end of the runners queue, filtering the data based on the provided field.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Arguments
    ///
    /// * `field` - The field to filter the data by.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn wo(&self, field: &str) -> Self {
        let mut runners = VecDeque::from(self.runners.as_ref().clone());
        runners.push_back(Runner::Where(field.to_string()));

        Self {
            runners: Arc::new(runners.clone()),
            ..self.clone()
        }
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
    pub fn entspricht(&self, value: &str) -> Self {
        let mut runners = VecDeque::from(self.runners.as_ref().clone());
        runners.push_back(Runner::Compare(Comparator::Equals(value.to_string())));

        Self {
            runners: Arc::new(runners.clone()),
            ..self.clone()
        }
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
    pub fn nicht_entspricht(&self, value: &str) -> Self {
        let mut runners = VecDeque::from(self.runners.as_ref().clone());
        runners.push_back(Runner::Compare(Comparator::NotEquals(value.to_string())));

        Self {
            runners: Arc::new(runners.clone()),
            ..self.clone()
        }
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
    pub fn is_in(&self, value: &[String]) -> Self {
        let mut runners = VecDeque::from(self.runners.as_ref().clone());
        runners.push_back(Runner::Compare(Comparator::In(value.to_vec())));

        Self {
            runners: Arc::new(runners),
            ..self.clone()
        }
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
    pub fn less_than(&self, value: u64) -> Self {
        let mut runners = VecDeque::from(self.runners.as_ref().clone());
        runners.push_back(Runner::Compare(Comparator::LessThan(value)));

        Self {
            runners: Arc::new(runners),
            ..self.clone()
        }
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
    pub fn greater_than(&self, value: u64) -> Self {
        let mut runners = VecDeque::from(self.runners.as_ref().clone());
        runners.push_back(Runner::Compare(Comparator::GreaterThan(value)));

        Self {
            runners: Arc::new(runners),
            ..self.clone()
        }
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
    pub fn zwischen(&self, range: [u64; 2]) -> Self {
        let mut runners = VecDeque::from(self.runners.as_ref().clone());
        runners.push_back(Runner::Compare(Comparator::Between((range[0], range[1]))));

        Self {
            runners: Arc::new(runners),
            ..self.clone()
        }
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
    pub async fn run(&mut self) -> Result<Arc<Vec<ToDo>>, std::io::Error> {
        let runners = VecDeque::from(self.runners.as_ref().clone());
        let mut key_chain: VecDeque<String> = VecDeque::new();
        let mut method = CallerType::Read;

        for r in runners {
            if let Runner::Caller(ref c) = r {
                if let CallerType::Read = c {
                    let todos = self.hole_alle().await.unwrap();
                    self.value = Arc::new(todos);
                    continue;
                }

                if let CallerType::Delete = c {
                    method = CallerType::Delete;
                    continue;
                }
            }

            if let Runner::Where(ref s) = r {
                key_chain.push_back(s.to_string());
                continue;
            }

            if let Runner::Compare(ref cmp) = r {
                match cmp {
                    Comparator::Equals(to) => {
                        let equal = true;

                        if method == CallerType::Read {
                            let todos = self.find_if(equal, &to, &mut key_chain).await;
                            self.value = Arc::new(todos);
                        }

                        if method == CallerType::Delete {
                            let deleted = self.delete_if(equal, &to, &mut key_chain).await.unwrap();
                            self.value = Arc::new(deleted);
                        }
                    }
                    Comparator::NotEquals(to) => {
                        let not_equal = false;

                        if method == CallerType::Read {
                            let todos = self.find_if(not_equal, &to, &mut key_chain).await;
                            self.value = Arc::new(todos);
                        }

                        if method == CallerType::Delete {
                            let deleted = self
                                .delete_if(not_equal, &to, &mut key_chain)
                                .await
                                .unwrap();
                            self.value = Arc::new(deleted);
                        }
                    }
                    Comparator::In(list) => {
                        let todos = self.enthalten(&list, &mut key_chain).await;
                        self.value = Arc::new(todos);
                    }
                    Comparator::LessThan(number) => {
                        let less_than = true;
                        let todos = self.versuche_ob(number, less_than, &mut key_chain).await;
                        self.value = Arc::new(todos);
                    }
                    Comparator::GreaterThan(number) => {
                        let greater_than = false;
                        let todos = self.versuche_ob(number, greater_than, &mut key_chain).await;
                        self.value = Arc::new(todos);
                    }
                    Comparator::Between(range) => {
                        let todos = self.between_range(range, &mut key_chain).await;
                        self.value = Arc::new(todos);
                    }
                };
            }

            if let Runner::Done = r {
                return Ok(self.value.clone());
            }
        }

        Ok(self.value.clone())
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
    async fn enthalten(&self, list: &[String], key_chain: &mut VecDeque<String>) -> Vec<ToDo> {
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

    async fn versuche_ob(
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
        let mut content = self.lies().await?;
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

        self.speichere(content).await?;

        Ok(deleted_vec)
    }
}
