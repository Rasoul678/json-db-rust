use fake::{ Fake, Faker};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_value::Value;
use std::collections::VecDeque;
use std::io::{Error, ErrorKind, Result};

pub fn fake_it<T>(count: u32) -> Vec<T>
where
    T: fake::Dummy<fake::Faker>
{
    let mut list: Vec<T> = vec![];

    for _ in 0..count {
        let item = Faker.fake::<T>();

        list.push(item);
    }

    list
}

pub fn get_field_by_name<T, R>(data: T, field: &str) -> Result<R>
where
    T: Serialize,
    R: DeserializeOwned,
{
    let mut map = match serde_value::to_value(data) {
        Ok(Value::Map(map)) => map,
        _ => {
            return Err(Error::new(ErrorKind::InvalidInput, "expected a struct"));
        }
    };

    let key = Value::String(field.to_owned());
    let value = match map.remove(&key) {
        Some(value) => value,
        None => return Err(Error::new(ErrorKind::NotFound, "no such field")),
    };

    match R::deserialize(value) {
        Ok(r) => Ok(r),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e.to_string())),
    }
}

pub fn get_key_chain_value<T>(data: T, key_chain: &str) -> Option<Value>
where
    T: Serialize,
{
    let mut parts = key_chain.split('.').collect::<Vec<&str>>();
    let key = parts.remove(0);
    let value: Value = get_field_by_name(data, key).unwrap();

    if parts.len() > 0 {
        let new_key_chain = parts.join(".");
        return get_key_chain_value(value, &new_key_chain);
    }

    Some(value)
}

pub fn get_nested_value<T, R>(data: T, key_chain: &str) -> Result<R>
where
    T: Serialize,
    R: DeserializeOwned,
{
    let parts: VecDeque<&str> = key_chain.split('.').collect();
    let mut current_value = serde_value::to_value(data).unwrap();

    for key in parts {
        match current_value {
            Value::Map(mut map) => {
                let value_key = Value::String(key.to_owned());
                current_value = map.remove(&value_key).ok_or_else(|| {
                    Error::new(ErrorKind::NotFound, format!("Key '{}' not found", key))
                })?;
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Expected a nested structure",
                ))
            }
        }
    }

    match R::deserialize(current_value) {
        Ok(r) => Ok(r),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e.to_string())),
    }
}

pub fn value_map_to_struct<T>(value: Value) -> Result<T>
where
    T: DeserializeOwned,
{
    match value {
        Value::Map(_) => match T::deserialize(value) {
            Ok(result) => Ok(result),
            Err(e) => Err(Error::new(ErrorKind::InvalidData, e.to_string())),
        },
        _ => Err(Error::new(ErrorKind::InvalidInput, "Expected a Value::Map")),
    }
}
