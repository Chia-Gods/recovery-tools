use std::env;
use std::path::Path;
use dg_xch_clients::{ClientSSLConfig};
use dg_xch_clients::rpc::full_node::FullnodeClient;

pub fn get_chia_client(port: u16) -> FullnodeClient {
    let chia_root = env::var("CHIA_ROOT").unwrap_or_else(|_| {
        let home_dir = dirs_next::home_dir().unwrap();
        format!("{}/.chia/mainnet", home_dir.to_str().unwrap())
    });
    let chia_root_path = Path::new(&chia_root);

    let none_var = None;
    FullnodeClient::new("localhost", port, 60, Some(ClientSSLConfig{
        ssl_crt_path: String::from(chia_root_path.join("config/ssl/full_node/private_full_node.crt").to_str().unwrap()),
        ssl_key_path: String::from(chia_root_path.join("config/ssl/full_node/private_full_node.key").to_str().unwrap()),
        ssl_ca_crt_path: String::from(chia_root_path.join("config/ssl/ca/private_ca.crt").to_str().unwrap()),
    }), &none_var)
}