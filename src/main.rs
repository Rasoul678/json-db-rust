#![allow(unused_variables)]

use std::collections::HashSet;
use json_db::{fake_it, Date, JsonDB, Name, Status, ToDo, User};

#[tokio::main]
async fn main() {
    let mut my_db = JsonDB::new("todo").await.unwrap();

    let todos = fake_it::<ToDo>(10);

    for todo in todos {
        my_db.insert(todo).run().await.unwrap();
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

    my_db.insert(my_todo).run().await.unwrap_or_else(|e| {
        println!("Error: {}", e);
        HashSet::new()
    });

    println!("************\nFound:\n************\n ");
    let found = my_db
        .find()
        .where_("status")
        .equals("Pending")
        .where_("point")
        .less_than(300)
        .run()
        .await
        .unwrap();

    println!("{:#?}", found);

    println!("************\nDeleted:\n************\n");
    // let deleted = my_db
    //     .delete()
    //     .where_("status")
    //     .not_equals("Archived")
    //     .run()
    //     .await
    //     .unwrap();

    // println!("{:#?}", deleted);

    println!("************\nAll items in db has been deleted! :)\n************\n");
    my_db.delete_all().await.unwrap();
}
