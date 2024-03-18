use uuid::Uuid;
use serde_json::{Result, Value};
use std::fs::{File, self};
use std::path::Path;

const INDEX_URL: &str = "slashpay.json";

#[derive(PartialEq)]
pub struct TransportOptions { encrypt: bool }

pub struct Transport {}
impl Transport {
    pub fn get(&self, url: &str) -> Result<Value> {
        let mut file = File::open(url).expect("unable to open a file");
        let content: Value = serde_json::from_reader(&mut file).expect("unable to parse a file");

        Self::is_valid_json(&content);

        Ok(content)
    }

    pub fn put<'a, 'b>(&'a self, url: &'b str, data: Value, opts: Option<TransportOptions>) -> Result<&'b str> {
        Self::is_valid_json(&data);

        if Some(TransportOptions { encrypt: true }) == opts {
            println!("encrypted data is not yet supported");
        }
        let path = Path::new(url);
        let parent = path.parent().unwrap();
        fs::create_dir_all(parent).expect("unable to create a dir");
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
        Self::is_valid_json(&data);

        if Some(TransportOptions { encrypt: true }) == opts {
            println!("encrypted data is not yet supported");
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

    pub fn get_path(name: &str, index_url: Option<&str>, id: Option<&String>) -> String {
        let index_url = match index_url {
            Some(index_url) => index_url,
            None => INDEX_URL
        };

        let p = Path::new(index_url);
        // if !p.is_file() { panic!("malformed index"); }

        match id {
            Some(id) => Self::get_path_with_id(p, name, id),
            None => Self::get_path_without_id(p, name)
        }
    }

    fn get_path_with_id(p: &Path, name: &str, id: &str) -> String {
        if !Self::valid_uuid(id) { panic!("Invalid UUID: {id}"); }

        let dirs = p.parent().unwrap();
        let file_name: &str = p.file_name().unwrap().to_str().unwrap();
        dirs.join(id).join(name).join(file_name).to_str().unwrap().to_string()
    }

    fn get_path_without_id(p: &Path, name: &str) -> String {
        let dirs = p.parent().unwrap();
        let file_name: &str = p.file_name().unwrap().to_str().unwrap();
        dirs.join(name).join(file_name).to_str().unwrap().to_string()
    }

    fn valid_uuid(id: &str) -> bool {
        match Uuid::parse_str(id) {
            Ok(_) => true,
            Err(_) => false
        }
    }

    fn is_valid_json(content: &Value) -> () {
        if content.is_object() { () } else { panic!("invalid JSON") }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport;
    use std::env;

    #[test]
    fn walk_through() {
        let TRANSPORT: transport::Transport = Transport {};

        let path_to_file = Path::new(&env::temp_dir()).join("slashpay.test.json");
        let URL: &str = path_to_file.to_str().unwrap();
        let content: Value = serde_json::json!({"foo": "bar"});

        let result = TRANSPORT.put(URL, content, None).unwrap();
        assert_eq!(result, URL);

        let result = TRANSPORT.get(URL).unwrap();
        assert_eq!(result, serde_json::json!({"foo": "bar"}));

        let result = TRANSPORT.update(URL, serde_json::json!({"zar":"gar"}), None).unwrap();
        assert_eq!(result, URL);

        let result = TRANSPORT.get(URL).unwrap();
        assert_eq!(result, serde_json::json!({"foo": "bar", "zar": "gar"}));

        let result = TRANSPORT.del(URL).unwrap();
        assert_eq!(result, ());

        fs::metadata(URL).expect_err("file should not exist");
    }

    #[test]
    fn get_path_without_id() {
        let name = "test";
        let p = Path::new(&env::temp_dir()).join("slashpay.json");
        assert_eq!(
            Transport::get_path_without_id(&p, name),
            Path::new(&env::temp_dir()).join("test").join("slashpay.json").to_str().unwrap().to_string()
        );
    }

    #[test]
    fn get_path_with_id() {
        let name = "test";
        let id = "f47ac10b-58cc-4372-a567-0e02b2c3d479";
        let p = Path::new(&env::temp_dir()).join("slashpay.json");
        assert_eq!(
            Transport::get_path_with_id(&p, name, id),
            Path::new(&env::temp_dir()).join(id).join(name).join("slashpay.json").to_str().unwrap().to_string()
        );
    }

    #[test]
    #[should_panic]
    fn get_path_with_invalid_id() {
        let name = "test";
        let id = "invalid-uuid";
        let p = Path::new(&env::temp_dir()).join("slashpay.json");
        Transport::get_path_with_id(&p, name, id);
    }

    #[test]
    fn valid_uuid() {
        let id = "f47ac10b-58cc-4372-a567-0e02b2c3d479";
        assert_eq!(Transport::valid_uuid(id), true); 
    }

    #[test]
    fn invalid_uuid() {
        let id = "invalid-uuid";
        assert_eq!(Transport::valid_uuid(&id), false);
    }

    #[test]
    fn is_valid_json() {
        let content: Value = serde_json::json!({"foo": "bar"});
        assert_eq!(Transport::is_valid_json(&content), ());
    }

    #[test]
    #[should_panic]
    fn is_invalid_json() {
        let content: Value = serde_json::json!(["foo", "bar"]);
        Transport::is_valid_json(&content);
    }
}
