use const_default::ConstDefault;
use zerocopy::{AsBytes, FromBytes};

use crate::{
    SmallIE, C_SC_NA_1, C_SE_NB_1, C_SE_NC_1, M_DP_NA_1, M_ME_NB_1, M_ME_NE_1, M_SP_NA_1, P_ME_NC_1, TI136,
    TI137, TI138, TI139, TI200, TI201, TI202, TI203,
};

pub struct IE<const TYPECODE: u8>;

pub trait IEMeta:
    FromBytes
    + AsBytes
    + Into<SmallIE>
    + PartialEq
    + Copy
    + TryFrom<SmallIE>
    + TryFrom<f32>
    + core::fmt::Debug
    + Default
    + ConstDefault
{
    const TYPECODE: u8;
}
pub trait GetMeta {
    type Out: IEMeta;
}

macro_rules! impl_qie {
    ($($ie:ident => $code:expr,)*) => {
        $(
            impl GetMeta for IE<$code> {
                type Out = $ie;
            }
            impl GetMeta for $ie {
                type Out = $ie;
            }
            impl IEMeta for $ie {
                const TYPECODE: u8 = $code;
            }
        )*
    };
}

impl_qie! {
    M_SP_NA_1 => 1,
    M_DP_NA_1 => 3,
    M_ME_NB_1 => 11,
    M_ME_NE_1 => 13,
    C_SC_NA_1 => 45,
    C_SE_NB_1 => 49,
    C_SE_NC_1 => 50,
    P_ME_NC_1 => 112,
    TI136 => 136,
    TI137 => 137,
    TI138 => 138,
    TI139 => 139,
    TI200 => 200,
    TI201 => 201,
    TI202 => 202,
    TI203 => 203,
}
