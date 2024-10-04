#![allow(dead_code)]

use crate::get_nested_value;
use crate::types::{JsonContent, ToDo};
use colored::{customcolors::CustomColor, *};
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
        let value = HashSet::new();

        // let value: HashSet<ToDo> = if content.is_empty() {
        //     HashSet::new()
        // } else {
        //     serde_json::from_str(&content).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?
        // };

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

    /// Runs the database operations specified in the runners queue.
    ///
    /// This method processes the runners queue, performing various database operations such as creating, reading, updating, and deleting records.
    /// The method returns the resulting set of `ToDo` items after applying the specified operations.
    ///
    /// # Errors
    ///
    /// This method may return an `std::io::Error` if there is an error saving the database state after the operations are completed.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `HashSet` of `ToDo` items representing the final state of the database after the operations have been performed.
    pub async fn run(&mut self) -> Result<HashSet<ToDo>, std::io::Error> {
        let mut result = (*self.value).clone();
        let mut key_chain = String::new();
        let mut method: Option<MethodName> = None;
        Arc::make_mut(&mut self.runners).push_back(Runner::Done);

        while let Some(runner) = Arc::make_mut(&mut self.runners).pop_front() {
            match runner {
                Runner::Method(name) => match name {
                    MethodName::Create(new_item) => {
                        method = Some(MethodName::Create(new_item.clone()));
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
                    let teal = CustomColor::new(0, 201, 217);
                    let gold = CustomColor::new(251, 190, 13);
                    let green = CustomColor::new(8, 171, 112);
                    let yellow = CustomColor::new(242, 140, 54);
                    let red = CustomColor::new(217, 33, 33);

                    match method {
                        Some(MethodName::Read) => {
                            println!(
                                "{lead} {} {trail}",
                                self.name.custom_color(gold).bold(),
                                lead = "Querying".custom_color(teal).bold(),
                                trail = "database...".custom_color(teal).bold()
                            )
                        }
                        Some(MethodName::Create(ref new_item)) => {
                            self.insert_into_records(&new_item)?;

                            println!(
                                "{lead} {} {trail}\n {} \n",
                                self.name.custom_color(gold).bold(),
                                new_item,
                                lead = "Creating new record in".custom_color(green).bold(),
                                trail = "database...".custom_color(green).bold()
                            );
                        }
                        Some(MethodName::Update) => {
                            println!(
                                "{lead} {} {middle} {} {trail}",
                                result.len().to_string().custom_color(teal).bold(),
                                self.name.custom_color(gold).bold(),
                                lead = "Updating".custom_color(yellow).bold(),
                                middle = "records in".custom_color(yellow).bold(),
                                trail = "database...".custom_color(yellow).bold()
                            );
                        }

                        Some(MethodName::Delete) => {
                            for t in result.iter() {
                                Arc::make_mut(&mut self.value).retain(|todo| todo.id != t.id);
                            }

                            let mut lenght = result.len().to_string();

                            if self.value.is_empty() {
                                lenght = "all".to_string();
                            }

                            println!(
                                "{lead} {} {middle} {} {trail}",
                                lenght.custom_color(teal).bold(),
                                self.name.custom_color(gold).bold(),
                                lead = "Deleting".custom_color(red).bold(),
                                middle = "records from".custom_color(red).bold(),
                                trail = "database...".custom_color(red).bold()
                            );
                        }
                        _ => {}
                    }

                    self.save().await?;

                    break;
                }
            }
        }

        Ok(result)
    }

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

    fn insert_into_records<'a>(&mut self, new_item: &'a ToDo) -> Result<&'a ToDo, io::Error> {
        let mut todos = self
            .value
            .iter()
            .map(Clone::clone)
            .collect::<VecDeque<ToDo>>();

        //* Check if the new item already exists in the database
        //* If so, you know what to do (just kidding 😉)
        //* If it does, return an error
        let so_weit_so_gut = Arc::make_mut(&mut self.value).insert(new_item.clone());

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

        Ok(new_item)
    }
}
