use json_db::{Data, JsonDB, Status, ToDo, User};

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let todo = ToDo {
        id: "1".to_string(),
        name: "Go to the Gym".to_string(),
        status: Status::Pending,
        user: User("Rasoul".to_string()),
    };

    let mut my_todo = JsonDB::new("todo").await.unwrap();

    my_todo.save(Data::SingleTodo(&todo)).await.unwrap();

    let cont = my_todo.insert(todo).await.unwrap();
    println!("{:?}", cont);
}
