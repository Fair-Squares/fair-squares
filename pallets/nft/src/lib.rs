//! # NFT Pallet
//!
//! The NFT pallet provides supports for non fungible assets on Litentry
//!
//! ## Overview
//!
//! The NFT pallet enables third parties to issue (mainly identity related) non fungible assets.
//! Currently there are 3 types (check `ClassType`) of non fungible assets:
//! 1. Each instance is directly issued by the corresponding third party: Simple(u32)
//! 2. At issuance, a list of user is provided and only these users may claim: Claim(HashByte32)
//! 3. Can be minted only when the user have 2 specific base non fungible assets: Merge(ID, ID, bool)
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//! #### Class Issuance
//! * `create_class` - Create an NFT class (think the whole CryptoKitties or Hashmask each as a class)
//!
//! #### Instance Generation
//! * `mint` - Mint specified number of instance of `Simple(u32)` type
//! * `claim` - Whitelisted user claim an instance of `Claim(HashByte32)`, with a Merkle proof whose root
//! is the HashByte32
//! * `merge` - From two NFT instance, mint a new NFT instance of `Merge(ID, ID, bool)` type
//!
//! #### Daily User Actions
//! * `transfer` - Transfer ownership of a transferable NFT
//! * `burn` - Burn a burnable NFT
//!
//! [`Call`]: ./enum.Call.html
//! [`Config`]: ./trait.Config.html

#![cfg_attr(not(feature = "std"), no_std)]

use enumflags2::BitFlags;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement::KeepAlive, Get},
	transactional,
};
use frame_system::pallet_prelude::*;
use scale_info::{build::Fields, meta_type, Path, Type, TypeInfo, TypeParameter};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_io::hashing::keccak_256;
use sp_runtime::{traits::StaticLookup, DispatchResult, RuntimeDebug};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

pub mod benchmarking;
#[cfg(test)]
mod tests;
pub mod weights;

mod impl_nonfungibles;
pub mod merkle_proof;

pub use pallet::*;
pub use weights::WeightInfo;

pub type CID = Vec<u8>;

pub type HashByte32 = [u8; 32];

pub const CREATION_FEE: u32 = 100;

#[repr(u8)]
#[derive(Encode, Decode, Clone, Copy, BitFlags, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub enum ClassProperty {
	/// Token can be transferred
	Transferable = 0b00000001,
	/// Token can be burned
	Burnable = 0b00000010,
}

#[derive(Clone, Copy, PartialEq, Default, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Properties(pub BitFlags<ClassProperty>);

impl Eq for Properties {}
impl Encode for Properties {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.bits().using_encoded(f)
	}
}
impl Decode for Properties {
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let field = u8::decode(input)?;
		Ok(Self(<BitFlags<ClassProperty>>::from_bits(field as u8).map_err(|_| "invalid value")?))
	}
}

impl TypeInfo for Properties {
	type Identity = Self;

	fn type_info() -> Type {
		Type::builder()
			.path(Path::new("Properties", module_path!()))
			.type_params(vec![TypeParameter::new("T", Some(meta_type::<ClassProperty>()))])
			.composite(Fields::unnamed().field(|f| f.ty::<u8>().type_name("ClassProperty")))
	}
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ClassData<BN, ID> {
	/// Property of token
	pub properties: Properties,
	/// from when user can claim this nft
	pub start_block: Option<BN>,
	/// till when user can claim this nft
	pub end_block: Option<BN>,
	/// type of this NFT class
	pub class_type: ClassType<ID>,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TokenData {
	/// if token is used to generate an advanced nft
	pub used: bool,
	/// 0 = common, otherwise say 1 = rare, 2 = super rare
	pub rarity: u8,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ClassType<ID> {
	/// A class that owner can mint instances no more than u32
	Simple(u32),
	/// A class whitelisted user may claim provided a proof
	/// that indicates his/her account is in the Merkle tree with
	/// root HashByte32
	Claim(HashByte32),
	/// A class that is merged from two class ID and ID
	/// if true, burn the two instances
	Merge(ID, ID, bool),
}

pub type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
pub type Barak<T> = <T as frame_system::Config>::BlockNumber;
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ orml_nft::Config<
			ClassData = ClassData<BlockNumberOf<Self>, ClassIdOf<Self>>,
			TokenData = TokenData,
		>
	{
		/// The the currency to pay NFT class creation fee.
		type Currency: Currency<Self::AccountId>;

		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;

		/// The amount of fee to pay to create an NFT class.
		#[pallet::constant]
		type ClassCreationFee: Get<BalanceOf<Self>>;

		/// Treasury address
		#[pallet::constant]
		type Pot: Get<Self::AccountId>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// ClassId not found
		ClassIdNotFound,
		/// Class ClaimedList not found (Only for Claim type)
		ClassClaimedListNotFound,
		/// The operator is not the owner of the token and has no permission
		NoPermission,
		/// Quantity is invalid. need >= 1
		InvalidQuantity,
		/// Property of class don't support transfer
		NonTransferable,
		/// Property of class don't support burn
		NonBurnable,
		/// Token not found
		TokenNotFound,
		/// Wrong class type
		WrongClassType,
		/// Merge nft's base nfts are not provided correctly
		WrongMergeBase,
		/// Use already used token to merge new token
		TokenUsed,
		/// Mint more NFT than the maximum allowed
		QuantityOverflow,
		/// Out of NFT valid issuance period
		OutOfCampaignPeriod,
		/// NFT for certain user already claimed
		TokenAlreadyClaimed,
		/// user claim verification fails
		UserNotInClaimList,
		/// user cannot pay NFT class creation fee
		CreationFeeNotPaid,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Created NFT class. \[owner, class_id\]
		CreatedClass(T::AccountId, ClassIdOf<T>),
		/// Minted NFT token. \[from, to, class_id, start_token_id, quantity\]
		MintedToken(T::AccountId, T::AccountId, ClassIdOf<T>, TokenIdOf<T>, u32),
		/// Claimed NFT token. \[claimer, class_id, token_id\]
		ClaimedToken(T::AccountId, ClassIdOf<T>, TokenIdOf<T>),
		/// Merged NFT token. \[owner, class_id, token_id\]
		MergedToken(T::AccountId, ClassIdOf<T>, TokenIdOf<T>),
		/// Transferred NFT token. \[from, to, class_id, token_id\]
		TransferredToken(T::AccountId, T::AccountId, ClassIdOf<T>, TokenIdOf<T>),
		/// Burned NFT token. \[owner, class_id, token_id\]
		BurnedToken(T::AccountId, ClassIdOf<T>, TokenIdOf<T>),
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn claimed_list)]
	/// Claimed index vec for `Claim(HashByte32)` type NFT class,
	/// to guarantee each user claims once.
	/// maximal index of claiming user is 2^16 which is more than enough
	pub(super) type ClaimedList<T: Config> =
		StorageMap<_, Blake2_128Concat, ClassIdOf<T>, Vec<u16>, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create NFT class, each class is a collection of NFT instances.
		/// Currently there are 3 types (refer to `ClassType`)
		/// 1. Each instance is directly issued by the corresponding third party: Simple(u32)
		/// 2. At issuance, a list of user is provided and only these users may claim: Claim(HashByte32)
		/// 3. Can be minted only when the user have 2 specific base non fungible assets: Merge(ID, ID, bool)
		///
		/// Parameters:
		/// - `metadata`: CID identifier of the class's metadata
		/// - `properties`: Class property, include `Transferable` `Burnable`
		/// - `start_block`: From when the instances can be minted (None if no restriction)
		/// - `end_block`: Till when the instances can be minted (None if no restriction)
		/// - `class_type`: Type of this class (refer to `ClassType`)
		///
		/// Emits `CreatedClass` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::create_class())]
		#[transactional]
		pub fn create_class(
			origin: OriginFor<T>,
			metadata: CID,
			properties: Properties,
			start_block: Option<BlockNumberOf<T>>,
			end_block: Option<BlockNumberOf<T>>,
			class_type: ClassType<ClassIdOf<T>>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let next_id = orml_nft::Pallet::<T>::next_class_id();

			let fee = T::ClassCreationFee::get();
			T::Currency::transfer(&who, &T::Pot::get(), fee, KeepAlive)
				.map_err(|_| Error::<T>::CreationFeeNotPaid)?;

			match class_type {
				ClassType::Merge(id1, id2, burn) =>
					if !burn {
						ensure!(
							<orml_nft::Pallet<T>>::classes(id1).is_some(),
							Error::<T>::ClassIdNotFound
						);
						ensure!(
							<orml_nft::Pallet<T>>::classes(id2).is_some(),
							Error::<T>::ClassIdNotFound
						);
					} else {
						let class_info1 = orml_nft::Pallet::<T>::classes(id1)
							.ok_or(Error::<T>::ClassIdNotFound)?;
						let class_info2 = orml_nft::Pallet::<T>::classes(id2)
							.ok_or(Error::<T>::ClassIdNotFound)?;

						let data1 = class_info1.data;
						ensure!(
							data1.properties.0.contains(ClassProperty::Burnable),
							Error::<T>::NonBurnable
						);
						let data2 = class_info2.data;
						ensure!(
							data2.properties.0.contains(ClassProperty::Burnable),
							Error::<T>::NonBurnable
						);
					},
				ClassType::Claim(_) => {
					ClaimedList::<T>::insert(next_id, Vec::<u16>::new());
				},
				_ => {},
			}

			let data = ClassData { properties, start_block, end_block, class_type };
			orml_nft::Pallet::<T>::create_class(&who, metadata.to_vec(), data)?;

			Self::deposit_event(Event::CreatedClass(who, next_id));
			Ok(().into())
		}

		/// Mint `Simple(u32)` NFT instances from the class owner
		///
		/// Parameters:
		/// - `to`: The receiver of the minted NFTs
		/// - `class_id`: Identifier of the NFT class to mint
		/// - `metadata`: CID identifier of the instance's metadata
		/// - `quantity`: number of NFT to mint
		///
		/// Emits `MintedToken` event when successful
		#[pallet::weight(<T as Config>::WeightInfo::mint(*quantity))]
		#[transactional]
		pub fn mint(
			origin: OriginFor<T>,
			to: <T::Lookup as StaticLookup>::Source,
			class_id: ClassIdOf<T>,
			metadata: CID,
			quantity: u32,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			ensure!(quantity >= 1, Error::<T>::InvalidQuantity);
			let class_info =
				orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
			ensure!(who == class_info.owner, Error::<T>::NoPermission);
			ensure!(Self::check_time(&class_info.data), Error::<T>::OutOfCampaignPeriod);

			match class_info.data.class_type {
				ClassType::Simple(max_num) => {
					let issued = class_info.total_issuance;
					if TokenIdOf::<T>::from(quantity) > (TokenIdOf::<T>::from(max_num) - issued) {
						Err(Error::<T>::QuantityOverflow)?
					}
				},
				_ => Err(Error::<T>::WrongClassType)?,
			}

			// TODO: adjustible rarity
			let data = TokenData { used: false, rarity: 0 };
			let start_token_id =
				orml_nft::Pallet::<T>::mint(&to, class_id, metadata.clone(), data.clone())?;
			for _ in 1..quantity {
				orml_nft::Pallet::<T>::mint(&to, class_id, metadata.clone(), data.clone())?;
			}

			Self::deposit_event(Event::MintedToken(who, to, class_id, start_token_id, quantity));
			Ok(().into())
		}

		/// Claim a `Claim(HashByte32)` by a whitelisted user,
		/// with a Merkle proof that proves the user's account
		/// is in the Merkle tree of the given root
		///
		/// Parameters:
		/// - `index`: Index of user's Merkle proof
		/// - `class_id`: Identifier of the NFT class to mint
		/// - `proof`: Merkle proof
		///
		/// Emits `ClaimedToken` event when successful
		#[pallet::weight(<T as Config>::WeightInfo::mint(1))]
		#[transactional]
		pub fn claim(
			origin: OriginFor<T>,
			index: u16,
			class_id: ClassIdOf<T>,
			proof: Vec<HashByte32>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let class_info =
				orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;

			ensure!(ClaimedList::<T>::contains_key(class_id), Error::<T>::ClassClaimedListNotFound);

			ensure!(Self::check_time(&class_info.data), Error::<T>::OutOfCampaignPeriod);

			match class_info.data.class_type {
				ClassType::Claim(merkle_root) => {
					// check if this user has already claimed
					ensure!(
						!ClaimedList::<T>::get(class_id).contains(&index),
						Error::<T>::TokenAlreadyClaimed
					);

					// calculate hash for this user
					let mut bytes = index.encode();
					bytes.append(&mut who.encode());
					let computed_hash = keccak_256(&bytes);

					// verify the proof
					ensure!(
						merkle_proof::proof_verify(&computed_hash, &proof, &merkle_root),
						Error::<T>::UserNotInClaimList
					);

					// push this user's index into already claimed list
					ClaimedList::<T>::mutate(class_id, |claimed_vec| {
						claimed_vec.push(index);
					});
				},

				_ => Err(Error::<T>::WrongClassType)?,
			}

			// TODO: adjustable rarity
			let data = TokenData { used: false, rarity: 0 };

			// TODO: if metadata can change?
			let metadata = class_info.metadata;

			let next_token_id =
				orml_nft::Pallet::<T>::mint(&who, class_id, metadata.to_vec(), data)?;
			Self::deposit_event(Event::ClaimedToken(who, class_id, next_token_id));
			Ok(().into())
		}

		/// Merge from two NFT instances and generate a new NFT
		/// of type `Merge(ID, ID, bool)`
		///
		/// Parameters:
		/// - `class_id`: Identifier of the NFT class to mint
		/// - `token1`: First NFT of the merge base
		/// - `token2`: Seconde NFT of the merge base
		///
		/// Emits `MergedToken` event when successful
		#[pallet::weight(<T as Config>::WeightInfo::mint(1))]
		#[transactional]
		pub fn merge(
			origin: OriginFor<T>,
			class_id: ClassIdOf<T>,
			token1: (ClassIdOf<T>, TokenIdOf<T>),
			token2: (ClassIdOf<T>, TokenIdOf<T>),
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let merged_class_info =
				orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;

			ensure!(Self::check_time(&merged_class_info.data), Error::<T>::OutOfCampaignPeriod);

			let mut burn = false;

			if let ClassType::Merge(id1, id2, b) = merged_class_info.data.class_type {
				ensure!(
					((id1 == token1.0) && (id2 == token2.0)) ||
						((id1 == token2.0) && (id2 == token1.0)),
					Error::<T>::WrongMergeBase,
				);
				burn = b;
			} else {
				Err(Error::<T>::WrongClassType)?
			}

			// get token 1 and 2
			let mut token_info1 = <orml_nft::Pallet<T>>::tokens(token1.0, token1.1)
				.ok_or(Error::<T>::TokenNotFound)?;
			let mut token_info2 = <orml_nft::Pallet<T>>::tokens(token2.0, token2.1)
				.ok_or(Error::<T>::TokenNotFound)?;

			// burn or set used of token 1 and 2
			if burn {
				Self::do_burn(&who, token1)?;
				Self::do_burn(&who, token2)?;
			} else {
				ensure!(!token_info1.data.used && !token_info2.data.used, Error::<T>::TokenUsed);
				token_info1.data.used = true;
				token_info2.data.used = true;
				orml_nft::Tokens::<T>::insert(token1.0, token1.1, token_info1);
				orml_nft::Tokens::<T>::insert(token2.0, token2.1, token_info2);
			}

			// mint new token
			// TODO: adjustible rarity
			let data = TokenData { used: false, rarity: 0 };

			// TODO: if metadata can change?
			let metadata = merged_class_info.metadata;

			let next_token_id =
				orml_nft::Pallet::<T>::mint(&who, class_id, metadata.to_vec(), data)?;
			Self::deposit_event(Event::MergedToken(who, class_id, next_token_id));

			Ok(().into())
		}

		/// Transfer NFT token to another account, must be transferable
		///
		/// Parameters:
		/// - `to`: Receiver of the token
		/// - `token`: NFT instance to transfer
		///
		/// Emits `TransferredToken` event when successful
		#[pallet::weight(<T as Config>::WeightInfo::transfer())]
		#[transactional]
		pub fn transfer(
			origin: OriginFor<T>,
			to: <T::Lookup as StaticLookup>::Source,
			token: (ClassIdOf<T>, TokenIdOf<T>),
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			Self::do_transfer(&who, &to, token)?;
			Ok(().into())
		}

		/// Burn an NFT token instance, must be burnable
		///
		/// Parameters:
		/// - `token`: NFT instance to burn
		///
		/// Emits `BurnedToken` event when successful
		#[pallet::weight(<T as Config>::WeightInfo::burn())]
		#[transactional]
		pub fn burn(
			origin: OriginFor<T>,
			token: (ClassIdOf<T>, TokenIdOf<T>),
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::do_burn(&who, token)?;
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Ensured atomic.
	#[transactional]
	fn do_transfer(
		from: &T::AccountId,
		to: &T::AccountId,
		token: (ClassIdOf<T>, TokenIdOf<T>),
	) -> DispatchResult {
		let class_info =
			orml_nft::Pallet::<T>::classes(token.0).ok_or(Error::<T>::ClassIdNotFound)?;
		let data = class_info.data;
		ensure!(
			data.properties.0.contains(ClassProperty::Transferable),
			Error::<T>::NonTransferable
		);

		orml_nft::Pallet::<T>::transfer(from, to, token)?;

		Self::deposit_event(Event::TransferredToken(from.clone(), to.clone(), token.0, token.1));
		Ok(())
	}

	/// Ensured atomic.
	#[transactional]
	fn do_burn(who: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
		let class_info =
			orml_nft::Pallet::<T>::classes(token.0).ok_or(Error::<T>::ClassIdNotFound)?;
		let data = class_info.data;
		ensure!(data.properties.0.contains(ClassProperty::Burnable), Error::<T>::NonBurnable);

		let token_info =
			orml_nft::Pallet::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenNotFound)?;
		ensure!(*who == token_info.owner, Error::<T>::NoPermission);

		orml_nft::Pallet::<T>::burn(&who, token)?;

		Self::deposit_event(Event::BurnedToken(who.clone(), token.0, token.1));
		Ok(())
	}
}

impl<T: Config> Pallet<T> {
	/// check if current block time is in the range of the time span given by the
	/// token class info
	fn check_time(token_info: &ClassData<BlockNumberOf<T>, ClassIdOf<T>>) -> bool {
		let current_block_number = <frame_system::Pallet<T>>::block_number();
		if let Some(start_block) = token_info.start_block {
			if start_block > current_block_number {
				return false
			}
		}
		if let Some(end_block) = token_info.end_block {
			if end_block < current_block_number {
				return false
			}
		}
		true
	}
}
