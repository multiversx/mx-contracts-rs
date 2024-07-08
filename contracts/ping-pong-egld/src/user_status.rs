use multiversx_sc::derive_imports::*;

#[type_abi]
#[derive(TopEncode, TopDecode, PartialEq, Eq, Clone, Copy)]
pub enum UserStatus {
    New,
    Registered,
    Withdrawn,
}
