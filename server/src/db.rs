use std::collections::HashMap;
use std::hash::Hash;

use anyhow::Ok;
use persy::{Persy, Config};
use tokio::sync::RwLock;

// Wrapper over Persy, an in-process database with persistent disk storage
// TODO: Sled as alternative
pub struct Db<K, V> {
    name: String,
    db: Persy,
    cache: AsyncCache<K, Option<V>>,
}

pub type DbResult<T> = anyhow::Result<T>;

impl<K, V> Db<K, V>
where K: persy::IndexType, V: persy::IndexType
{
    pub fn open(path: &str, name: String) -> DbResult<Self> {
        let db = Persy::open(path, Config::new())?;

        Ok(Self {
            name,
            db,
            cache: AsyncCache::new(),
        })
    }

    pub fn create(path: &str, name: String) -> DbResult<Self> {
        Persy::create(path)?;

        let db = Self::open(path, name)?;
        let mut tx = db.db.begin()?;
        tx.create_index::<K, V>(&db.name, persy::ValueMode::Replace)?;
        let prepared = tx.prepare()?;
        prepared.commit()?;

        Ok(db)
    }

    pub fn open_or_create(name: String) -> DbResult<Self>
    {
        let path = format!("./{}.db", name);
        let path = std::path::Path::new(&path);

        let db: Db<K, V> = if path.exists() {
            println!("Opening storage");
            Db::open(path.to_str().unwrap(), name)?
        } else {
            println!("Creating storage");
            Db::create(path.to_str().unwrap(), name)?
        };

        Ok(db)
    }

    pub async fn get(&self, key: &K) -> DbResult<Option<V>>
    where K: Eq + Hash + persy::IndexType, V: Clone + persy::IndexType
    {
        // cached values are Options:
        // None         => the key is not cached
        // Some(None)   => the key is cached, without value
        // Some(val)    => the value is cached
        if let Some(cached) = self.cache.get(&key).await {
            println!("Cache hit");
            Ok(cached)
        } else {
            println!("Cache miss");
            let mut tx = self.db.begin()?;
            let val = tx.one::<K, V>(&self.name, &key)?;

            self.cache.set(key.clone(), val.clone()).await;

            Ok(val)
        }
    }

    pub async fn set(&self, key: &K, val: &V) -> DbResult<()>
    where K: Eq + Hash + persy::IndexType, V: Clone + persy::IndexType
    {
        let mut tx = self.db.begin()?;
        tx.put::<K, V>(&self.name, key.to_owned(), val.to_owned())?;
        tx.prepare()?.commit()?;

        self.cache.set(key.clone(), Some(val.clone())).await;

        Ok(())
    }
}


// Multi-reader, single-writer cache
struct AsyncCache<K, V> {
    cache: RwLock<HashMap<K, V>>
}

impl<K, V> AsyncCache<K, V> {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::<K,V>::new())
        }
    }

    pub async fn get(&self, key: &K) -> Option<V>
    where K: Eq + Hash, V: Clone
    {
        self.cache
            .read()
            .await
            .get(&key)
            .cloned()
    }

    pub async fn set(&self, key: K, val: V)
    where K: Eq + Hash
    {
        self.cache
            .write()
            .await
            .insert(key, val);
    }
}
