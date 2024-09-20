use json_db::{fake_todo, JsonDB};

#[tokio::main]
async fn main() {
    let my_db = JsonDB::new("todo").await.unwrap();

    let todos = fake_todo(10);

    for todo in todos {
        let _ = my_db.insert(&todo).await;
    }
}
