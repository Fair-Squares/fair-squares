//! # Non Fungible Token
//! The module provides implementations for non-fungible-token.
//!
//! - [`Config`](./trait.Config.html)
//! - [`Call`](./enum.Call.html)
//! - [`Module`](./struct.Module.html)
//!
//! ## Overview
//!
//! This module provides basic functions to create and manager
//! NFT(non fungible token) such as `create_class`, `transfer`, `mint`, `burn`.

//! ### Module Functions
//!
//! - `create_class` - Create NFT(non fungible token) class
//! - `transfer` - Transfer NFT(non fungible token) to another account.
//! - `mint` - Mint NFT(non fungible token)
//! - `burn` - Burn NFT(non fungible token)
//! - `destroy_class` - Destroy NFT(non fungible token) class

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

pub use anmol_utils;
use codec::{Decode, Encode};
use frame_support::{ensure, pallet_prelude::*, traits::Get, BoundedVec, Parameter};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member, One, Zero},
	ArithmeticError, DispatchError, DispatchResult, RuntimeDebug,
};

use sp_std::convert::TryInto;
use sp_std::vec::Vec;

mod mock;
mod tests;
//type AccountOf<T> = <T as frame_system::Config>::AccountId;

/// Class info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ClassInfo<TokenId, AccountId, Data, ClassMetadataOf> {
	/// Class metadata
	pub metadata: ClassMetadataOf,
	/// Total issuance for the class
	pub total_issuance: TokenId,
	/// Class owner
	pub owner: AccountId,
	/// Class Properties
	pub data: Data,
}

/// Token info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct TokenInfo<AccountId, Data> {
	/// Token metadata
	pub metadata: Vec<u8>,
	/// Token owner
	pub owners: Vec<AccountId>,
	/// Token Properties
	pub data: Data,
}


pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The class ID type
		type ClassId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
		/// The token ID type
		type TokenId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
		/// The class properties type
		type ClassData: Parameter + Member + MaybeSerializeDeserialize + Default;
		/// The token properties type
		type TokenData: Parameter + Member + MaybeSerializeDeserialize + Default;
		/// The maximum size of a class's metadata
		type MaxClassMetadata: Get<u32>;
		/// The maximum size of a token's metadata
		type MaxTokenMetadata: Get<u32>;
	}

	pub type ClassMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxClassMetadata>;
	pub type TokenMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxTokenMetadata>;
	pub type ClassInfoOf<T> = ClassInfo<
		<T as Config>::TokenId,
		<T as frame_system::Config>::AccountId,
		<T as Config>::ClassData,
		ClassMetadataOf<T>,
	>;
	pub type TokenInfoOf<T> =
		TokenInfo<<T as frame_system::Config>::AccountId, <T as Config>::TokenData>;

	pub type GenesisTokenData<T> = (
		<T as frame_system::Config>::AccountId, // Token owner
		Vec<u8>,                                // Token metadata
		<T as Config>::TokenData,
	);
	pub type GenesisTokens<T> = (
		<T as frame_system::Config>::AccountId, // Token class owner
		Vec<u8>,                                // Token class metadata
		<T as Config>::ClassData,
		Vec<GenesisTokenData<T>>, // Vector of tokens belonging to this class
	);

	/// Error for non-fungible-token module.
	#[pallet::error]
	pub enum Error<T> {
		/// No available class ID
		NoAvailableClassId,
		/// No available token ID
		NoAvailableTokenId,
		/// Token(ClassId, TokenId) not found
		TokenNotFound,
		/// Class not found
		ClassNotFound,
		/// The operator is not the owner of the token and has no permission
		NoPermission,
		/// Can not destroy class
		/// Total issuance is not 0
		CannotDestroyClass,
		/// Failed because the Maximum amount of metadata was exceeded
		MaxMetadataExceeded,
        /// Sender tried to send more ownership than they have
		SenderInsufficientPercentage,
        /// Wrong arguments
		WrongArguments,
	}

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config>{
        /// A new NFT class was successfully created. \[owner, NFT_id\]
        CreatedId(T::AccountId, T::ClassId),
        /// A new NFT was successfully minted. \[owner, Token_id\]
        Minted(T::AccountId, T::TokenId),
        /// NFT tokens were successfully transferred. \[from, to, NFT_id_&_Token_id, percentage\]
        Transferred(T::AccountId, T::AccountId, (T::ClassId, T::TokenId),u8),

    }


	/// Next available class ID.
	#[pallet::storage]
	#[pallet::getter(fn next_class_id)]
	pub type NextClassId<T: Config> = StorageValue<_, T::ClassId, ValueQuery>;

	/// Next available token ID.
	#[pallet::storage]
	#[pallet::getter(fn next_token_id)]
	pub type NextTokenId<T: Config> = StorageMap<_, Twox64Concat, T::ClassId, T::TokenId, ValueQuery>;

	/// Store class info.
	///
	/// Returns `None` if class info not set or removed.
	#[pallet::storage]
	#[pallet::getter(fn classes)]
	pub type Classes<T: Config> = StorageMap<_, Twox64Concat, T::ClassId, ClassInfoOf<T>>;

	/// Store token info.
	///
	/// Returns `None` if token info not set or removed.
	#[pallet::storage]
	#[pallet::getter(fn tokens)]
	pub type Tokens<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::ClassId, Twox64Concat, T::TokenId, TokenInfoOf<T>>;

	#[derive(Default, Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
	#[derive(TypeInfo)]
	pub struct TokenByOwnerData {
		pub percent_owned: u8,
		pub token_id: u32,
}

	


        /// Token existence check by owner and class ID.
	#[pallet::storage]
	
	#[pallet::getter(fn tokens_by_owner)]
	pub type TokensByOwner<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		(T::ClassId, T::TokenId),
        TokenByOwnerData,
		ValueQuery,
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub tokens: Vec<GenesisTokens<T>>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { tokens: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			self.tokens.iter().for_each(|token_class| {
				let class_id = Pallet::<T>::create_class(&token_class.0, token_class.1.to_vec(), token_class.2.clone())
					.expect("Create class cannot fail while building genesis");
				for (account_id, token_metadata, token_data) in &token_class.3 {
					Pallet::<T>::mint(account_id, class_id, token_metadata.to_vec(), token_data.clone())
						.expect("Token mint cannot fail during genesis");
				}
			})
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	/// Create NFT(non fungible token) class
	pub fn create_class(
		owner: &T::AccountId,
		metadata: Vec<u8>,
		data: T::ClassData,
	) -> Result<T::ClassId, DispatchError> {
		let bounded_metadata: BoundedVec<u8, T::MaxClassMetadata> =
			metadata.try_into().map_err(|_| Error::<T>::MaxMetadataExceeded)?;

		let class_id = NextClassId::<T>::try_mutate(|id| -> Result<T::ClassId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableClassId)?;
            
			Ok(current_id)
		})?;

		let info = ClassInfo {
			metadata: bounded_metadata,
			total_issuance: Default::default(),
			owner: owner.clone(),
			data,
		};
        // Logging to the console
        log::info!("A NFT was created with ID: {:?}.", class_id);
        //Self::deposit_event(Event::CreatedId(owner,class_id));
		Classes::<T>::insert(class_id, info);

		Ok(class_id)
	}

	/// Transfer NFT(non fungible token) from `from` account to `to` account
	pub fn transfer(
		from: &T::AccountId,
		to: &T::AccountId,
		token: (T::ClassId, T::TokenId),
		percentage: u8,
	) -> DispatchResult {
		if from == to {
			return Ok(());
		}

		ensure!(percentage > 0, Error::<T>::WrongArguments);

		Tokens::<T>::try_mutate(token.0, token.1, |token_info| -> DispatchResult {
			let token_info_value = token_info.as_mut().ok_or(Error::<T>::TokenNotFound)?;

			ensure!(
				token_info_value.owners.contains(from),
				Error::<T>::NoPermission
			);

			TokensByOwner::<T>::try_mutate_exists(from, token, |sender_token| -> DispatchResult {
				let sender_token_value = sender_token
					.as_mut()
					.ok_or(Error::<T>::SenderInsufficientPercentage)?;

				ensure!(
					sender_token_value.percent_owned >= percentage,
					Error::<T>::SenderInsufficientPercentage
				);

				sender_token_value.percent_owned -= percentage;
				if sender_token_value.percent_owned == 0 {
					// remove sender from TokensByOwner if precent_owned is 0
					*sender_token = None;
					// remove sender from token.owners
					anmol_utils::remove_vector_item(&mut token_info_value.owners, from)?;
				}

				TokensByOwner::<T>::mutate(to, token, |recipient_token| -> DispatchResult {
					recipient_token.percent_owned += percentage;
					if let Err(pos) = token_info_value.owners.binary_search(&to) {
						let owners_token = to.clone();
						token_info_value.owners.insert(pos, owners_token)
					}
					Ok(())
				})
			})
		})
	}


	/// Mint NFT(non fungible token) to `owner`
	pub fn mint(
		owner:&T::AccountId,
		class_id: T::ClassId,
		metadata: Vec<u8>,
		data: T::TokenData,
	) -> Result<T::TokenId, DispatchError> {
		NextTokenId::<T>::try_mutate(class_id, |id| -> Result<T::TokenId, DispatchError> {
			//let bounded_metadata: BoundedVec<u8, T::MaxTokenMetadata> =
				//metadata.try_into().map_err(|_| Error::<T>::MaxMetadataExceeded)?;

			let token_id = *id;
			*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableTokenId)?;

			Classes::<T>::try_mutate(class_id, |class_info| -> DispatchResult {
				let info = class_info.as_mut().ok_or(Error::<T>::ClassNotFound)?;
				info.total_issuance = info
					.total_issuance
					.checked_add(&One::one())
					.ok_or(ArithmeticError::Overflow)?;
				Ok(())
			})?;

			let token_info = TokenInfo {
				metadata,
				owners: [owner.clone()].to_vec(),
				data,
			};

			Tokens::<T>::insert(class_id, token_id, token_info);

			//TokensByOwner::<T>::insert((owner, class_id, token_id),
            match TryInto::<u32>::try_into(token_id){
                Err(_) => Err(DispatchError::Arithmetic(ArithmeticError::Overflow)),
                Ok(token) => {
                    TokensByOwner::<T>::insert(owner, (class_id, token_id),
                TokenByOwnerData{
                    percent_owned:100,
                    token_id: token,
                },
            );
            log::info!("A NFT was minted with ID: {:?}.", token_id);
            //Self::deposit_event(Event::Minted(owner,token_id));
            Ok(token_id)
        }

            }
        
            
        })
        

		}

	/// Burn NFT(non fungible token) from `owner`
	pub fn burn(owner:&T::AccountId, token: (T::ClassId, T::TokenId)) -> DispatchResult {
		Tokens::<T>::try_mutate_exists(token.0, token.1, |token_info| -> DispatchResult {
			let t = token_info.take().ok_or(Error::<T>::TokenNotFound)?;
			ensure!(t.owners.contains(owner), Error::<T>::NoPermission);

			Classes::<T>::try_mutate(token.0, |class_info| -> DispatchResult {
				let info = class_info.as_mut().ok_or(Error::<T>::ClassNotFound)?;
				info.total_issuance = info
					.total_issuance
					.checked_sub(&One::one())
					.ok_or(ArithmeticError::Overflow)?;
				Ok(())
			})?;

			TokensByOwner::<T>::remove(owner, token);

			Ok(())
		})
	}

	/// Destroy NFT(non fungible token) class
	pub fn destroy_class(owner:&T::AccountId, class_id: T::ClassId) -> DispatchResult {
		Classes::<T>::try_mutate_exists(class_id, |class_info| -> DispatchResult {
			let info = class_info.take().ok_or(Error::<T>::ClassNotFound)?;
			ensure!(info.owner == *owner, Error::<T>::NoPermission);
			ensure!(info.total_issuance == Zero::zero(), Error::<T>::CannotDestroyClass);

			NextTokenId::<T>::remove(class_id);

			Ok(())
		})
	}

	pub fn is_owner(account: &T::AccountId, token: (T::ClassId, T::TokenId)) -> bool {
		//return Tokens::<T>::get(token.0, token.1).map_or(false, |token| token.owner == *account);
		TokensByOwner::<T>::contains_key(account, token)
	}
}
