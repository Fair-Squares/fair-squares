pub use crate::structs::*;

impl<T: Config> Pallet<T> {
	// Conversion of u64 to BalanxceOf<T>
	pub fn u64_to_balance_option(input: u64) -> Option<BalanceOf<T>> {
		input.try_into().ok()
	}

	// Conversion of BalanceOf<T> to u32
	pub fn balance_to_u32_option(input: BalanceOf<T>) -> Option<u32> {
		input.try_into().ok()
	}

	pub fn fund_account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	pub fn get_contribution_share() -> Vec<ContributionShare<T>> {
		let mut contribution_shares = Vec::<ContributionShare<T>>::new();
		let amount = FundBalance::<T>::get().total;
		let contributions_iter = Contributions::<T>::iter();
		let factor = Self::u64_to_balance_option(PERCENT_FACTOR);

		for item in contributions_iter {
			let share = factor.unwrap() * (item.1.clone().get_total_balance()) / amount;
			contribution_shares.push(ContributionShare {
				account_id: item.1.account_id.clone(),
				share: Self::balance_to_u32_option(share).unwrap(),
			});
		}

		contribution_shares
	}

	/// Check that the fund can afford the amount
	pub fn check_available_fund(value: BalanceOf<T>) -> bool {
		let fund = FundBalance::<T>::get();

		fund.can_take_off(value)
	}

	/// Execute a bid on a house, funds are reserve for the bid before the transfer
	/// - account_id : account of the house seller
	/// - collection_id : id of a ollection of house type
	/// - item_id : id of the house in the collection
	/// - amount : amount used to buy the house
	/// - contributions : list of investors contributions
	/// Emits FundReservationSucceeded when successful
	pub fn house_bidding(
		nft_collection_id: NftCollectionId<T>,
		nft_item_id: NftItemId<T>,
		amount: BalanceOf<T>,
		contributions: Vec<(AccountIdOf<T>, BalanceOf<T>)>,
	) -> DispatchResultWithPostInfo {
		// Check that the fund can afford the bid
		let mut fund = FundBalance::<T>::get();

		ensure!(fund.can_take_off(amount), Error::<T>::NotEnoughFundForHouse);

		// Check the number of investors
		ensure!(
			contributions.len() <= T::MaxInvestorPerHouse::get().try_into().unwrap(),
			Error::<T>::NotMoreThanMaxInvestorPerHouse
		);

		// Checks that each contribution is possible
		let contribution_iter = contributions.iter();

		let mut contribution_list = Vec::new();

		for item in contribution_iter {
			let entry = Contributions::<T>::get(item.0.clone());
			ensure!(entry.is_some(), Error::<T>::NotAContributor);
			ensure!(entry.unwrap().can_reserve(item.1), Error::<T>::NotEnoughAvailableBalance);

			Contributions::<T>::mutate(item.0.clone(), |val| {
				let mut unwrap_val = val.clone().unwrap();
				unwrap_val.reserve_amount(item.1);
				let contribution = unwrap_val.clone();
				*val = Some(contribution);
			});
			contribution_list.push((item.0.clone(), item.1));
		}

		// The amount is tagged as reserved in the fund for the account_id
		T::LocalCurrency::reserve(&Self::fund_account_id(), amount)?;
		fund.reserve(amount);

		// The amount is reserved in the pot
		FundBalance::<T>::mutate(|val| {
			*val = fund.clone();
		});

		// Get the block number for timestamp
		let block_number = <frame_system::Pallet<T>>::block_number();

		let reservation = FundOperation {
			nft_collection_id,
			nft_item_id,
			amount,
			block_number,
			contributions: contribution_list,
		};

		// The reservation is added to the storage
		Reservations::<T>::insert((nft_collection_id, nft_item_id), reservation);

		// Emit an event.
		Self::deposit_event(Event::FundReservationSucceeded(
			nft_collection_id,
			nft_item_id,
			amount,
			block_number,
		));

		Ok(().into())
	}

	pub fn get_contributions() -> Vec<(AccountIdOf<T>, Contribution<T>)> {
		Contributions::<T>::iter().map(|elt| (elt.0, elt.1)).collect()
	}

	/// Cancel a house bidding
	/// The reserved funds from contributions are restored
	/// The Housing Fund pot is restored with the reserved amount
	pub fn cancel_house_bidding(
		nft_collection_id: NftCollectionId<T>,
		nft_item_id: NftItemId<T>,
	) -> DispatchResultWithPostInfo {
		let reservation_wrap = Reservations::<T>::get((nft_collection_id, nft_item_id));

		ensure!(reservation_wrap.is_some(), Error::<T>::NoFundReservationFound);

		let reservation = reservation_wrap.unwrap();

		let contributions_iter = reservation.contributions.iter();

		for item in contributions_iter {
			Contributions::<T>::mutate(item.0.clone(), |val| {
				let mut unwrap_val = val.clone().unwrap();
				unwrap_val.unreserve_amount(item.1);
				let contribution = unwrap_val.clone();
				*val = Some(contribution);
			});
		}

		let mut fund = FundBalance::<T>::get();
		T::LocalCurrency::unreserve(&Self::fund_account_id(), reservation.amount);
		fund.unreserve(reservation.amount);

		Reservations::<T>::remove((nft_collection_id, nft_item_id));

		// The amount is unreserved in the pot
		FundBalance::<T>::mutate(|val| {
			*val = fund.clone();
		});

		// Get the block number for timestamp
		let block_number = <frame_system::Pallet<T>>::block_number();

		// Emit an event.
		Self::deposit_event(Event::FundReservationCancelled(
			nft_collection_id,
			nft_item_id,
			reservation.amount,
			block_number,
		));

		Ok(().into())
	}

	/// Unreserved the amount of the house in the Housing fund
	pub fn unreserve_house_bidding_amount(
		nft_collection_id: NftCollectionId<T>,
		nft_item_id: NftItemId<T>,
	) -> DispatchResultWithPostInfo {
		let reservation_wrap = Reservations::<T>::get((nft_collection_id, nft_item_id));

		ensure!(reservation_wrap.is_some(), Error::<T>::NoFundReservationFound);

		let reservation = reservation_wrap.unwrap();

		let _fund = FundBalance::<T>::get();
		// The amount is unreserved in the currency pallet
		T::LocalCurrency::unreserve(&Self::fund_account_id(), reservation.amount);

		// Get the block number for timestamp
		let block_number = <frame_system::Pallet<T>>::block_number();

		// Emit an event.
		Self::deposit_event(Event::FundUnreservedForPurchase(
			nft_collection_id,
			nft_item_id,
			reservation.amount,
			block_number,
		));

		Ok(().into())
	}

	/// Move the reserved funds as purchased
	/// Unreserved fund from contributions and Fund
	/// Add operation in Purchases storage
	pub fn validate_house_bidding(
		nft_collection_id: NftCollectionId<T>,
		nft_item_id: NftItemId<T>,
	) -> DispatchResultWithPostInfo {
		let reservation_wrap = Reservations::<T>::get((nft_collection_id, nft_item_id));

		ensure!(reservation_wrap.is_some(), Error::<T>::NoFundReservationFound);

		let reservation = reservation_wrap.unwrap();

		let contributions_iter = reservation.contributions.iter();

		// We tag the reserved amount in the contribution as used
		for item in contributions_iter {
			Contributions::<T>::mutate(item.0.clone(), |val| {
				let mut unwrap_val = val.clone().unwrap();
				unwrap_val.use_reserved_amount(item.1);
				let contribution = unwrap_val.clone();
				*val = Some(contribution);
			});
		}

		let mut fund = FundBalance::<T>::get();
		// The amount is tagged as used for history
		fund.use_reserved(reservation.amount);

		// Delete from reservation
		Reservations::<T>::remove((nft_collection_id, nft_item_id));
		// Add to purchased operations
		Purchases::<T>::insert((nft_collection_id, nft_item_id), reservation.clone());

		// The amount is updated in the pot
		FundBalance::<T>::mutate(|val| {
			*val = fund.clone();
		});

		// Get the block number for timestamp
		let block_number = <frame_system::Pallet<T>>::block_number();

		// Emit an event.
		Self::deposit_event(Event::PurchaseFundValidated(
			nft_collection_id,
			nft_item_id,
			reservation.amount,
			block_number,
		));

		Ok(().into())
	}
}
