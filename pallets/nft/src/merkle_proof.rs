use crate::HashByte32;
use sp_core::hash::H256;
use sp_io::hashing::keccak_256;
use sp_std::vec::Vec;

/// Verify the given Merkle proof and Merkle root
/// - Each pair of leaves and each pair of pre-images are assumed to be sorted.
/// - With reference of https://docs.openzeppelin.com/contracts/4.x/api/utils#MerkleProof
pub fn proof_verify(
	computed_hash: &HashByte32,
	proof: &Vec<HashByte32>,
	root: &HashByte32,
) -> bool {
	let mut next_hash = computed_hash.clone();

	for iter in proof {
		let iter_hash = H256::from_slice(iter);

		if iter_hash < H256::from_slice(&next_hash) {
			next_hash = keccak_256(&[iter_hash.as_bytes(), &next_hash].concat());
		} else {
			next_hash = keccak_256(&[&next_hash, iter_hash.as_bytes()].concat());
		}
	}

	next_hash == *root
}
