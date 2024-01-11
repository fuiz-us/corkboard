use std::{
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
};

use actix_web::web::Bytes;
use dashmap::DashMap;

pub trait Storage<T: Clone>: Default + Debug {
    fn store(&self, bytes: Bytes, thumbnail_bytes: Bytes) -> T;

    fn retrieve(&self, object_id: &T) -> Option<Bytes>;

    fn retrieve_thumbnail(&self, object_id: &T) -> Option<Bytes>;

    fn contains(&self, object_id: &T) -> bool;

    fn delete(&self, object_id: &T);
}

#[derive(Debug, Default)]
pub struct Memory(DashMap<u64, (Bytes, Bytes)>);

impl Storage<u64> for Memory {
    fn retrieve(&self, object_id: &u64) -> Option<Bytes> {
        self.0.get(object_id).map(|x| x.clone().0)
    }

    fn retrieve_thumbnail(&self, object_id: &u64) -> Option<Bytes> {
        self.0.get(object_id).map(|x| x.clone().1)
    }

    fn store(&self, bytes: Bytes, thumbnail_bytes: Bytes) -> u64 {
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        let hash = hasher.finish();
        self.0.insert(hash, (bytes, thumbnail_bytes));
        hash
    }

    fn contains(&self, object_id: &u64) -> bool {
        self.0.contains_key(object_id)
    }

    fn delete(&self, object_id: &u64) {
        self.0.remove(object_id);
    }
}
