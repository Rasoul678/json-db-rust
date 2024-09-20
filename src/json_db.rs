use crate::types::{Data, JsonContent, ToDo};
use std::io::{self, ErrorKind};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct JsonDB {
    name: String,
    path: String,
    file: File,
}

impl JsonDB {
    pub async fn new(db_name: &str) -> Result<JsonDB, io::Error> {
        let path = format!("{}.json", db_name);

        //? Try to open json file Or create a new one!
        let mut json_db_file = match File::open(&path).await {
            Ok(file) => file,
            Err(error) => {
                if error.kind() == ErrorKind::NotFound {
                    File::create(&path).await?
                } else {
                    return Err(error);
                }
            }
        };

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

    async fn save(&self, data: Data) -> Result<(), io::Error> {
        let mut content = self.read().await?;

        match data {
            Data::SingleTodo(todo) => {
                for t in content.todos.iter() {
                    if t.id == todo.id {
                        return Err(io::Error::new(
                            ErrorKind::AlreadyExists,
                            "Record already exists",
                        ));
                    }
                }

                content.todos.push(todo);
            }
            Data::ListOfTodo(todos) => content.todos = todos,
        };

        let mut file = OpenOptions::new().write(true).open(&self.path).await?;
        file.write_all(serde_json::to_string_pretty(&content)?.as_bytes())
            .await?;

        Ok(())
    }

    pub async fn insert<'a>(&'a self, item: &'a ToDo) -> Result<&'a ToDo, io::Error> {
        let data = Data::SingleTodo(item.clone());
        self.save(data).await?;
        Ok(item)
    }
}
