use actix_web::web::Bytes;
use dashmap::{mapref::entry::Entry, DashMap};
use mime::Mime;
use serde::{Deserialize, Serialize};
use serde_hex::{SerHex, Strict};

use crate::storage::Storage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MediaId(#[serde(with = "SerHex::<Strict>")] u64);

impl MediaId {
    fn new() -> Self {
        Self(rand::random())
    }
}

#[derive(Debug, Default)]
pub struct MediaManager<U: Clone, T: Storage<U>> {
    mapping: DashMap<MediaId, U>,
    storage: T,
}

impl<U: Clone, T: Storage<U>> MediaManager<U, T> {
    pub fn store(&self, bytes: Bytes, content_type: Mime) -> MediaId {
        let object_id = self.storage.store(bytes, content_type);
        loop {
            let media_id = MediaId::new();
            match self.mapping.entry(media_id) {
                Entry::Occupied(_) => continue,
                Entry::Vacant(v) => {
                    v.insert(object_id);
                    return media_id;
                }
            };
        }
    }

    pub fn retrieve(&self, media_id: MediaId) -> Option<(Bytes, Mime)> {
        self.mapping
            .get(&media_id)
            .map(|x| x.to_owned())
            .and_then(|x| self.storage.retrieve(&x))
    }

    pub fn contains(&self, media_id: MediaId) -> bool {
        match self.mapping.get(&media_id) {
            Some(x) => self.storage.contains(&x),
            None => false,
        }
    }

    pub fn delete(&self, media_id: MediaId) {
        self.mapping.remove(&media_id);
    }
}
