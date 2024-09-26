#![allow(unused_variables)]

use json_db::{fake_it, Date, JsonDB, Name, Status, ToDo, User};

#[tokio::main]
async fn main() {
    let my_db = JsonDB::new("todo").await.unwrap();

    let todos = fake_it::<ToDo>(10);

    for todo in &todos {
        let _ = my_db.eingeben(&todo).await;
    }

    let my_todo = ToDo {
        id: "100".to_string(),
        text: "Learn Rust".to_string(),
        status: Status::Pending,
        user: User {
            name: Name {
                first: "Rasoul".to_string(),
                last: "Hesami".to_string(),
            },
            email: "rasoul.hesami@gmail.com".to_string(),
        },
        date: Date {
            start: "2023-01-01".to_string(),
            end: "2025-01-01".to_string(),
        },
        tags: vec!["rust".to_string(), "programming".to_string()],
        point: 10,
    };

    my_db.eingeben(&my_todo).await.unwrap();

    println!("************\nFound:\n************\n ");
    let found = my_db
        .finde()
        .wo("point")
        .zwischen([10, 400])
        .wo("status")
        .entspricht("Pending")
        .run()
        .await
        .unwrap();

    println!("{:#?}", found);

    println!("************\nDeleted:\n************\n");
    let deleted = my_db
        .entferne()
        .wo("status")
        .nicht_entspricht("Archived")
        .run()
        .await
        .unwrap();

    println!("{:#?}", deleted);

    println!("************\nAll items in db has been deleted! :)\n************\n");
    my_db.entferne_alle().await.unwrap();
}
