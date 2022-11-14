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
		let factor = Self::u64_to_balance_option(PERCENT_FACTOR);

		for (account_id, contribution) in Contributions::<T>::iter() {
			let share = factor.unwrap() * (contribution.clone().get_total_balance()) / amount;
			contribution_shares.push(ContributionShare {
				account_id,
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
		let mut contribution_list = Vec::new();

		for (account_id, balance) in contributions.into_iter() {
			let entry = Contributions::<T>::get(account_id.clone());
			ensure!(entry.is_some(), Error::<T>::NotAContributor);
			ensure!(entry.unwrap().can_reserve(balance), Error::<T>::NotEnoughAvailableBalance);

			Contributions::<T>::mutate(account_id.clone(), |val| {
				let mut unwrap_val = val.clone().unwrap();
				unwrap_val.reserve_amount(balance);
				let contribution = unwrap_val.clone();
				*val = Some(contribution);
			});
			contribution_list.push((account_id.clone(), balance));
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
		Contributions::<T>::iter().map(|(account_id, contribution)| (account_id, contribution)).collect()
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

		for (account_id, balance) in reservation.contributions.into_iter() {
			Contributions::<T>::mutate(account_id.clone(), |val| {
				let mut unwrap_val = val.clone().unwrap();
				unwrap_val.unreserve_amount(balance);
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

		// We tag the reserved amount in the contribution as used
		for (account_id, balance) in reservation.contributions.into_iter() {
			Contributions::<T>::mutate(account_id.clone(), |val| {
				let mut unwrap_val = val.clone().unwrap();
				unwrap_val.use_reserved_amount(balance);
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
