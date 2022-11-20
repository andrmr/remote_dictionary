use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Get { key: String },
    Set { key: String, val: String },
    Stats,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub enum Response {
    Get { ok: bool, val: Option<String>, err: Option<String> },
    Set { ok: bool, err: Option<String> },
    Stats { ok: bool, total: Option<u64>, good: Option<u64>, bad: Option<u64> },

    #[default]
    Empty,
}
