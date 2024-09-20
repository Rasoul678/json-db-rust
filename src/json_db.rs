use crate::types::{JsonContent, Status, ToDo};
use std::io::{self, ErrorKind};
use std::path::PathBuf;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct JsonDB {
    name: String,
    path: PathBuf,
    file: File,
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

        let db: JsonDB = Self {
            name: db_name.to_string(),
            path,
            file: json_db_file,
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

    pub async fn get_all(&self) -> Result<Vec<ToDo>, io::Error> {
        let content = self.read().await?;
        Ok(content.todos)
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

    pub async fn get_by_id(&self, id: &str) -> Result<ToDo, io::Error> {
        let content = self.read().await?;
        let todo = content
            .todos
            .into_iter()
            .find(|todo| todo.id == id)
            .ok_or(io::Error::new(ErrorKind::NotFound, "Record not found"))?;
        Ok(todo)
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

    pub async fn delete_completed(&self) -> Result<(), io::Error> {
        let mut content = self.read().await?;
        content.todos.retain(|todo| match todo.status {
            Status::Completed => false,
            _ => true,
        });

        self.save(content).await?;
        Ok(())
    }

    pub async fn delete_archived(&self) -> Result<(), io::Error> {
        let mut content = self.read().await?;
        content.todos.retain(|todo| match todo.status {
            Status::Archived => false,
            _ => true,
        });

        self.save(content).await?;
        Ok(())
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

    pub fn pipe(self) -> Self {
        self
    }
}
