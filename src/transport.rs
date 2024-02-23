use serde_json::{Result, Value};
use std::fs::{File};
use std::io::{Read, Write};

#[derive(PartialEq)]
pub struct TransportOptions { encrypt: bool }

pub struct Transport {}
impl Transport {
    pub fn get(&self, url: &str) -> Result<Value> {
        let mut file = File::open(url).expect("unable to open a file");
        let mut content = String::new();
        file.read_to_string(&mut content).expect("unable to read a file");

        Ok(serde_json::from_str(&content).expect("unable to parse a file"))
    }

    pub fn put(&self, url: &str, data: &str, opts: Option<TransportOptions>) -> Result<String> {
        if Some(TransportOptions { encrypt: true }) == opts {
            println!("encrypt data is not yet supported");
        }
        let mut file = File::create(url).expect("unable to create a file");
        file.write_all(data.as_bytes()).expect("unable to write a file");

        Ok(url.to_string())
    }

    pub fn del(&self, url: &str) -> Result<()> {
        File::open(url).expect("unable to open a file");
        std::fs::remove_file(url).expect("unable to remove a file");

        Ok(())
    }

    // XXX: seem to be quite useless
    pub fn update(&self, url: &str, data: &str, opts: Option<TransportOptions>) -> Result<String> {
        if Some(TransportOptions { encrypt: true }) == opts {
            println!("encrypt data is not yet supported");
        }

        let content = self.get(url).expect("unable to get a file");
        let update = serde_json::from_str(data).expect("unable to parse a file");
        let merge = Self::merge_json_objects(&content, &update);

        let res = self.put(url, &merge.to_string(), None).expect("unable to store file");

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
    static CONTENT: &str = "{\"foo\":\"bar\"}";


    #[test]
    #[should_panic]
    fn it_all_works() {
        let result = TRANSPORT.put(URL, CONTENT, None) .unwrap();
        assert_eq!(result, URL);

        let result = TRANSPORT.get(URL).unwrap();
        assert_eq!(result, serde_json::from_str::<Value>(&CONTENT).unwrap());

        let result = TRANSPORT.update(URL, "{\"zar\":\"gar\"}", None) .unwrap();
        assert_eq!(result, URL);

        let result = TRANSPORT.get(URL).unwrap();
        assert_eq!(result, serde_json::from_str::<Value>("{\"foo\": \"bar\", \"zar\": \"gar\"}").unwrap());

        let result = TRANSPORT.del(URL).unwrap();
        assert_eq!(result, ());

        let _ = TRANSPORT.get(URL);
    }
}
