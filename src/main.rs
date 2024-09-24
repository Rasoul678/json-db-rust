#![allow(unused_variables)]

use json_db::{fake_it, Date, JsonDB, Name, Status, ToDo, User};

#[tokio::main]
async fn main() {
    let my_db = JsonDB::new("todo").await.unwrap();

    let todos = fake_it::<ToDo>(10);

    for todo in &todos {
        let _ = my_db.insert(&todo).await;
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

    my_db.insert(&my_todo).await.unwrap();

    let all_todos_before = my_db.find_all().await.unwrap();
    // println!("{:#?}", all_todos_before);

    let id = &all_todos_before.iter().nth(3).unwrap().id;

    let found = my_db
        .find()
        ._where("point")
        .less_than(500)
        ._where("status")
        .equals("Completed")
        .run()
        .await
        .unwrap();

    println!("{:#?}", found);

    // my_db.delete_all().await.unwrap();

    // my_db.delete_completed().await.unwrap();

    // let all_todos_after = my_db.get_all().await.unwrap();
    // println!("{:#?}", all_todos_after);

    // let item_by_id = my_db.get_by_id(id).run().await.unwrap();
    // println!("{:#?}", item_by_id.first().unwrap());

    // let deleted = my_db.delete(id).await.unwrap();
    // println!("{:#?}", deleted);

    // let deleted_by_id = my_db.delete_by_id(id).await.unwrap();
    // println!("{:#?}", deleted_by_id);

    // let deleted_arch = my_db.delete_archived().run().await;
    // println!("{:#?}", deleted_arch);

    // let deleted_com = my_db.delete_completed().run().await;
    // println!("{:#?}", deleted_com);

    // my_db.delete_not_completed().await.unwrap();
}
