use micrortu_build_utils::{Block, BlockConf};
use std::{
    collections::BTreeMap,
    sync::Mutex,
};

static STRINGS: Mutex<String> = Mutex::new(String::new());
static BLOCK_CONFIGS: Mutex<BTreeMap<(String, String), BlockConf>> = Mutex::new(BTreeMap::new());
static BLOCKS: Mutex<BTreeMap<(String, String), Block>> = Mutex::new(BTreeMap::new());
static PARAMS: Mutex<BTreeMap<(String, String), Vec<micrortu_build_utils::Port>>> =
    Mutex::new(BTreeMap::new());
static PORTS: Mutex<BTreeMap<(String, String), Vec<micrortu_build_utils::Port>>> =
    Mutex::new(BTreeMap::new());

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

pub fn set_ports(block_name: &str, ports: Vec<micrortu_build_utils::Port>) -> Result<(), ()> {
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let key = (block_name.to_string(), crate_name.clone());
    let prev = PORTS.lock().expect("poison").insert(key, ports);
    if prev.is_some() {
        return Err(());
    }
    Ok(())
}

pub fn set_params(block_name: &str, params: Vec<micrortu_build_utils::Port>) -> Result<(), ()> {
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let key = (block_name.to_string(), crate_name.clone());
    let prev = PARAMS.lock().expect("poison").insert(key, params);
    if prev.is_some() {
        return Err(());
    }
    Ok(())
}

pub fn get_block_conf(block_name: &str) -> Option<BlockConf> {
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let key = (block_name.to_string(), crate_name.clone());
    BLOCK_CONFIGS.lock().expect("poison").get(&key).cloned()
}

pub fn set_block_conf(block_name: &str, block_conf: BlockConf) -> Result<(), ()> {
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let key = (block_name.to_string(), crate_name.clone());
    let mut prev = BLOCK_CONFIGS.lock().expect("poison");
    if prev.insert(key, block_conf).is_some() {
        return Err(());
    }
    Ok(())
}

pub fn set_block(block_name: &str, block: Block) -> Result<(), ()> {
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let key = (block_name.to_string(), crate_name.clone());
    let prev = BLOCKS.lock().expect("poison").insert(key, block);
    if prev.is_some() {
        return Err(());
    }
    Ok(())
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
