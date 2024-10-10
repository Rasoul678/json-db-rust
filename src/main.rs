#![allow(unused_variables)]

use ohmydb::{fake_it, JsonDB, ToDo};

#[tokio::main]
async fn main() {
    let mut db = JsonDB::new().await.unwrap();

    db.add_table("product").unwrap();

    let todos = fake_it::<ToDo>(1);

    for todo in &todos {
        db.insert_or("person", todo.clone()).run().await.unwrap();
    }

    let todo = ToDo::default();

    db.insert_or("todo", todo.clone())
        .run()
        .await
        .unwrap_or_else(|e| {
            println!("Error: {}", e);
            Vec::new()
        });

    db.insert("todo", todo.clone())
        .run()
        .await
        .unwrap_or(Vec::new());

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
    let deleted = db
        .delete("person")
        .where_("status")
        .not_equals("Archived")
        .where_("point")
        .less_than(500)
        .run()
        .await
        .unwrap();

    println!("{:#?}", deleted);

    println!("************\nUpdate:\n************\n");
    let td = ToDo::default();

    let updated = db.update("todo", td.clone()).run().await.unwrap_or(vec![]);
    let updated = db.update("person", td).run().await.unwrap_or(vec![]);
    println!("{:#?}", updated);

    println!("************\nAll items in db has been deleted! :)\n************\n");
    db.delete("person").run().await.unwrap();
    db.delete("todo").run().await.unwrap();
}
