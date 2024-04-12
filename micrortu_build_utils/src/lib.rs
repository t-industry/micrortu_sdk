use std::num::NonZeroU8;

use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};
use validator::Validate;
use wasm_global_shared_data::{NativeBindingDefinition, REQUIRED};

#[derive(Serialize, JsonSchema, Deserialize, Debug)]
pub struct WasmMetadata {
    pub minimum_firmware_version: (u8, u8, u8),
    pub sdk_version: (u8, u8, u8),
    pub blocks: Vec<Block>,
}

#[derive(Serialize, JsonSchema)]
pub struct FirmwareDump {
    pub version: Version,
    pub blocks: Vec<Block>,
}

#[derive(Serialize, JsonSchema, Validate, Deserialize, Clone, Debug)]
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
    pub block_conf: Option<BlockConf>,
}

#[derive(Serialize, JsonSchema, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    In,
    Out,
    InOut,
}

#[derive(Serialize, JsonSchema, Deserialize, Clone, Copy, Debug)]
#[repr(u8)]
#[serde(rename_all = "lowercase")]
pub enum IEType {
    TI1 = 1,
    TI3 = 3,
    TI13 = 13,
    TI45 = 45,
    TI49 = 49,
    TI50 = 50,
    TI112 = 112,
    TI136 = 136,
    TI137 = 137,
    TI138 = 138,
    TI139 = 139,
}

#[derive(Serialize, JsonSchema, Validate, Deserialize, Clone, Debug)]
pub struct Port {
    #[validate(length(min = 1, max = 32))]
    pub name: String,
    #[serde(rename = "type")]
    pub typ: IEType,
    pub description: String,
    pub direction: Direction,
    pub required: bool,
    pub min: NonZeroU8,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<NonZeroU8>,
}

impl TryFrom<u8> for IEType {
    type Error = ConvertError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::TI1),
            3 => Ok(Self::TI3),
            13 => Ok(Self::TI13),
            45 => Ok(Self::TI45),
            49 => Ok(Self::TI49),
            50 => Ok(Self::TI50),
            112 => Ok(Self::TI112),
            136 => Ok(Self::TI136),
            137 => Ok(Self::TI137),
            138 => Ok(Self::TI138),
            139 => Ok(Self::TI139),
            _ => Err(ConvertError),
        }
    }
}

#[derive(Debug)]
pub struct ConvertError;

impl TryFrom<NativeBindingDefinition<'static>> for Port {
    type Error = ConvertError;
    fn try_from(value: NativeBindingDefinition) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name.to_string(),
            typ: value.typ.try_into()?,
            description: String::new(),
            direction: match value.direction.0 {
                0 => Direction::In,
                1 => Direction::Out,
                2 => Direction::InOut,
                _ => return Err(ConvertError),
            },
            required: value.flags & REQUIRED != 0,
            min: value.min_size.try_into().map_err(|_| ConvertError)?,
            max: value.max_size,
        })
    }
}

#[derive(Serialize, JsonSchema, Deserialize, Clone, Copy, Debug)]
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

#[derive(Serialize, JsonSchema, Validate, Deserialize, Clone, Debug)]
pub struct BlockConf {
    pub fields: Vec<(String, AllowedType)>,
}
