pub use super::*;

impl<T: Config> Pallet<T> {
	

	pub fn fund_account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	

	/// Check that the fund can afford the amount
	pub fn check_available_fund(value: BalanceOf<T>) -> bool {
		let fund_account = Self::fund_account_id();
		let amount = T::LocalCurrency::free_balance(&fund_account);

		amount>value
	}

	pub fn get_contributions() -> Vec<(AccountIdOf<T>, UserFundStatus<T>)> {
		Contributions::<T>::iter()
			.map(|(account_id, contribution)| (account_id, contribution))
			.collect()
	}

	pub fn get_contribution_share() -> Vec<ContributionShare<T>> {
		let mut contribution_shares = Vec::<ContributionShare<T>>::new();
		let fund_account = Self::fund_account_id();
		let total = T::LocalCurrency::free_balance(&fund_account);
		

		for (account_id, contribution) in Contributions::<T>::iter() {
			let contributed_balance = contribution.clone().get_total_user_balance();
			let share = Percent::from_rational(contributed_balance,total);
			contribution_shares.push(ContributionShare {
				account_id,
				share,
			});
		}
		contribution_shares
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
		let fund_account = Self::fund_account_id();
		let fund = T::LocalCurrency::free_balance(&fund_account);

		ensure!(fund>amount, Error::<T>::NotEnoughFundForHouse);

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
				
		// Get the block number for timestamp
		let block_number = <frame_system::Pallet<T>>::block_number();

		let reservation = HousingFundOperation {
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

		T::LocalCurrency::unreserve(&Self::fund_account_id(), reservation.amount);		
		Reservations::<T>::remove((nft_collection_id, nft_item_id));

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
		for (account_id, balance) in reservation.clone().contributions.into_iter() {
			Contributions::<T>::mutate(account_id.clone(), |val| {
				let mut unwrap_val = val.clone().unwrap();
				unwrap_val.use_reserved_amount(balance);
				let contribution = unwrap_val.clone();
				*val = Some(contribution);
			});
		}


		// Delete from reservation
		Reservations::<T>::remove((nft_collection_id, nft_item_id));
		// Add to purchased operations
		Purchases::<T>::insert((nft_collection_id, nft_item_id), reservation.clone());


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

	