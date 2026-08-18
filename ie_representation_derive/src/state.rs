use micrortu_build_utils::{BlockConf, Port};
use std::{collections::BTreeMap, fmt, sync::Mutex};

static BLOCK_CONFIGS: Mutex<BTreeMap<Key, Entry<BlockConf>>> = Mutex::new(BTreeMap::new());
static BLOCKS: Mutex<BTreeMap<Key, Entry<BlockFns>>> = Mutex::new(BTreeMap::new());
static PARAMS: Mutex<BTreeMap<Key, Entry<Vec<Port>>>> = Mutex::new(BTreeMap::new());
static PORTS: Mutex<BTreeMap<Key, Entry<Vec<Port>>>> = Mutex::new(BTreeMap::new());
static FINALIZED: Mutex<BTreeMap<String, Loc>> = Mutex::new(BTreeMap::new());

type Key = (String, String); // (block name, crate name)

/// Source location of a macro invocation.
///
/// Registrations are keyed by it so that a re-expansion of the same invocation
/// (as done by long-lived macro hosts like rust-analyzer's proc-macro server)
/// stays idempotent, while a second registration of the same block from a
/// different place in the source is reported as a conflict.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Loc {
    file: String,
    line: usize,
    column: usize,
}

impl Loc {
    pub fn of(span: proc_macro2::Span) -> Self {
        if proc_macro::is_available() {
            let span = span.unwrap();
            Self {
                file: span.file(),
                line: span.line(),
                column: span.column(),
            }
        } else {
            Self {
                file: String::new(),
                line: 0,
                column: 0,
            }
        }
    }
}

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

struct Entry<T> {
    value: T,
    loc: Loc,
}

/// The identifiers a block was registered with. Imported blocks carry a
/// marker instead, so re-registering an imported block under the same name
/// as a local one is detected as a conflict.
#[derive(Clone, PartialEq, Eq)]
pub struct BlockFns {
    pub block_type: String,
    pub factory: String,
    pub init: String,
    pub step: String,
}

impl BlockFns {
    pub fn imported() -> Self {
        Self {
            block_type: String::new(),
            factory: String::new(),
            init: String::new(),
            step: String::new(),
        }
    }
}

fn crate_name() -> String {
    std::env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME is set by cargo")
}

/// Inserts a registration.
///
/// Re-registration is accepted when it comes from the same location (a
/// re-expansion) or carries an identical value (in which case the generated
/// code collides on its own and rustc reports the duplicate items). A
/// different value from a different location is a conflict, reported with the
/// location of the existing registration.
fn insert<T: PartialEq>(
    map: &Mutex<BTreeMap<Key, Entry<T>>>,
    block_name: &str,
    value: T,
    loc: Loc,
) -> Result<(), Loc> {
    let key = (block_name.to_string(), crate_name());
    let mut map = map.lock().expect("poison");
    if let Some(prev) = map.get(&key) {
        if prev.loc != loc && prev.value != value {
            return Err(prev.loc.clone());
        }
    }
    map.insert(key, Entry { value, loc });
    Ok(())
}

fn get<T: Clone>(map: &Mutex<BTreeMap<Key, Entry<T>>>, block_name: &str) -> Option<T> {
    let key = (block_name.to_string(), crate_name());
    map.lock()
        .expect("poison")
        .get(&key)
        .map(|entry| entry.value.clone())
}

pub fn get_ports_params(block_name: &str) -> (Option<Vec<Port>>, Option<Vec<Port>>) {
    (get(&PORTS, block_name), get(&PARAMS, block_name))
}

pub fn set_ports(block_name: &str, ports: Vec<Port>, loc: Loc) -> Result<(), Loc> {
    insert(&PORTS, block_name, ports, loc)
}

pub fn set_params(block_name: &str, params: Vec<Port>, loc: Loc) -> Result<(), Loc> {
    insert(&PARAMS, block_name, params, loc)
}

pub fn get_block_conf(block_name: &str) -> Option<BlockConf> {
    get(&BLOCK_CONFIGS, block_name)
}

pub fn set_block_conf(block_name: &str, block_conf: BlockConf, loc: Loc) -> Result<(), Loc> {
    insert(&BLOCK_CONFIGS, block_name, block_conf, loc)
}

pub fn set_block(block_name: &str, fns: BlockFns, loc: Loc) -> Result<(), Loc> {
    insert(&BLOCKS, block_name, fns, loc)
}

/// The blocks registered by the current crate, sorted by name, with the
/// location of each `register_block!` invocation.
pub fn registered_blocks() -> Vec<(String, Loc)> {
    let krate = crate_name();
    BLOCKS
        .lock()
        .expect("poison")
        .iter()
        .filter(|((_, entry_crate), _)| *entry_crate == krate)
        .map(|((name, _), entry)| (name.clone(), entry.loc.clone()))
        .collect()
}

/// Marks the current crate as finalized, refusing a second `finalize!` at a
/// different location.
pub fn mark_finalized(loc: Loc) -> Result<(), Loc> {
    let mut map = FINALIZED.lock().expect("poison");
    match map.get(&crate_name()) {
        Some(prev) if *prev != loc => Err(prev.clone()),
        _ => {
            map.insert(crate_name(), loc);
            Ok(())
        }
    }
}
