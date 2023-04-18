#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod functions;
mod types;
pub use crate::types::*;
pub use functions::*;

pub mod weights;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// Dispute resolution account
		type DisputeResolver: DisputeResolver<Self::AccountId>;
		/// Fee handler trait
		type FeeHandler: FeeHandler<Self>;
		/// Incentive percentage - amount witheld from sender
		#[pallet::constant]
		type IncentivePercentage: Get<Percent>;
		/// Maximum permitted size of `Remark`
		#[pallet::constant]
		type MaxRemarkLength: Get<u32>;
		/// Buffer period - number of blocks to wait before user can claim
		/// canceled payment
		#[pallet::constant]
		type CancelBufferBlockLength: Get<Self::BlockNumber>;
		/// Buffer period - number of blocks to wait before user can claim
		/// canceled payment
		#[pallet::constant]
		type MaxScheduledTaskListLength: Get<u32>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn payment)]
	/// Payments created by a user, this method of storageDoubleMap is chosen
	/// since there is no usecase for listing payments by provider/currency. The
	/// payment will only be referenced by the creator in any transaction of
	/// interest. The storage map keys are the creator and the recipient, this
	/// also ensures that for any (sender,recipient) combo, only a single
	/// payment is active. The history of payment is not stored.
	pub type Payment<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId, // payment creator
		Blake2_128Concat,
		T::AccountId, // payment recipient
		PaymentDetail<T>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn tasks)]
	/// Store the list of tasks to be executed in the on_idle function
	pub(super) type ScheduledTasks<T: Config> = StorageValue<_, ScheduledTaskList<T>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		/// A new payment has been created
		PaymentCreated {
			from: T::AccountId,
			amount: BalanceOf<T>,
			remark: Option<BoundedDataOf<T>>,
		},
		/// Payment amount released to the recipient
		PaymentReleased { from: T::AccountId, to: T::AccountId },
		/// Payment has been cancelled by the creator
		PaymentCancelled { from: T::AccountId, to: T::AccountId },
		/// A payment that NeedsReview has been resolved by Judge
		PaymentResolved { from: T::AccountId, to: T::AccountId, recipient_share: Percent },
		/// the payment creator has created a refund request
		PaymentCreatorRequestedRefund {
			from: T::AccountId,
			to: T::AccountId,
			expiry: T::BlockNumber,
		},
		/// the refund request from creator was disputed by recipient
		PaymentRefundDisputed { from: T::AccountId, to: T::AccountId },
		/// Payment request was created by recipient
		PaymentRequestCreated { from: T::AccountId, to: T::AccountId },
		/// Payment request was completed by sender
		PaymentRequestCompleted { from: T::AccountId, to: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// The selected payment does not exist
		InvalidPayment,
		/// The selected payment cannot be released
		PaymentAlreadyReleased,
		/// The selected payment already exists and is in process
		PaymentAlreadyInProcess,
		/// Action permitted only for whitelisted users
		InvalidAction,
		/// Payment is in review state and cannot be modified
		PaymentNeedsReview,
		/// Unexpeted math error
		MathError,
		/// Payment request has not been created
		RefundNotRequested,
		/// Dispute period has not passed
		DisputePeriodNotPassed,
		/// The automatic cancelation queue cannot accept
		RefundQueueFull,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Hook that execute when there is leftover space in a block
		/// This function will look for any pending scheduled tasks that can
		/// be executed and will process them.
		fn on_idle(now: T::BlockNumber, remaining_weight: Weight) -> Weight {
			Self::check_task(now, remaining_weight)
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This allows any user to create a new payment, that releases only to
		/// specified recipient The only action is to store the details of this
		/// payment in storage and reserve the specified amount. User also has
		/// the option to add a remark, this remark can then be used to run
		/// custom logic and trigger alternate payment flows. the specified
		/// amount.
		#[pallet::weight(T::WeightInfo::pay(T::MaxRemarkLength::get()))]
		pub fn pay(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			#[pallet::compact] amount: BalanceOf<T>,
			remark: Option<BoundedDataOf<T>>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// create PaymentDetail and add to storage
			let payment_detail = <Self as PaymentHandler<T>>::create_payment(
				&who,
				&recipient,
				amount,
				PaymentState::Created,
				T::IncentivePercentage::get(),
				remark.as_ref().map(|x| x.as_slice()),
			)?;
			// reserve funds for payment
			<Self as PaymentHandler<T>>::reserve_payment_amount(&who, &recipient, payment_detail)?;
			// emit paymentcreated event
			Self::deposit_event(Event::PaymentCreated { from: who, amount, remark });
			Ok(().into())
		}

		/// Release any created payment, this will transfer the reserved amount
		/// from the creator of the payment to the assigned recipient
		#[pallet::weight(T::WeightInfo::release())]
		pub fn release(origin: OriginFor<T>, to: T::AccountId) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;

			// ensure the payment is in Created state
			let payment = Payment::<T>::get(&from, &to).ok_or(Error::<T>::InvalidPayment)?;
			ensure!(payment.state == PaymentState::Created, Error::<T>::InvalidAction);

			// release is a settle_payment with 100% recipient_share
			<Self as PaymentHandler<T>>::settle_payment(&from, &to, Percent::from_percent(100))?;

			Self::deposit_event(Event::PaymentReleased { from, to });
			Ok(().into())
		}

		/// Cancel a payment in created state, this will release the reserved
		/// back to creator of the payment. This extrinsic can only be called by
		/// the recipient of the payment
		#[pallet::weight(T::WeightInfo::cancel())]
		pub fn cancel(origin: OriginFor<T>, creator: T::AccountId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			if let Some(payment) = Payment::<T>::get(&creator, &who) {
				match payment.state {
					// call settle payment with recipient_share=0, this refunds the sender
					PaymentState::Created => {
						<Self as PaymentHandler<T>>::settle_payment(
							&creator,
							&who,
							Percent::from_percent(0),
						)?;
						Self::deposit_event(Event::PaymentCancelled { from: creator, to: who });
					},
					// if the payment is in state PaymentRequested, remove from storage
					PaymentState::PaymentRequested => Payment::<T>::remove(&creator, &who),
					_ => fail!(Error::<T>::InvalidAction),
				}
			}
			Ok(().into())
		}

		/// This extrinsic is used to resolve disputes between the creator and
		/// recipient of the payment.
		/// This extrinsic allows the assigned judge to
		/// cancel/release/partial_release the payment.
		#[pallet::weight(T::WeightInfo::resolve_payment())]
		pub fn resolve_payment(
			origin: OriginFor<T>,
			from: T::AccountId,
			recipient: T::AccountId,
			recipient_share: Percent,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let account_pair = (from, recipient);
			// ensure the caller is the assigned resolver
			if let Some(payment) = Payment::<T>::get(&account_pair.0, &account_pair.1) {
				ensure!(who == payment.resolver_account, Error::<T>::InvalidAction);
				ensure!(payment.state != PaymentState::PaymentRequested, Error::<T>::InvalidAction);
				if matches!(payment.state, PaymentState::RefundRequested { .. }) {
					ScheduledTasks::<T>::mutate(|tasks| {
						tasks.remove(&account_pair);
					})
				}
			}
			// try to update the payment to new state
			<Self as PaymentHandler<T>>::settle_payment(
				&account_pair.0,
				&account_pair.1,
				recipient_share,
			)?;
			Self::deposit_event(Event::PaymentResolved {
				from: account_pair.0,
				to: account_pair.1,
				recipient_share,
			});
			Ok(().into())
		}

		/// Allow the creator of a payment to initiate a refund that will return
		/// the funds after a configured amount of time that the reveiver has to
		/// react and opose the request
		#[pallet::weight(T::WeightInfo::request_refund())]
		pub fn request_refund(
			origin: OriginFor<T>,
			recipient: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Payment::<T>::try_mutate(
				who.clone(),
				recipient.clone(),
				|maybe_payment| -> DispatchResult {
					// ensure the payment exists
					let payment = maybe_payment.as_mut().ok_or(Error::<T>::InvalidPayment)?;
					// refunds only possible for payments in created state
					ensure!(payment.state == PaymentState::Created, Error::<T>::InvalidAction);

					// set the payment to requested refund
					let current_block = frame_system::Pallet::<T>::block_number();
					let cancel_block = current_block
						.checked_add(&T::CancelBufferBlockLength::get())
						.ok_or(Error::<T>::MathError)?;

					ScheduledTasks::<T>::try_mutate(|task_list| -> DispatchResult {
						task_list
							.try_insert(
								(who.clone(), recipient.clone()),
								ScheduledTask { task: Task::Cancel, when: cancel_block },
							)
							.map_err(|_| Error::<T>::RefundQueueFull)?;
						Ok(())
					})?;

					payment.state = PaymentState::RefundRequested { cancel_block };

					Self::deposit_event(Event::PaymentCreatorRequestedRefund {
						from: who,
						to: recipient,
						expiry: cancel_block,
					});

					Ok(())
				},
			)?;

			Ok(().into())
		}

		/// Allow payment recipient to dispute the refund request from the
		/// payment creator This does not cancel the request, instead sends the
		/// payment to a NeedsReview state The assigned resolver account can
		/// then change the state of the payment after review.
		#[pallet::weight(T::WeightInfo::dispute_refund())]
		pub fn dispute_refund(
			origin: OriginFor<T>,
			creator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			use PaymentState::*;
			let who = ensure_signed(origin)?;

			Payment::<T>::try_mutate(
				creator.clone(),
				who.clone(), // should be called by the payment recipient
				|maybe_payment| -> DispatchResult {
					// ensure the payment exists
					let payment = maybe_payment.as_mut().ok_or(Error::<T>::InvalidPayment)?;
					// ensure the payment is in Requested Refund state
					match payment.state {
						RefundRequested { cancel_block } => {
							ensure!(
								cancel_block > frame_system::Pallet::<T>::block_number(),
								Error::<T>::InvalidAction
							);

							payment.state = PaymentState::NeedsReview;

							// remove the payment from scheduled tasks
							ScheduledTasks::<T>::try_mutate(|task_list| -> DispatchResult {
								task_list
									.remove(&(creator.clone(), who.clone()))
									.ok_or(Error::<T>::InvalidAction)?;
								Ok(())
							})?;

							Self::deposit_event(Event::PaymentRefundDisputed {
								from: creator,
								to: who,
							});
						},
						_ => fail!(Error::<T>::InvalidAction),
					}

					Ok(())
				},
			)?;

			Ok(().into())
		}

		// Creates a new payment with the given details. This can be called by the
		// recipient of the payment to create a payment and then completed by the sender
		// using the `accept_and_pay` extrinsic.  The payment will be in
		// PaymentRequested State and can only be modified by the `accept_and_pay`
		// extrinsic.
		#[pallet::weight(T::WeightInfo::request_payment())]
		pub fn request_payment(
			origin: OriginFor<T>,
			from: T::AccountId,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let to = ensure_signed(origin)?;

			// create PaymentDetail and add to storage
			<Self as PaymentHandler<T>>::create_payment(
				&from,
				&to,
				amount,
				PaymentState::PaymentRequested,
				Percent::from_percent(0),
				None,
			)?;

			Self::deposit_event(Event::PaymentRequestCreated { from, to });

			Ok(().into())
		}

		// This extrinsic allows the sender to fulfill a payment request created by a
		// recipient. The amount will be transferred to the recipient and payment
		// removed from storage
		#[pallet::weight(T::WeightInfo::accept_and_pay())]
		pub fn accept_and_pay(
			origin: OriginFor<T>,
			to: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;

			let payment = Payment::<T>::get(&from, &to).ok_or(Error::<T>::InvalidPayment)?;

			ensure!(payment.state == PaymentState::PaymentRequested, Error::<T>::InvalidAction);

			// reserve all the fees from the sender
			<Self as PaymentHandler<T>>::reserve_payment_amount(&from, &to, payment.clone())?;

			// release the payment and delete the payment from storage
			<Self as PaymentHandler<T>>::settle_payment(&from, &to, Percent::from_percent(100))?;
			T::Currency::reserve(&to, payment.amount).ok();

			Self::deposit_event(Event::PaymentRequestCompleted { from, to });

			Ok(().into())
		}
	}
}
