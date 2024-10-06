#![allow(dead_code)]

use crate::get_nested_value;
use crate::types::{JsonContent, ToDo};
use colored::{customcolors::CustomColor, *};
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
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
    Create(String, ToDo, bool),
    Read(String),
    Update(String, ToDo),
    Delete(String),
}

impl MethodName {
    fn notify(&self) {
        let teal = CustomColor::new(0, 201, 217);
        let gold = CustomColor::new(251, 190, 13);
        let green = CustomColor::new(8, 171, 112);
        let yellow = CustomColor::new(242, 140, 54);
        let red = CustomColor::new(217, 33, 33);

        match self {
            MethodName::Create(table, item, _) => println!(
                "{lead} {} {trail}\n\n {} \n",
                table.custom_color(gold).bold(),
                item,
                lead = "🌱 Creating a new record in".custom_color(green).bold(),
                trail = "table...".custom_color(green).bold()
            ),
            MethodName::Read(table) => println!(
                "{lead} {} {trail}\n",
                table.custom_color(gold).bold(),
                lead = "🔎 Querying".custom_color(teal).bold(),
                trail = "table...".custom_color(teal).bold()
            ),
            MethodName::Update(table, item) => println!(
                "{lead} {} {trail}\n\n {} \n",
                table.custom_color(gold).bold(),
                item,
                lead = "⛁ Updating a record in".custom_color(yellow).bold(),
                trail = "table...".custom_color(yellow).bold()
            ),
            MethodName::Delete(table) => println!(
                "{lead} {} {trail}\n",
                table.custom_color(gold).bold(),
                lead = "✗ Deleting records from".custom_color(red).bold(),
                trail = "table...".custom_color(red).bold()
            ),
        }
    }
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
    tables: HashSet<String>,
    path: PathBuf,
    file: Arc<File>,
    value: Arc<HashMap<String, HashSet<ToDo>>>,
    runners: Arc<VecDeque<Runner>>,
}

impl JsonDB {
    pub async fn new() -> Result<JsonDB, io::Error> {
        let dir_path = std::env::current_dir()?;
        let file_path = dir_path.join("db.json");

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)
            .await?;

        let mut content = String::new();

        file.try_clone().await?.read_to_string(&mut content).await?;
        // let mut value = HashMap::new();

        let value = if content.is_empty() {
            HashMap::new()
        } else {
            serde_json::from_str(&content).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?
        };

        let db = JsonDB {
            tables: HashSet::new(),
            path: file_path,
            file: Arc::new(file),
            value: Arc::new(value),
            runners: Arc::new(VecDeque::new()),
        };

        Ok(db)
    }

    fn get_table_mut(&mut self, table_name: &str) -> Result<&mut HashSet<ToDo>, io::Error> {
        let table = Arc::make_mut(&mut self.value)
            .get_mut(table_name)
            .ok_or_else(|| {
                println!(
                    "{} {} \"{}\" {}\n\t\t{} {}\n",
                    "(get_table_mut)".bright_cyan().bold(),
                    "✗ Retrieving".bright_red().bold(),
                    table_name.to_string().bright_red().bold(),
                    "table failed!".bright_red().bold(),
                    "✔".bright_green().bold().blink(),
                    "Tipp: Add a table first!".bright_green().bold()
                );
                io::Error::new(
                    ErrorKind::NotFound,
                    format!("Table '{}' not found", table_name),
                )
            })?;

        Ok(table)
    }

    fn get_table_vec(&mut self, table_name: &str) -> Result<Vec<ToDo>, io::Error> {
        let hash_table = (*self.value)
            .clone()
            .get(table_name)
            .map(Clone::clone)
            .ok_or_else(|| {
                io::Error::new(
                    ErrorKind::NotFound,
                    format!("Table '{}' not found", table_name),
                )
            })?;

        let table = Vec::from_iter(hash_table);

        Ok(table)
    }

    fn add_table(&mut self, table_name: &str) -> Result<(), io::Error> {
        let value = Arc::make_mut(&mut self.value);

        if !value.contains_key(table_name) {
            value.insert(table_name.to_string(), HashSet::new());
            self.tables.insert(table_name.to_string());
        }

        Ok(())
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

    /// Inserts a new record into the JSON database table.
    ///
    /// # Arguments
    ///
    /// * `table` - The name of the table to insert the record into.
    /// * `item` - The `ToDo` item to insert.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `JsonDb` instance, allowing for method chaining.
    pub fn insert(&mut self, table: &str, item: ToDo) -> &mut Self {
        Arc::make_mut(&mut self.runners).push_back(Runner::Method(MethodName::Create(
            table.to_string(),
            item,
            false,
        )));
        self
    }

    /// Inserts a new record into the JSON database table,
    /// or creates a table first if it does not already exists.
    ///
    /// # Arguments
    ///
    /// * `table` - The name of the table to insert the record into.
    /// * `item` - The `ToDo` item to insert or update.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `JsonDb` instance, allowing for method chaining.
    pub fn insert_or(&mut self, table: &str, item: ToDo) -> &mut Self {
        Arc::make_mut(&mut self.runners).push_back(Runner::Method(MethodName::Create(
            table.to_string(),
            item,
            true,
        )));
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
    pub fn find(&mut self, table: &str) -> &mut Self {
        Arc::make_mut(&mut self.runners)
            .push_back(Runner::Method(MethodName::Read(table.to_string())));

        self
    }

    /// Adds a `Runner::Method(MethodName::Update)` to the end of the runners queue, indicating that the current operation is an update operation.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn update(&mut self, table: &str, data: ToDo) -> &mut Self {
        Arc::make_mut(&mut self.runners)
            .push_back(Runner::Method(MethodName::Update(table.to_string(), data)));

        self
    }

    /// Adds a `Runner::Method(MethodName::Delete(c))` to the end of the runners queue,
    /// indicating that the current operation is a delete operation.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Arguments
    ///
    /// * `key` - The character to use for the delete operation.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn delete(&mut self, table: &str) -> &mut Self {
        Arc::make_mut(&mut self.runners)
            .push_back(Runner::Method(MethodName::Delete(table.to_string())));

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

    /// Adds a `Runner::Compare(Comparator::Between((start, end)))` to the end of the runners queue, filtering the data based on the provided start and end values.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Arguments
    ///
    /// * `start` - The start value to filter the data by.
    /// * `end` - The end value to filter the data by.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn between(&mut self, start: u64, end: u64) -> &mut Self {
        Arc::make_mut(&mut self.runners)
            .push_back(Runner::Compare(Comparator::Between((start, end))));

        self
    }

    /// Runs the database operations specified in the runners queue.
    ///
    /// This method processes the runners queue, performing various database operations such as creating, reading, updating, and deleting records.
    /// The method returns the resulting list of `ToDo` items after applying the specified operations.
    ///
    /// # Errors
    ///
    /// This method may return an `std::io::Error` if there is an error saving the database state after the operations are completed.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec` of `ToDo` items representing the final state of the database after the operations have been performed.
    pub async fn run(&mut self) -> Result<Vec<ToDo>, std::io::Error> {
        let mut result = Vec::new();
        let mut key_chain = String::new();
        let mut method: Option<MethodName> = None;

        Arc::make_mut(&mut self.runners).push_back(Runner::Done);

        while let Some(runner) = Arc::make_mut(&mut self.runners).pop_front() {
            match runner {
                Runner::Method(name) => match name {
                    MethodName::Create(table, new_item, or) => {
                        result = self.get_table_vec(&table).unwrap_or_default();
                        method = Some(MethodName::Create(table, new_item.clone(), or));
                    }
                    MethodName::Read(table) => {
                        result = self.get_table_vec(&table).unwrap_or_default();
                        method = Some(MethodName::Read(table));
                    }
                    _ => {
                        method = Some(name);
                    }
                },
                Runner::Where(f) => {
                    key_chain = f;
                }
                Runner::Compare(ref comparator) => {
                    result = result
                        .into_iter()
                        .filter(|todo| {
                            let value: Value = get_nested_value(todo, &key_chain).unwrap();
                            self.filter_with_conmpare(value, comparator)
                        })
                        .collect();
                }
                Runner::Done => {
                    match method {
                        Some(MethodName::Read(table)) => {
                            MethodName::Read(table).notify();
                        }
                        Some(MethodName::Create(table, ref new_item, or)) => {
                            self.insert_into_table(table.as_str(), &new_item, or)?;
                            MethodName::Create(table, new_item.clone(), or).notify();
                        }
                        Some(MethodName::Update(table, new_todo)) => {
                            // let search_result =
                            //     result
                            //         .iter()
                            //         .find(|t| t.id == new_todo.id)
                            //         .ok_or(io::Error::new(
                            //             ErrorKind::NotFound,
                            //             format!(
                            //                 "Schade! Record with id \"{}\" not found!",
                            //                 new_todo.id
                            //             ),
                            //         ));

                            // match search_result {
                            //     Ok(old_todo) => {
                            //         MethodName::Update(new_todo.to_owned()).notify(&self.name);

                            //         Arc::make_mut(&mut self.value).retain(|t| t.id != old_todo.id);
                            //         Arc::make_mut(&mut self.value).insert(new_todo.clone());

                            //         result.clear();
                            //         result.insert(new_todo);
                            //     }
                            //     Err(err) => {
                            //         println!(
                            //             "{} {}\n{}",
                            //             "Updating error:".bright_red().bold(),
                            //             err.to_string().bright_cyan().bold(),
                            //             "Tipp: Consider adding new record!".bright_green().bold()
                            //         );
                            //         return Err(err);
                            //     }
                            // };
                        }
                        Some(MethodName::Delete(table)) => {
                            // for t in result.iter() {
                            //     Arc::make_mut(&mut self.value).retain(|todo| todo.id != t.id);
                            // }

                            MethodName::Delete(table).notify();
                        }
                        _ => {}
                    }

                    self.save().await?;

                    break;
                }
            }
        }

        // let result_vec = result.iter().cloned().collect::<Vec<ToDo>>();
        Ok(result)
    }

    /// Filters a `Value` based on the provided `Comparator`.
    ///
    /// This function takes a `Value` and a `Comparator` and returns a boolean indicating whether the `Value` matches the comparison criteria.
    ///
    /// # Examples
    ///
    /// use serde_json::Value;
    /// use json_db::Comparator;
    ///
    /// let json_db = JsonDB::new();
    /// let value = Value::from(42u64);
    /// let comparator = Comparator::GreaterThan(30);
    /// assert!(json_db.filter_with_conmpare(value, &comparator));
    ///
    fn filter_with_conmpare(&self, value: Value, comparator: &Comparator) -> bool {
        match comparator {
            Comparator::Equals(v) => value.as_str() == Some(v.as_str()),
            Comparator::NotEquals(v) => value.as_str() != Some(v.as_str()),
            Comparator::LessThan(v) => value.as_u64().map_or(false, |x| x < *v),
            Comparator::GreaterThan(v) => value.as_u64().map_or(false, |x| x > *v),
            Comparator::In(vs) => value
                .as_str()
                .map_or(false, |x| vs.contains(&x.to_string())),
            Comparator::Between((start, end)) => {
                value.as_u64().map_or(false, |x| x >= *start && x <= *end)
            }
        }
    }

    /// Inserts a new item into the records of the JsonDB.
    ///
    /// This method takes a reference to a `ToDo` item and attempts to insert it into the records of the JsonDB. It first checks if the new item already exists in the database by iterating through the existing `ToDo` items and comparing their IDs. If the new item's ID matches an existing item, an `io::Error` with the `ErrorKind::AlreadyExists` error kind is returned. Otherwise, the new item is inserted into the records and a reference to the new item is returned.
    fn insert_into_table<'a>(
        &mut self,
        table_name: &str,
        new_item: &'a ToDo,
        or: bool,
    ) -> Result<&'a ToDo, io::Error> {
        let table = if or {
            let db_hash = Arc::make_mut(&mut self.value);

            match db_hash.get_mut(table_name) {
                Some(t) => t,
                None => {
                    self.tables.insert(table_name.to_string());
                    db_hash.insert(table_name.to_string(), HashSet::new());
                    db_hash.get_mut(table_name).unwrap()
                }
            }
        } else {
            self.get_table_mut(table_name)?
        };

        // Check if the new item already exists in the set for exact same properties
        if table.contains(new_item) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Record already exists",
            ));
        }

        // TODO: check for double entries with same id

        // Insert the new item
        table.insert(new_item.clone());

        Ok(new_item)
    }
}
