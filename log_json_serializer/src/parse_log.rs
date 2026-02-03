use chrono::Utc;
use log::{
    Record,
    kv::{Key, Value, VisitSource},
};
use std::{collections::BTreeMap, fmt::Arguments};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to parse log to json: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn parse_log(message: &Arguments, record: &Record) -> Result<String, Error> {
    let mut visitor = Collect(BTreeMap::new());

    // this funtion is infallable
    record.key_values().visit(&mut visitor).unwrap();

    let kv = {
        let mut kv = visitor.0;
        kv.insert(
            Key::from("msg"),
            if let Some(msg) = message.as_str() {
                Value::from(msg)
            } else {
                Value::from_display(message)
            },
        );

        kv.insert(Key::from_str("target"), Value::from(record.target()));

        if let Some(val) = record.module_path() {
            kv.insert(Key::from("module"), Value::from(val));
        }
        if let Some(val) = record.file() {
            kv.insert(Key::from("file"), Value::from(val));
        }
        if let Some(val) = record.line() {
            kv.insert(Key::from("line"), Value::from(val));
        }

        let now: i64 = Utc::now().timestamp();
        kv.insert(Key::from("timestamp"), Value::from(now));

        kv
    };
    let json = serde_json::to_string(&kv).map_err(Error::Json)?;

    Ok(json)
}

struct Collect<'kvs>(BTreeMap<Key<'kvs>, Value<'kvs>>);
impl<'kvs> VisitSource<'kvs> for Collect<'kvs> {
    fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), log::kv::Error> {
        self.0.insert(key, value);
        Ok(())
    }
}
