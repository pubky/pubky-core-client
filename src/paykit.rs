use crate::transport_fs::TransportFs;
use std::collections::HashMap;
use serde_json::Value;

const INDEX_URL: &str = "slashpay.json";

struct Paykit {
    transport: TransportFs
    // index_url: String
}

// struct PluginData {}

// struct PaykitEndpoint { HashMap<String, PluginData> }

impl Paykit {
    pub fn new() -> Paykit {
        Paykit { transport: TransportFs {} }
    }
    /* ------ RECEIVER PERSPECTIVE: ------ */
    // NOTE: index file is always auto updated

    /* PUBLIC PAYMENT ENDPOINT */

    /// Creates a new public payment endpoint for each plugin in the plugin_map, filling the
    /// content with the plugin data. It stores links to each plugin related file in index file
    /// accessible via `index_url` and returns index url as a result.
    pub fn create_all(&self, plugin_map: &Value, index_url: Option<&str>) -> Result<String, String> {
        let index_url = Self::get_url(index_url);

        for (name, data) in plugin_map.as_object().unwrap() {
            match Self::create_public_payment_endpoint(&self, name, data, Some(index_url)) {
                Ok(_) => (),
                Err(e) => return Err(e)
            };
        };

        Ok(index_url.to_string())
    }

    /// Creates a new public payment endpoint for a plugin with the given `plugin_name` and fills
    /// the content with the `plugin_data`. It stores the link to the plugin related file in index
    /// file accessible via `index_url` and returns the path to the plugin related file as a result.
    pub fn create_public_payment_endpoint(&self, plugin_name: &str, plugin_data: &Value, index_url: Option<&str>) -> Result<String, String> {
        let index_url = Self::get_url(index_url);
        let path = TransportFs::get_path(&plugin_name, Some(index_url), None);
        self.transport.put(&path, plugin_data, None).expect("Failed to write plugin data");

        let mut index = HashMap::new();
        // TODO: insert top level key for extensibility
        index.insert(plugin_name, &path);

        match self.transport.update(&index_url, &serde_json::json!(&index), None) {
            Ok(_) => Ok(path),
            Err(_) => {
                match self.transport.put(&index_url, &serde_json::json!(&index), None) {
                    Ok(_) => Ok(path),
                    Err(e) => Err(format!("Failed to write index: {e}"))
                }
            }
        }
    }

    /// Updates a public payment endpoint for a plugin with the given `plugin_name` and fills the content with the `plugin_data`.
    /// It stores the link to the plugin related file in index file accessible via `index_url` and returns the path to the plugin related file as a result.
    pub fn update_pulic_payment_endpoint(&self, plugin_name: &str, plugin_data: &Value, index_url: Option<&str>) -> Result<String, String> {
        let index_url = Self::get_url(index_url);
        let path = TransportFs::get_path(&plugin_name, Some(index_url), None);

        match self.transport.put(&path, plugin_data, None) {
            Ok(_) => Ok(index_url.to_string()),
            Err(e) => return Err(format!("Failed to write plugin data: {e}"))
        }
    }


    /// Deletes a public payment endpoint for a plugin with the given `plugin_name`. It removes the link to the plugin related file from index file.
    /// Returns the path to the plugin related file as a result.
    pub fn delete_public_payment_endpoint(&self, plugin_name: &str, index_url: Option<&str>) -> Result<String, String> {
        let index_url = Self::get_url(index_url);
        let path = TransportFs::get_path(&plugin_name, Some(index_url), None);

        match self.transport.del(&path) {
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to delete plugin data: {e}"))
        }

        let mut index = self.transport.get(&index_url).unwrap();
        index.as_object_mut().unwrap().remove(plugin_name);

        match self.transport.put(&index_url, &index, None) {
            Ok(_) => Ok(index_url.to_string()),
            Err(e) => Err(format!("Failed to write index: {e}"))
        }
    }

    /* PRIVATE PAYMENT ENDPOINT */
    // NOTE: url for index file is always autoderived based on id

    // pub create_all_private (PluginMap: HashMap<String, PluginData>, amount: u8) - return public index url
    //

    // pub create_private_payment_endpoint(id: String, plugin_name: String, plugin_data: Value, amount: u8) - return private index url
    //

    // pub update_private_payment_endpoint(id: String, plugin_name: String, plugin_data: Value, amount: u8) - return private index url
    //

    // pub delete_private_payment_endpoint(id: String, plugin_name: String) - return private index url
    //

    /* SENDER PERSPECTIVE: */
    /* PUBLIC AND PRIVATE PAYMENT ENDPOINT */
    /// Read index file by url and return its content as a result.
    pub fn read_index(&self, index_url: Option<&str>) -> Result<Value, String> {
        let index_url = Self::get_url(index_url);

        self.transport.get(index_url)
    }

    /// Read payment endpoint file by url and return its content as a result.
    pub fn read_payment_endpoint(&self, path: &str) -> Result<Value, String> {
        self.transport.get(path)
    }

    /// Read payment endpoint by name and return its content as a result.
    pub fn read_payment_endpoint_by_name(&self, plugin_name: &str, index_url: Option<&str>) -> Result<Value, String> {
        let index_url = Self::get_url(index_url);
        let path = TransportFs::get_path(&plugin_name, Some(index_url), None);

        self.transport.get(&path)
    }

    /// Read all payment endpoints by index url and return their content as a result.
    pub fn read_all(&self, index_url: Option<&str>) -> Result<Value, String> {
        let index_url = Self::get_url(index_url);
        let index = match self.transport.get(&index_url) {
            Ok(index) => index,
            Err(e) => return Err(format!("Failed to read index: {e}"))
        };

        let mut result: HashMap<String, Value> = HashMap::new();
        for (name, path) in index.as_object().unwrap() {
            match self.transport.get(path.as_str().unwrap()) {
                Ok(data) => result.insert(name.to_string(), data),
                Err(e) => return Err(format!("Failed to read plugin data: {e}"))
            };
        };

        Ok(serde_json::json!(result))
    }
    

    fn get_url(url: Option<&str>) -> &str {
        match url {
            Some(url) => url,
            None => INDEX_URL
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::Path;

    use super::*;

    #[test]
    fn get_url() {
        let _paykit = Paykit::new();
        let url = Some("slashpay.json");
        assert_eq!(Paykit::get_url(url), String::from("slashpay.json"));

        let url = None;
        assert_eq!(Paykit::get_url(url), String::from("slashpay.json"));
    }

    #[test]
    fn create_all() {
        let paykit = Paykit::new();
        let plugin_a_name = "pluginA";
        let plugin_b_name = "pluginB";
        // TODO: add some top level key for extensibility
        let value = serde_json::json!({
            plugin_a_name: { "bolt11": "lnbcrt..."},
            plugin_b_name: { "onchain": "bc1q..."}
        });
        let test_folder = Path::new(&env::temp_dir()).join("pdk_test").join("paykit").join("create_all");
        let index_url = test_folder.join("slashpay.json");
        let index_url: &str = index_url.to_str().unwrap();
        assert_eq!(paykit.create_all(&value, Some(index_url)), Ok(index_url.to_string()));

        let file_a_path = test_folder.join(plugin_a_name).join("slashpay.json");
        let file_a_path = file_a_path.to_str().unwrap();
        let file_b_path = test_folder.join(plugin_b_name).join("slashpay.json");
        let file_b_path: &str = file_b_path.to_str().unwrap();

        assert_eq!(paykit.transport.get(index_url), Ok(serde_json::json!({ plugin_a_name: file_a_path, plugin_b_name: file_b_path})));
        assert_eq!(paykit.transport.get(file_a_path), Ok(serde_json::json!({ "bolt11": "lnbcrt..."})));
        assert_eq!(paykit.transport.get(file_b_path), Ok(serde_json::json!({ "onchain": "bc1q..."})));

        std::fs::remove_file(index_url).unwrap();
        std::fs::remove_file(file_a_path).unwrap();
        std::fs::remove_file(file_b_path).unwrap();
    }

    #[test]
    fn create_public_payment_endpoint() {
        let paykit = Paykit::new();
        let test_folder = Path::new(&env::temp_dir()).join("pdk_test").join("paykit").join("create_public_payment_endpoint");
        let index_url = test_folder.join("slashpay.json");
        let index_url: &str = index_url.to_str().unwrap();

        let plugin1_name: &str = "test1";
        let plugin1_data = serde_json::json!({ "data": "lnbcrt..." });

        assert_eq!(
            paykit.create_public_payment_endpoint(plugin1_name, &plugin1_data, Some(index_url)),
            Ok(test_folder.join(plugin1_name).join("slashpay.json").to_str().unwrap().to_string())
        );

        let read_index = paykit.transport.get(index_url).unwrap();
        let file1_path = test_folder.join("test1").join("slashpay.json");
        let file1_path: &str = file1_path.to_str().unwrap();
        let read_value = paykit.transport.get(file1_path).unwrap();

        assert_eq!(read_index, serde_json::json!({"test1": file1_path}));
        assert_eq!(read_value, plugin1_data);

        let plugin2_name: &str = "test2";
        let plugin2_data = serde_json::json!({ "data": "lnbcrt..." });
        assert_eq!(
            paykit.create_public_payment_endpoint(plugin2_name, &plugin2_data, Some(index_url)),
            Ok(test_folder.join(plugin2_name).join("slashpay.json").to_str().unwrap().to_string())
        );

        let file2_path = test_folder.join("test2").join("slashpay.json");
        let file2_path: &str = file2_path.to_str().unwrap();
        let read_index = paykit.transport.get(index_url).unwrap();
        let read_value = paykit.transport.get(file2_path).unwrap();

        assert_eq!(read_index, serde_json::json!({"test1": file1_path, "test2": file2_path}));
        assert_eq!(read_value, plugin2_data);

        std::fs::remove_file(index_url).unwrap();
        std::fs::remove_file(file1_path).unwrap();
        std::fs::remove_file(file2_path).unwrap();
    }

    #[test]
    fn update_pulic_payment_endpoint() {
        let paykit = Paykit::new();
        let test_folder = Path::new(&env::temp_dir()).join("pdk_test").join("paykit").join("update_pulic_payment_endpoint");
        let index_url = test_folder.join("slashpay.json");
        let index_url: &str = index_url.to_str().unwrap();

        let plugin1_name: &str = "test1";
        let plugin1_data = serde_json::json!({ "data": "lnbcrt..." });

        assert_eq!(
            paykit.create_public_payment_endpoint(plugin1_name, &plugin1_data, Some(index_url)),
            Ok(test_folder.join(plugin1_name).join("slashpay.json").to_str().unwrap().to_string())
        );

        let read_index = paykit.transport.get(index_url).unwrap();
        let file1_path = test_folder.join(plugin1_name).join("slashpay.json");
        let file1_path: &str = file1_path.to_str().unwrap();
        let read_value = paykit.transport.get(file1_path).unwrap();

        assert_eq!(read_index, serde_json::json!({plugin1_name: file1_path}));
        assert_eq!(read_value, plugin1_data);

        let plugin1_data = serde_json::json!({ "data": "lnbcrt...updated" });
        assert_eq!(
            paykit.update_pulic_payment_endpoint(plugin1_name, &plugin1_data, Some(index_url)),
            Ok(test_folder.join("slashpay.json").to_str().unwrap().to_string())
        );

        let read_index = paykit.transport.get(index_url).unwrap();
        let read_value = paykit.transport.get(file1_path).unwrap();

        assert_eq!(read_index, serde_json::json!({plugin1_name: file1_path}));
        assert_eq!(read_value, plugin1_data);

        std::fs::remove_file(index_url).unwrap();
        std::fs::remove_file(file1_path).unwrap();
    }

    #[test]
    fn delete_public_payment_endpoint() {
        let paykit = Paykit::new();
        let test_folder = Path::new(&env::temp_dir()).join("pdk_test").join("paykit").join("delete_public_payment_endpoint");
        let index_url = test_folder.join("slashpay.json");
        let index_url: &str = index_url.to_str().unwrap();
        
        let plugin1_name: &str = "test1";
        let plugin1_data = serde_json::json!({ "data": "lnbcrt...1" });
        let plugin2_name: &str = "test2";
        let plugin2_data = serde_json::json!({ "data": "lnbcrt...2" });
        let data = serde_json::json!({plugin1_name: plugin1_data, plugin2_name: plugin2_data});

        assert_eq!(
            paykit.create_all(&data, Some(index_url)),
            Ok(test_folder.join("slashpay.json").to_str().unwrap().to_string())
        );

        let read_index = paykit.transport.get(index_url).unwrap();
        let file1_path = test_folder.join("test1").join("slashpay.json");
        let file1_path: &str = file1_path.to_str().unwrap();
        let file2_path = test_folder.join("test2").join("slashpay.json");
        let file2_path: &str = file2_path.to_str().unwrap();
        let index_data = serde_json::json!({plugin1_name: file1_path, plugin2_name: file2_path});

        assert_eq!(read_index, index_data);

        assert_eq!(
            paykit.delete_public_payment_endpoint(plugin1_name, Some(index_url)),
            Ok(index_url.to_string())
        );

        let read_index = paykit.transport.get(index_url).unwrap();
        assert_eq!(read_index, serde_json::json!({plugin2_name: file2_path}));

        let read_value = paykit.transport.get(file2_path);
        assert_eq!(read_value, Ok(plugin2_data));

        let read_value = paykit.transport.get(file1_path);
        assert!(read_value.unwrap_err().to_string().contains("No such file or directory"));
    }

    #[test]
    fn read_index() {
        let paykit = Paykit::new();
        let test_folder = Path::new(&env::temp_dir()).join("pdk_test").join("paykit").join("read_index");
        let index_url = test_folder.join("slashpay.json");
        let index_url: &str = index_url.to_str().unwrap();
        
        let plugin1_name: &str = "test1";
        let plugin1_data = serde_json::json!({ "data": "lnbcrt...1" });
        let data = serde_json::json!({plugin1_name: plugin1_data});

        assert_eq!(
            paykit.create_all(&data, Some(index_url)),
            Ok(test_folder.join("slashpay.json").to_str().unwrap().to_string())
        );

        let read_index = paykit.read_index(Some(index_url)).unwrap();
        let file1_path = test_folder.join(plugin1_name).join("slashpay.json");
        let file1_path: &str = file1_path.to_str().unwrap();

        assert_eq!(read_index, serde_json::json!({plugin1_name: file1_path}));

        std::fs::remove_file(index_url).unwrap();
        std::fs::remove_file(file1_path).unwrap();
    }

    #[test]
    fn read_payment_endpoint() {
        let paykit = Paykit::new();
        let test_folder = Path::new(&env::temp_dir()).join("pdk_test").join("paykit").join("read_payment_endpoint");
        let index_url = test_folder.join("slashpay.json");
        let index_url: &str = index_url.to_str().unwrap();

        let plugin1_name: &str = "test1";
        let plugin1_data = serde_json::json!({ "data": "lnbcrt...1" });

        assert_eq!(
            paykit.create_public_payment_endpoint(plugin1_name, &plugin1_data, Some(index_url)),
            Ok(test_folder.join(plugin1_name).join("slashpay.json").to_str().unwrap().to_string())
        );

        let file1_path = test_folder.join(plugin1_name).join("slashpay.json");
        let file1_path: &str = file1_path.to_str().unwrap();

        let read_value = paykit.read_payment_endpoint(file1_path).unwrap();
        assert_eq!(read_value, plugin1_data);

        std::fs::remove_file(index_url).unwrap();
        std::fs::remove_file(file1_path).unwrap();
    }

    #[test]
    fn read_payment_endpoint_by_name() {
        let paykit = Paykit::new();
        let test_folder = Path::new(&env::temp_dir()).join("pdk_test").join("paykit").join("read_payment_endpoint_by_name");
        let index_url = test_folder.join("slashpay.json");
        let index_url: &str = index_url.to_str().unwrap();

        let plugin1_name: &str = "test1";
        let plugin1_data = serde_json::json!({ "data": "lnbcrt...1" });

        assert_eq!(
            paykit.create_public_payment_endpoint(plugin1_name, &plugin1_data, Some(index_url)),
            Ok(test_folder.join(plugin1_name).join("slashpay.json").to_str().unwrap().to_string())
        );

        let read_value = paykit.read_payment_endpoint_by_name(plugin1_name, Some(index_url)).unwrap();
        assert_eq!(read_value, plugin1_data);

        std::fs::remove_file(index_url).unwrap();
        std::fs::remove_file(test_folder.join(plugin1_name).join("slashpay.json")).unwrap();
    }

    #[test]
    fn read_all() {
        let paykit = Paykit::new();
        let test_folder = Path::new(&env::temp_dir()).join("pdk_test").join("paykit").join("read_all");
        let index_url = test_folder.join("slashpay.json");
        let index_url: &str = index_url.to_str().unwrap();

        let plugin1_name: &str = "test1";
        let plugin1_data = serde_json::json!({ "data": "lnbcrt...1" });
        let plugin2_name: &str = "test2";
        let plugin2_data = serde_json::json!({ "data": "lnbcrt...2" });
        let data = serde_json::json!({plugin1_name: plugin1_data, plugin2_name: plugin2_data});

        assert_eq!(
            paykit.create_all(&data, Some(index_url)),
            Ok(test_folder.join("slashpay.json").to_str().unwrap().to_string())
        );

        let read_value = paykit.read_all(Some(index_url)).unwrap();
        assert_eq!(read_value, data);

        std::fs::remove_file(index_url).unwrap();
        std::fs::remove_file(test_folder.join(plugin1_name).join("slashpay.json")).unwrap();
        std::fs::remove_file(test_folder.join(plugin2_name).join("slashpay.json")).unwrap();
    }
}
