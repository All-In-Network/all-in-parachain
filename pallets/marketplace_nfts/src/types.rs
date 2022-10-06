use codec::{Decode, Encode};
use scale_info::TypeInfo;

/// Status types for different NFT
#[derive(Encode, Decode, Debug, Clone, PartialEq, TypeInfo)]
pub enum StatusType {
	ClaimSoulbound,
}
