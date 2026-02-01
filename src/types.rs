use core::mem::MaybeUninit;

//  SOLANA SPECIFIC
pub const DISCRIMINATOR_LEN: usize = size_of::<Discriminator>();
pub type Discriminator = u8;

//      RUST MEMORY
pub const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::<u8>::uninit();

//      RUST POD
//  UNSIGNED
pub const U64_LEN: usize = size_of::<u64>();
pub type U64Bytes = [u8; U64_LEN];

pub const U32_LEN: usize = size_of::<u32>();
pub type U32Bytes = [u8; U32_LEN];

pub const U16_LEN: usize = size_of::<u16>();
pub type U16Bytes = [u8; U16_LEN];

// SIGNED
pub const I64_LEN: usize = size_of::<i64>();
pub type I64Bytes = [u8; I64_LEN];

pub const I32_LEN: usize = size_of::<i32>();
pub type I32Bytes = [u8; I32_LEN];

pub const I16_LEN: usize = size_of::<i16>();
pub type I16Bytes = [u8; I16_LEN];

// FLOAT
pub const F64_LEN: usize = size_of::<f64>();
pub type F64Bytes = [u8; F64_LEN];

pub const F32_LEN: usize = size_of::<f32>();
pub type F32Bytes = [u8; F32_LEN];
