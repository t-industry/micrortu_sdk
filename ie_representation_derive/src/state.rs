use micrortu_build_utils::{Block, BlockConf};
use std::{collections::BTreeMap, sync::Mutex};

static STRINGS: Mutex<String> = Mutex::new(String::new());
static BLOCK_CONFIGS: Mutex<BTreeMap<(String, String), BlockConf>> = Mutex::new(BTreeMap::new());
static BLOCKS: Mutex<BTreeMap<(String, String), Block>> = Mutex::new(BTreeMap::new());
static PARAMS: Mutex<BTreeMap<(String, String), Vec<micrortu_build_utils::Port>>> =
    Mutex::new(BTreeMap::new());
static PORTS: Mutex<BTreeMap<(String, String), Vec<micrortu_build_utils::Port>>> =
    Mutex::new(BTreeMap::new());

pub fn should_bail_on_duplicates() -> bool {
    std::env::var("MICRORTU_BAIL_ON_DUPLICATES").is_ok_and(|v| v == "1" || v == "true")
}

pub fn get_ports_params(
    block_name: &str,
) -> (
    Option<Vec<micrortu_build_utils::Port>>,
    Option<Vec<micrortu_build_utils::Port>>,
) {
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let key = (block_name.to_string(), crate_name.clone());
    let ports = PORTS.lock().expect("poison");
    let params = PARAMS.lock().expect("poison");
    let ports = ports.get(&key).cloned();
    let params = params.get(&key).cloned();

    (ports, params)
}

/// Returns previous ports, if any
pub fn set_ports(
    block_name: &str,
    ports: Vec<micrortu_build_utils::Port>,
) -> Option<Vec<micrortu_build_utils::Port>> {
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let key = (block_name.to_string(), crate_name.clone());
    PORTS.lock().expect("poison").insert(key, ports)
}

/// Returns previous params, if any
pub fn set_params(
    block_name: &str,
    params: Vec<micrortu_build_utils::Port>,
) -> Option<Vec<micrortu_build_utils::Port>> {
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let key = (block_name.to_string(), crate_name.clone());
    PARAMS.lock().expect("poison").insert(key, params)
}

pub fn get_block_conf(block_name: &str) -> Option<BlockConf> {
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let key = (block_name.to_string(), crate_name.clone());
    BLOCK_CONFIGS.lock().expect("poison").get(&key).cloned()
}

/// Returns previous block conf, if any
pub fn set_block_conf(block_name: &str, block_conf: BlockConf) -> Option<BlockConf> {
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let key = (block_name.to_string(), crate_name.clone());
    let mut prev = BLOCK_CONFIGS.lock().expect("poison");
    prev.insert(key, block_conf)
}

/// Returns previous block, if any
pub fn set_block(block_name: &str, block: Block) -> Option<Block> {
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let key = (block_name.to_string(), crate_name.clone());
    BLOCKS.lock().expect("poison").insert(key, block)
}

pub fn get_blocks() -> Vec<Block> {
    BLOCKS.lock().expect("poison").values().cloned().collect()
}

pub fn intern_static_string(s: &str) -> u16 {
    let mut strings = STRINGS.lock().expect("poison");
    let len = strings.len();
    strings.push_str(s);
    len.try_into().expect("too many strings interned")
}

pub fn get_interned_strings() -> String {
    STRINGS.lock().expect("poison").clone()
}
