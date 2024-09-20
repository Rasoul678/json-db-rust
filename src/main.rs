use json_db::{fake_todo, JsonDB};

#[tokio::main]
async fn main() {
    let my_db = JsonDB::new("todo").await.unwrap();

    let todos = fake_todo(10);

    for todo in todos {
        let _ = my_db.insert(&todo).await;
    }

    let all_todos_before = my_db.get_all().await.unwrap();

    println!("{:#?}", all_todos_before);
    let id = &all_todos_before.first().unwrap().id;

    // my_db.delete_all().await.unwrap();

    // my_db.delete_completed().await.unwrap();

    // let all_todos_after = my_db.get_all().await.unwrap();
    // println!("{:#?}", all_todos_after);

    // let item_by_id = my_db.get_by_id(id).await.unwrap();
    // println!("{:#?}", item_by_id);

    // let deleted = my_db.delete(id).await.unwrap();
    // println!("{:#?}", deleted);

    // let deleted_by_id = my_db.delete_by_id(id).await.unwrap();
    // println!("{:#?}", deleted_by_id);

    // my_db.delete_archived().await.unwrap();

    // my_db.delete_not_completed().await.unwrap();
}
