use serde_json::{Result, Value}; use std::fs::{File};
use std::io::{Read, Write};

#[derive(PartialEq)]
pub struct TransportOptions { encrypt: bool }

pub struct Transport {}
impl Transport {
    pub fn get(&self, url: &str) -> Result<Value> {
        let mut file = File::open(url).expect("unable to open a file");
        let content: Value = serde_json::from_reader(&mut file).expect("unable to parse a file");

        Ok(content)
    }

    pub fn put<'a, 'b>(&'a self, url: &'b str, data: Value, opts: Option<TransportOptions>) -> Result<&'b str> {
        if Some(TransportOptions { encrypt: true }) == opts {
            println!("encrypt data is not yet supported");
        }
        let mut file = File::create(url).expect("unable to create a file");
        serde_json::to_writer(&mut file, &data).expect("unable to serialize a file");

        Ok(url)
    }

    pub fn del(&self, url: &str) -> Result<()> {
        File::open(url).expect("unable to open a file");
        std::fs::remove_file(url).expect("unable to remove a file");

        Ok(())
    }

    // XXX: seem to be quite useless
    pub fn update<'a, 'b>(&'a self, url: &'b str, data: Value, opts: Option<TransportOptions>) -> Result<&'b str> {
        if Some(TransportOptions { encrypt: true }) == opts {
            println!("encrypt data is not yet supported");
        }

        let content = self.get(url).expect("unable to get a file");
        let merge = Self::merge_json_objects(&content, &data);

        let res = self.put(url, merge, None).expect("unable to store file");

        Ok(res)
    }

    fn merge_json_objects(obj1: &Value, obj2: &Value) -> Value {
        if let (Value::Object(obj1), Value::Object(obj2)) = (obj1, obj2) {
            let mut merged = obj1.clone();
            for (key, value) in obj2 {
                merged.insert(key.clone(), value.clone());
            }
            Value::Object(merged)
        } else {
            panic!("Both inputs must be JSON objects");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport;

    static TRANSPORT: transport::Transport = Transport {};
    static URL: &str = "/home/rxitech/Projects/Synonym/pdk/fixtures/slashpay.test.json";


    #[test]
    #[should_panic]
    fn it_all_works() {
        let content: Value = serde_json::json!({"foo": "bar"});

        let result = TRANSPORT.put(URL, content, None) .unwrap();
        assert_eq!(result, URL);

        let result = TRANSPORT.get(URL).unwrap();
        assert_eq!(result, serde_json::json!({"foo": "bar"}));

        let result = TRANSPORT.update(URL, serde_json::json!({"zar":"gar"}), None) .unwrap();
        assert_eq!(result, URL);

        let result = TRANSPORT.get(URL).unwrap();
        assert_eq!(result, serde_json::json!({"foo": "bar", "zar": "gar"}));

        let result = TRANSPORT.del(URL).unwrap();
        assert_eq!(result, ());

        let _ = TRANSPORT.get(URL);
    }
}
