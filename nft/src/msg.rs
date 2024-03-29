use create_type_spec_derive::CreateTypeSpec;
use pbc_contract_common::address::{Address, Shortname};
use read_write_rpc_derive::ReadWriteRPC;

use rpc_msg_derive::IntoShortnameRPCEvent;
use utils::events::IntoShortnameRPCEvent;

/// This structure describes fields for NFT initialize msg
#[derive(ReadWriteRPC, CreateTypeSpec, Clone, PartialEq, Eq, Debug)]
pub struct NFTInitMsg {
    pub name: String,
    pub symbol: String,
    pub uri_template: String,
}

/// This structure describes fields for NFT transfer from msg
#[derive(ReadWriteRPC, CreateTypeSpec, IntoShortnameRPCEvent, Clone, PartialEq, Eq, Debug)]
#[rpc_msg(action = 0x03)]
pub struct NFTTransferFromMsg {
    /// owner address
    pub from: Address,
    /// receiver address
    pub to: Address,
    /// token id
    pub token_id: u128,
}

/// This structure describes fields for NFT approve msg
#[derive(ReadWriteRPC, CreateTypeSpec, IntoShortnameRPCEvent, Clone, PartialEq, Eq, Debug)]
#[rpc_msg(action = 0x05)]
pub struct NFTApproveMsg {
    pub approved: Option<Address>,
    /// token id
    pub token_id: u128,
}

/// This structure describes fields for NFT approve for all msg
#[derive(ReadWriteRPC, CreateTypeSpec, IntoShortnameRPCEvent, Clone, PartialEq, Eq, Debug)]
#[rpc_msg(action = 0x07)]
pub struct NFTApproveForAllMsg {
    /// operator address to approve
    pub operator: Address,
    pub approved: bool,
}

/// This structure describes fields for NFT mint msg
#[derive(ReadWriteRPC, CreateTypeSpec, IntoShortnameRPCEvent, Clone, PartialEq, Eq, Debug)]
#[rpc_msg(action = 0x09)]
pub struct NFTMintMsg {
    /// newly minted token id
    pub token_id: u128,
    /// receiver address
    pub to: Address,
    /// optional token uri
    pub token_uri: Option<String>,
}

/// This structure describes fields for mpc721 burn msg
#[derive(ReadWriteRPC, CreateTypeSpec, IntoShortnameRPCEvent, Clone, PartialEq, Eq, Debug)]
#[rpc_msg(action = 0x17)]
pub struct NFTBurnMsg {
    /// token id to burn
    pub token_id: u128,
}
