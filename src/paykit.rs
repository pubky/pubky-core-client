use crate::transport::Transport;
use std::collections::HashMap;
use serde_json::Value;
use serde::{Serialize, Deserialize};
use std::fmt;
use uuid::Uuid;

const INDEX_URL: &str = "slashpay.json";

struct Paykit {
    transport: Transport
    // indexUrl: String
}

// struct PluginData {}

// struct PaykitEndpoint { HashMap<String, PluginData> }

impl Paykit {
    pub fn new() -> Paykit {
        Paykit { transport: Transport {} }
    }
    /* RECEIVER PERSPECTIVE: */
    // NOTE: index file is always auto updated

    /* PUBLIC PAYMENT ENDPOINT */
    // For each name as a key in first hashmap argument, create a new file with location derived
    // based on the name and content as json object of value. Store links to these files in index_url

    /// Creates a new public payment endpoint for each plugin in the plugin_map, filling the
    /// content with the plugin data. It stores links to each plugin related file in index file
    /// accessible via `index_url` and returns index url as a result.
    fn create_all<'a> (&'a self, plugin_map: HashMap <String, Value>, index_url: Option<&'a str>) -> &str {
        let index_url = Self::get_url(index_url);

        let mut index = HashMap::new();
        for (name, data) in plugin_map {
            let path = Self::get_path(&name, None);
            self.transport.put(&path, data, None).expect("Failed to write plugin data");
            index.insert(name, path);
        }

        self.transport.put(&index_url, serde_json::json!(&index), None).expect("Failed to write index")
    }

    // createPublicPaymentEndpoint(pluginName: String, pluginData: Value, indexUrl: Option(String)) - return public index url
    //

    // updatePulicPaymentEndpoint(pluginName: String, pluginData: Value, indexUrl: Option(Stirng)) - return public index url
    //

    // deletePublicPaymentEndpoint(pluginName: String, indexUrl: Option(String)) - return private index url
    //
  
    /* PRIVATE PAYMENT ENDPOINT */
    // NOTE: url for index file is always autoderived based on id

    // createAllPrivate (PluginMap: HashMap<String, PluginData>, amount: u8) - return public index url
    //

    // createPrivatePaymentEndpoint(id: String, pluginName: String, pluginData: Value, amount: u8) - return private index url
    //

    // updatePrivatePaymentEndpoint(id: String, pluginName: String, pluginData: Value, amount: u8) - return private index url
    //

    // deletePrivatePaymentEndpoint(id: String, pluginName: String) - return private index url
    //
  
    /* SENDER PERSPECTIVE: */
    /* PUBLIC AND PRIVATE PAYMENT ENDPOINT */
    // readAll(indexUrl: Option(String)) - return {plugin name, plugin data}
    //

    fn get_url(url: Option<&str>) -> &str {
        match url {
            Some(url) => url,
            None => INDEX_URL
        }
    }

    fn get_path(name: &str, id: Option<&String>) -> String {
        match id {
            Some(id) => Self::get_path_with_id(name, id),
            None => Self::get_path_without_id(name)
        }
    }

    fn get_path_without_id(name: &str) -> String {
        format!("/slashpay/{name}/slashpay.json")
    }

    fn get_path_with_id(name: &str, id: &str) -> String {
        if !Self::valid_uuid(id) { panic!("Invalid UUID: {id}"); }

        format!("/slashpay/{id}/{name}/slashpay.json")
    }

    fn valid_uuid(id: &str) -> bool {
        match Uuid::parse_str(id) {
            Ok(_) => true,
            Err(_) => false
        }
    }


}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_url() {
        let paykit = Paykit::new();
        let url = Some("slashpay.json");
        assert_eq!(Paykit::get_url(url), String::from("slashpay.json"));

        let url = None;
        assert_eq!(Paykit::get_url(url), String::from("slashpay.json"));
    }

    #[test]
    fn get_path_without_id() {
        let name = "test";
        assert_eq!(Paykit::get_path_without_id(name), "/slashpay/test/slashpay.json");
    }

    #[test]
    fn valid_uuid() {
        let id = "f47ac10b-58cc-4372-a567-0e02b2c3d479";
        assert_eq!(Paykit::valid_uuid(id), true); 
    }

    #[test]
    fn invalid_uuid() {
        let id = "invalid-uuid";
        assert_eq!(Paykit::valid_uuid(&id), false);
    }

    #[test]
    fn get_paht_with_id() {
        let name = "test";
        let id = "f47ac10b-58cc-4372-a567-0e02b2c3d479";
        assert_eq!(Paykit::get_path_with_id(name, id), "/slashpay/f47ac10b-58cc-4372-a567-0e02b2c3d479/test/slashpay.json");
    }

    #[test]
    #[should_panic]
    fn get_path_with_invalid_id() {
        let name = "test";
        let id = "invalid-uuid";
        Paykit::get_path_with_id(name, id);
    }

    // #[test]
    // fn create_all() {
    //     let paykit = Paykit::new();
    //     let mut plugin_map = HashMap::new();
    //     plugin_map.insert(String::from("test"), serde_json::json!({"test": "test"}));
    //     let index_url = "/home/rxitech/Projects/Synonym/pdk/fixtures/slashpay.test.json";
    //     assert_eq!(paykit.create_all(plugin_map, Some(index_url)), index_url);
    // }
}
