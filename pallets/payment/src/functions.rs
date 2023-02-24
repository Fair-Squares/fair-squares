pub use super::*;

impl<T: Config> Pallet<T> {
	pub fn balance_to_u128_option(input: BalanceOf<T>) -> Option<u128> {
		input.try_into().ok()
	}
	pub fn u128_to_balance_option(input: u128) -> Option<BalanceOf<T>> {
		input.try_into().ok()
	}
}

impl<T: Config> PaymentHandler<T> for Pallet<T> {
	/// The function will create a new payment. The fee and incentive
	/// amounts will be calculated and the `PaymentDetail` will be added to
	/// storage.
	#[require_transactional]
	fn create_payment(
		from: &T::AccountId,
		recipient: &T::AccountId,
		amount: BalanceOf<T>,
		payment_state: PaymentState<T>,
		incentive_percentage: Percent,
		remark: Option<&[u8]>,
	) -> Result<PaymentDetail<T>, sp_runtime::DispatchError> {
		Payment::<T>::try_mutate(
			from,
			recipient,
			|maybe_payment| -> Result<PaymentDetail<T>, sp_runtime::DispatchError> {
				// only payment requests can be overwritten
				if let Some(payment) = maybe_payment {
					ensure!(
						payment.state == PaymentState::PaymentRequested,
						Error::<T>::PaymentAlreadyInProcess
					);
				}

				// Calculate incentive amount - this is to insentivise the user to release
				// the funds once a transaction has been completed
				let incentive_amount = incentive_percentage.mul_floor(amount);

				let mut new_payment = PaymentDetail {
					amount,
					incentive_amount,
					state: payment_state,
					resolver_account: T::DisputeResolver::get_resolver_account(),
					fee_detail: None,
				};

				// Calculate fee amount - this will be implemented based on the custom
				// implementation of the fee provider
				let (fee_recipient, fee_percent) =
					T::FeeHandler::apply_fees(from, recipient, &new_payment, remark);
				let fee_amount = fee_percent.mul_floor(amount);
				new_payment.fee_detail = Some((fee_recipient, fee_amount));

				*maybe_payment = Some(new_payment.clone());

				Ok(new_payment)
			},
		)
	}

	/// The function will reserve the fees+transfer amount from the `from`
	/// account. After reserving the payment.amount will be transferred to
	/// the recipient but will stay in Reserve state.
	#[require_transactional]
	fn reserve_payment_amount(
		from: &T::AccountId,
		to: &T::AccountId,
		payment: PaymentDetail<T>,
	) -> DispatchResult {
		let fee_amount = payment.fee_detail.map(|(_, f)| f).unwrap_or_else(|| 0u32.into());

		let total_fee_amount = payment.incentive_amount.saturating_add(fee_amount);
		let total_amount = total_fee_amount.saturating_add(payment.amount);

		// reserve the total amount from payment creator
		T::Currency::reserve(from, total_amount)?;
		// transfer payment amount to recipient -- keeping reserve status
		T::Currency::repatriate_reserved(from, to, payment.amount, BalanceStatus::Reserved)?;
		Ok(())
	}

	/// This function allows the caller to settle the payment by specifying
	/// a recipient_share this will unreserve the fee+incentive to sender
	/// and unreserve transferred amount to recipient if the settlement is a
	/// release (ie recipient_share=100), the fee is transferred to
	/// fee_recipient For cancelling a payment, recipient_share = 0
	/// For releasing a payment, recipient_share = 100
	/// In other cases, the custom recipient_share can be specified
	fn settle_payment(
		from: &T::AccountId,
		to: &T::AccountId,
		recipient_share: Percent,
	) -> DispatchResult {
		Payment::<T>::try_mutate(from, to, |maybe_payment| -> DispatchResult {
			let payment = maybe_payment.take().ok_or(Error::<T>::InvalidPayment)?;

			// unreserve the incentive amount and fees from the owner account
			match payment.fee_detail {
				Some((fee_recipient, fee_amount)) => {
					T::Currency::unreserve(from, payment.incentive_amount + fee_amount);
					// transfer fee to marketplace if operation is not cancel
					if recipient_share != Percent::zero() {
						T::Currency::transfer(
							from,           // fee is paid by payment creator
							&fee_recipient, // account of fee recipient
							fee_amount,     // amount of fee
							AllowDeath,
						)?;
					}
				},
				None => {
					T::Currency::unreserve(from, payment.incentive_amount);
				},
			};

			// Unreserve the transfer amount
			T::Currency::unreserve(to, payment.amount);

			let amount_to_recipient = recipient_share.mul_floor(payment.amount);
			let amount_to_sender = payment.amount.saturating_sub(amount_to_recipient);
			// send share to recipient
			T::Currency::transfer(to, from, amount_to_sender, AllowDeath)?;

			Ok(())
		})?;
		Ok(())
	}

	fn get_payment_details(from: &T::AccountId, to: &T::AccountId) -> Option<PaymentDetail<T>> {
		Payment::<T>::get(from, to)
	}
}

impl<T: Config> Pallet<T> {
	pub fn check_task(now: T::BlockNumber, remaining_weight: Weight) -> Weight {
		const MAX_TASKS_TO_PROCESS: usize = 5;
		// reduce the weight used to read the task list
		remaining_weight.saturating_sub(T::WeightInfo::remove_task());
		let cancel_weight = T::WeightInfo::cancel();

		// calculate count of tasks that can be processed with remaining weight
		let possible_task_count: usize = remaining_weight
			.ref_time()
			.saturating_div(cancel_weight.ref_time())
			.try_into()
			.unwrap_or(MAX_TASKS_TO_PROCESS);

		ScheduledTasks::<T>::mutate(|tasks| {
			let mut task_list: Vec<_> = tasks
				.clone()
				.into_iter()
				.take(possible_task_count)
				// leave out tasks in the future
				.filter(|(_, ScheduledTask { when, task })| {
					when <= &now && matches!(task, Task::Cancel)
				})
				.collect();

			// order by oldest task to process
			task_list.sort_by(|(_, t), (_, x)| x.when.cmp(&t.when));

			while !task_list.is_empty() && remaining_weight >= cancel_weight {
				if let Some((account_pair, _)) = task_list.pop() {
					remaining_weight.saturating_sub(cancel_weight);
					// remove the task form the tasks storage
					tasks.remove(&account_pair);

					// process the cancel payment
					if <Self as PaymentHandler<T>>::settle_payment(
						&account_pair.0,
						&account_pair.1,
						Percent::from_percent(0),
					)
					.is_err()
					{
						// log the payment refund failure
						log::warn!(
							target: "runtime::payments",
							"Warning: Unable to process payment refund!"
						);
					} else {
						// emit the cancel event if the refund was successful
						Self::deposit_event(Event::PaymentCancelled {
							from: account_pair.0,
							to: account_pair.1,
						});
					}
				}
			}
		});
		remaining_weight
	}
}
