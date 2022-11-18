use anyhow::Ok;
use persy::{Persy, Config};

pub struct Db {
    handle: Persy,
}

pub type DbResult<T> = anyhow::Result<T>;

impl Db {
    pub fn open(path: &str) -> DbResult<Self> {
        let handle = Persy::open(path, Config::new())?;

        Ok(Self {
            handle
        })
    }

    pub fn create(path: &str) -> DbResult<Self> {
        Persy::create(path)?;

        let db = Self::open(path)?;
        let mut tx = db.handle.begin()?;
        tx.create_index::<String, String>("dict", persy::ValueMode::Replace)?;
        tx.create_index::<String, u32>("stats", persy::ValueMode::Replace)?;
        let prepared = tx.prepare()?;
        prepared.commit()?;

        Ok(db)
    }

    pub fn get(&self, key: &str) -> DbResult<Option<String>> {
        let mut tx = self.handle.begin()?;
        let val = tx.one::<String, String>("dict", &key.to_owned())?;

        Ok(val)
    }

    pub fn set(&self, key: &str, val: &str) -> DbResult<()> {
        let mut tx = self.handle.begin()?;
        tx.put::<String, String>("dict", key.to_owned(), val.to_owned())?;
        tx.prepare()?.commit()?;

        Ok(())
    }
}
