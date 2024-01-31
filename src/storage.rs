use std::{
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
};

use actix_web::web::Bytes;
use dashmap::DashMap;
use mime::Mime;

pub trait Storage<T: Clone>: Default + Debug {
    fn store(&self, bytes: Bytes, content_type: Mime) -> T;

    fn retrieve(&self, object_id: &T) -> Option<(Bytes, Mime)>;

    fn contains(&self, object_id: &T) -> bool;

    fn delete(&self, object_id: &T);
}

#[derive(Debug, Default)]
pub struct Memory(DashMap<u64, (Bytes, Mime)>);

impl Storage<u64> for Memory {
    fn retrieve(&self, object_id: &u64) -> Option<(Bytes, Mime)> {
        self.0.get(object_id).map(|x| x.clone())
    }

    fn store(&self, bytes: Bytes, content_type: Mime) -> u64 {
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        let hash = hasher.finish();
        self.0.insert(hash, (bytes, content_type));
        hash
    }

    fn contains(&self, object_id: &u64) -> bool {
        self.0.contains_key(object_id)
    }

    fn delete(&self, object_id: &u64) {
        self.0.remove(object_id);
    }
}
