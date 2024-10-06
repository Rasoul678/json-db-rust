#![allow(unused_variables)]

use json_db::{fake_it, JsonDB, ToDo};

#[tokio::main]
async fn main() {
    let mut db = JsonDB::new().await.unwrap();

    let todos = fake_it::<ToDo>(1);

    for todo in &todos {
        db.insert_or("people", todo.clone()).run().await.unwrap();
    }

    let todo = ToDo::default();

    db.insert_or("todo", todo.clone())
        .run()
        .await
        .unwrap_or_else(|e| {
            println!("Error: {}", e);
            Vec::new()
        });

    println!("************\nFound:\n************\n ");
    let found = db
        .find("todo")
        .where_("status")
        .equals("Pending")
        .where_("point")
        .less_than(500)
        .run()
        .await
        .unwrap();

    println!("{:#?}", found);

    println!("************\nDeleted:\n************\n");
    // let deleted = my_db
    //     .delete()
    //     .where_("status")
    //     .not_equals("Archived")
    //     .where_("point")
    //     .less_than(500)
    //     .run()
    //     .await
    //     .unwrap();

    // println!("{:#?}", deleted);

    println!("************\nUpdate:\n************\n");
    // let td = ToDo {
    //     id: "100".to_string(),
    //     text: "Learn Rust".to_string(),
    //     status: Status::Pending,
    //     user: User {
    //         name: Name {
    //             first: "Rasoul".to_string(),
    //             last: "Hesami Rostami".to_string(),
    //         },
    //         email: "rasoul.hesami@gmail.com".to_string(),
    //     },
    //     date: Date {
    //         start: "2023-01-01".to_string(),
    //         end: "2025-01-01".to_string(),
    //     },
    //     tags: vec!["rust".to_string(), "programming".to_string()],
    //     point: 10,
    // };
    // let updated = my_db.update("todo", td).run().await.unwrap();
    // println!("{:#?}", updated);

    println!("************\nAll items in db has been deleted! :)\n************\n");
    db.delete("todo").run().await.unwrap();
}
