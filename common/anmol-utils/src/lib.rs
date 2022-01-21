#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::vec::Vec;

pub fn remove_vector_item<'a, T: Ord>(vector: &'a mut Vec<T>, item: &T) -> Result<T, &'static str> {
	match vector.binary_search(item) {
		Ok(index) => Ok(vector.remove(index)),
		Err(_) => Err("RemoveVectorItem"),
	}
}
