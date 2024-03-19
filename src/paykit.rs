use crate::transport::Transport;
use std::collections::HashMap;
use serde_json::Value;

const INDEX_URL: &str = "slashpay.json";

struct Paykit {
    transport: Transport
    // index_url: String
}

// struct PluginData {}

// struct PaykitEndpoint { HashMap<String, PluginData> }

impl Paykit {
    pub fn new() -> Paykit {
        Paykit { transport: Transport {} }
    }
    /* ------ RECEIVER PERSPECTIVE: ------ */
    // NOTE: index file is always auto updated

    /* PUBLIC PAYMENT ENDPOINT */

    /// Creates a new public payment endpoint for each plugin in the plugin_map, filling the
    /// content with the plugin data. It stores links to each plugin related file in index file
    /// accessible via `index_url` and returns index url as a result.
    fn create_all(&self, plugin_map: &Value, index_url: Option<&str>) -> Result<String, String> {
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
    fn create_public_payment_endpoint(&self, plugin_name: &str, plugin_data: &Value, index_url: Option<&str>) -> Result<String, String> {
        let index_url = Self::get_url(index_url);
        let path = Transport::get_path(&plugin_name, Some(index_url), None);
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

// updatePulicPaymentEndpoint(plugin_name: String, plugin_data: Value, index_url: Option(Stirng)) - return public index url
//

// deletePublicPaymentEndpoint(plugin_name: String, index_url: Option(String)) - return private index url
//

/* PRIVATE PAYMENT ENDPOINT */
// NOTE: url for index file is always autoderived based on id

// createAllPrivate (PluginMap: HashMap<String, PluginData>, amount: u8) - return public index url
//

// createPrivatePaymentEndpoint(id: String, plugin_name: String, plugin_data: Value, amount: u8) - return private index url
//

// updatePrivatePaymentEndpoint(id: String, plugin_name: String, plugin_data: Value, amount: u8) - return private index url
//

// deletePrivatePaymentEndpoint(id: String, plugin_name: String) - return private index url
//

/* SENDER PERSPECTIVE: */
/* PUBLIC AND PRIVATE PAYMENT ENDPOINT */
// readAll(index_url: Option(String)) - return {plugin name, plugin data}
//

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
}
