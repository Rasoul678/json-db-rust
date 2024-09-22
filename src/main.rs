#![allow(unused_variables)]
use json_db::{fake_it, get_key_chain_value, JsonDB, ToDo};

#[tokio::main]
async fn main() {
    let my_db = JsonDB::new("todo").await.unwrap();

    let todos = fake_it::<ToDo>(10);

    for todo in &todos {
        let _ = my_db.insert(&todo).await;
    }

    let first_todo = &todos.into_iter().nth(0).unwrap();
    let value = get_key_chain_value(first_todo, "id").unwrap();
    // println!("{:#?}", value);

    let all_todos_before = my_db.get_all().await.unwrap();

    // println!("{:#?}", all_todos_before);
    let id = &all_todos_before.iter().nth(3).unwrap().id;

    // my_db.delete_all().await.unwrap();

    // my_db.delete_completed().await.unwrap();

    // let all_todos_after = my_db.get_all().await.unwrap();
    // println!("{:#?}", all_todos_after);

    let item_by_id = my_db.get_by_id(id).run().await;
    println!("{:#?}", item_by_id.first().unwrap());

    // let deleted = my_db.delete(id).await.unwrap();
    // println!("{:#?}", deleted);

    // let deleted_by_id = my_db.delete_by_id(id).await.unwrap();
    // println!("{:#?}", deleted_by_id);

    // my_db.delete_archived().run().await;

    // my_db.delete_not_completed().await.unwrap();
}
