use std::io::{self, ErrorKind};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct JsonDB {
    name: String,
    path: String,
    file: File,
}

#[derive(Debug)]

pub enum Status {
    Pending,
    Completed,
    Archived,
}

#[derive(Debug)]

pub struct User(pub String);

#[derive(Debug)]
pub struct ToDo {
    pub id: String,
    pub name: String,
    pub status: Status,
    pub user: User,
}

#[derive(Debug)]

pub enum Data<'a> {
    SingleTodo(&'a ToDo),
    ListOfTodo(Vec<ToDo>),
}

impl JsonDB {
    pub async fn new(db_name: &str) -> Result<JsonDB, io::Error> {
        let path = format!("{}.json", db_name);

        //? Try to open json file Or create a new one!
        let mut json_db_file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .await
        {
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
        json_db_file.write_all(b"[]").await?;

        Ok(Self {
            name: db_name.to_string(),
            path,
            file: json_db_file,
        })
    }

    pub async fn insert(&self, item: ToDo) -> Result<Vec<ToDo>, io::Error> {
        let constnt = self.read().await?;
        println!("{:?}", constnt);

        Ok(vec![item])
    }

    async fn read(&self) -> Result<String, io::Error> {
        let mut file = File::open(&self.path).await?;
        let mut contents = String::new();

        file.read_to_string(&mut contents).await?;
        Ok(contents)
    }

    pub async fn save(&mut self, data: Data<'_>) -> Result<(), io::Error> {
        let data = match data {
            Data::SingleTodo(todo) => format!("{todo:?}"),
            Data::ListOfTodo(todos) => format!("{todos:?}"),
        };

        let mut file = OpenOptions::new().write(true).open(&self.path).await?;
        file.write_all(data.as_bytes()).await?;

        Ok(())
    }
}
