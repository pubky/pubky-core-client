mod transport;
// use transport::Transport;

fn main() {
}


// #[derive(Serialize, Deserialize, Debug)]
// pub struct IndexPaymentEndpoint {
//     payment_endpoint: HashMap<String, String>
// }
// pub struct PaykitIndex(IndexPaymentEndpoint);
//
//
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let transport = Transport {};
//         let url = "/home/rxitech/Projects/Synonym/pdk/fixtures/slashpay.json";
//         let result = transport.get(url).unwrap();
//         // assert_eq!(result, String::from(url));
//     }
// }
//


  /* RECEIVER PERSPECTIVE: */
  // NOTE: index file is always auto updated

  /* PUBLIC PAYMENT ENDPOINT */
  // index_url = has default value
  // createAll ({plugin_name, plugin_data}, index_url = "/paykit.json") - return public index url
  // createPublicPaymentEndpoint(plugin_name, plugin_data, index_url = "/paykit.json") - return public index url
  // updatePulicPaymentEndpoint(plugin_name, plugin_data, index_url = "/paykit.json") - return public index url
  // deletePublicPaymentEndpoint(plugin_name, index_url = "/paykit.json") - return private index url

  /* PRIVATE PAYMENT ENDPOINT */
  // url for index file is always autoderived based on id
  // createAllPrivate ({plugin_name, plugin_data}, amount) - return public index url
  // createPrivatePaymentEndpoint(id, plugin_name, plugin_data, amount) - return private index url
  // updatePrivatePaymentEndpoint(id, plugin_name, plugin_data, amount) - return private index url
  // deletePrivatePaymentEndpoint(id, plugin name) - return private index url

  /* SENDER PERSPECTIVE: */
  /* PUBLIC AND PRIVATE PAYMENT ENDPOINT */
  // readAll(index_url) - return {plugin name, plugin data}

