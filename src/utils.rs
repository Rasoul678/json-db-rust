use crate::types::{Status, ToDo};
use fake::faker::lorem::en::*;
use fake::locales::*;
use fake::{faker::name::raw::*, Fake, Faker};

pub fn fake_todo(count: u32) -> Vec<ToDo> {
    let mut todos: Vec<ToDo> = vec![];

    for _ in 0..count {
        let todo = ToDo {
            id: (Faker.fake::<ToDo>()).id,
            name: Sentence(5..10).fake(),
            status: Faker.fake::<Status>(),
            user: Name(EN).fake(),
        };

        todos.push(todo);
    }

    todos
}
