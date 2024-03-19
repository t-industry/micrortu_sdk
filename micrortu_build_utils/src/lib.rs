use std::num::NonZeroU8;

use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, JsonSchema, Deserialize)]
pub struct WasmBlobDump {
    pub blocks: Vec<Block>,
}

#[derive(Serialize, JsonSchema)]
pub struct FirmwareDump {
    pub version: Version,
    pub blocks: Vec<Block>,
}

#[derive(Serialize, JsonSchema, Validate, Deserialize, Clone)]
pub struct Block {
    #[validate(length(min = 1, max = 32))]
    pub name: String,
    pub description: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semver_requirement: Option<String>,
    pub ports: Vec<Port>,
    pub params: Vec<Port>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_conf: Option<BlockConf>
}

#[derive(Serialize, JsonSchema, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    In,
    Out,
    InOut,
}

#[derive(Serialize, JsonSchema, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum IEType {
    TI1,
    TI3,
    TI13,
    TI45,
    TI50,
    TI112,
}

#[derive(Serialize, JsonSchema, Validate, Deserialize, Clone)]
pub struct Port {
    #[validate(length(min = 1, max = 32))]
    pub name: String,
    #[serde(rename = "type")]
    pub typ: IEType,
    pub description: String,
    pub direction: Direction,
    pub required: bool,
    pub min: NonZeroU8,
    pub max: NonZeroU8,
}

#[derive(Serialize, JsonSchema, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum AllowedType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}

#[derive(Serialize, JsonSchema, Validate, Deserialize, Clone)]
pub struct BlockConf {
    pub fields: Vec<(String, AllowedType)>,
}
