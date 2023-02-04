use crate::{
	mock::*, 
	types::{PaymentDetail, PaymentState},
	Payment as PaymentStore, PaymentHandler, ScheduledTask, ScheduledTasks, Task,
};
use frame_support::{assert_noop, assert_ok, storage::with_transaction};
use sp_runtime::{Percent, TransactionOutcome};
type Error = crate::Error<Test>;

fn last_event() -> Event {
	System::events().pop().expect("Event expected").event
}

#[test]
fn test_pay_works() {
	new_test_ext().execute_with(|| {
		let creator_initial_balance = 100_000_000_000;
		let payment_amount = 2000 as u64;
		let expected_incentive_amount = payment_amount / INCENTIVE_PERCENTAGE as u64;

		// the payment amount should not be reserved
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), 1);

		// should be able to create a payment with available balance
		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			payment_amount,
			None
		));
		assert_eq!(
			last_event(),
			crate::Event::<Test>::PaymentCreated {
				from: PAYMENT_CREATOR,
				amount: payment_amount,
				remark: None
			}
			.into()
		);

		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				amount: payment_amount,
				incentive_amount: expected_incentive_amount,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
			})
		);
		// the payment amount should be reserved correctly
		// the amount + incentive should be removed from the sender account
		assert_eq!(
			Balances::free_balance( &PAYMENT_CREATOR),
			creator_initial_balance - payment_amount - expected_incentive_amount
		);
		// the incentive amount should be reserved in the sender account
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR).saturating_add(Balances::reserved_balance(&PAYMENT_CREATOR)),
			creator_initial_balance - payment_amount
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), 1);
		// the transferred amount should be reserved in the recipent account
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT).saturating_add(Balances::reserved_balance(&PAYMENT_RECIPENT)), payment_amount.saturating_add(Balances::free_balance(&PAYMENT_RECIPENT)));

		// the payment should not be overwritten
		assert_noop!(
			PaymentModule::pay(
				Origin::signed(PAYMENT_CREATOR),
				PAYMENT_RECIPENT,
				payment_amount,
				None
			),
			crate::Error::<Test>::PaymentAlreadyInProcess
		);

		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				amount: payment_amount,
				incentive_amount: 200,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
			})
		);
	});
}

#[test]
fn test_cancel_works() {
	new_test_ext().execute_with(|| {
		let creator_initial_balance = 100_000_000_000;
		let payment_amount = 40;
		let expected_incentive_amount = payment_amount / INCENTIVE_PERCENTAGE as u64;

		// should be able to create a payment with available balance
		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			payment_amount,
			None
		));

		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				amount: payment_amount,
				incentive_amount: expected_incentive_amount,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
			})
		);
		// the payment amount should be reserved
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance - payment_amount - expected_incentive_amount
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), 1);

		// cancel should succeed when caller is the recipent
		assert_ok!(PaymentModule::cancel(Origin::signed(PAYMENT_RECIPENT), PAYMENT_CREATOR));
		assert_eq!(
			last_event(),
			crate::Event::<Test>::PaymentCancelled {
				from: PAYMENT_CREATOR,
				to: PAYMENT_RECIPENT
			}
			.into()
		);
		// the payment amount should be released back to creator
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), 1);

		// should be released from storage
		assert_eq!(PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT), None);
	});
}

#[test]
fn test_release_works() {
	new_test_ext().execute_with(|| {
		let creator_initial_balance = 100_000_000_000 as u64;
		let payment_amount = 40 as u64;
		let expected_incentive_amount = payment_amount / INCENTIVE_PERCENTAGE as u64;
		let initial_recipient_balance = Balances::free_balance(&PAYMENT_RECIPENT);

		// should be able to create a payment with available balance
		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			payment_amount,
			None
		));
		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				amount: payment_amount,
				incentive_amount: expected_incentive_amount,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
			})
		);
		// the payment amount should be reserved
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance - payment_amount - expected_incentive_amount
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), 1);

		// should succeed for valid payment
		assert_ok!(PaymentModule::release(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT
		));
		assert_eq!(
			last_event(),
			crate::Event::<Test>::PaymentReleased {
				from: PAYMENT_CREATOR,
				to: PAYMENT_RECIPENT
			}
			.into()
		);
		// the payment amount should be transferred
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance - payment_amount
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), payment_amount+initial_recipient_balance);

		// should be deleted from storage
		assert_eq!(PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT), None);

		// should be able to create another payment since previous is released
		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			payment_amount,
			None
		));
		// the payment amount should be reserved
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance - (payment_amount * 2) - expected_incentive_amount
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), payment_amount+initial_recipient_balance);
	});
}

#[test]
fn test_resolve_payment_works() {
	new_test_ext().execute_with(|| {
		let creator_initial_balance = 100_000_000_000 as u64;
		let payment_amount = 40 as u64;
		let initial_recipient_balance = Balances::free_balance(&PAYMENT_RECIPENT);

		// should be able to create a payment with available balance
		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			payment_amount,
			None
		));

		// should fail for non whitelisted caller
		assert_noop!(
			PaymentModule::resolve_payment(
				Origin::signed(PAYMENT_CREATOR),
				PAYMENT_CREATOR,
				PAYMENT_RECIPENT,
				Percent::from_percent(100)
			),
			Error::InvalidAction
		);

		// should be able to release a payment
		assert_ok!(PaymentModule::resolve_payment(
			Origin::signed(RESOLVER_ACCOUNT),
			PAYMENT_CREATOR,
			PAYMENT_RECIPENT,
			Percent::from_percent(100)
		));
		assert_eq!(
			last_event(),
			crate::Event::<Test>::PaymentResolved {
				from: PAYMENT_CREATOR,
				to: PAYMENT_RECIPENT,
				recipient_share: Percent::from_percent(100)
			}
			.into()
		);

		// the payment amount should be transferred
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance - payment_amount
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), payment_amount+initial_recipient_balance);

		// should be removed from storage
		assert_eq!(PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT), None);

		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			payment_amount,
			None
		));

		// should be able to cancel a payment
		assert_ok!(PaymentModule::resolve_payment(
			Origin::signed(RESOLVER_ACCOUNT),
			PAYMENT_CREATOR,
			PAYMENT_RECIPENT,
			Percent::from_percent(0)
		));
		assert_eq!(
			last_event(),
			crate::Event::<Test>::PaymentResolved {
				from: PAYMENT_CREATOR,
				to: PAYMENT_RECIPENT,
				recipient_share: Percent::from_percent(0)
			}
			.into()
		);

		// the payment amount should be transferred
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance - payment_amount
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), payment_amount+initial_recipient_balance);

		// should be released from storage
		assert_eq!(PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT), None);
	});
}

#[test]
fn test_charging_fee_payment_works() {
	new_test_ext().execute_with(|| {
		let creator_initial_balance = 100_000_000_000;
		let payment_amount = 40;
		let expected_incentive_amount = payment_amount / INCENTIVE_PERCENTAGE as u64;
		let expected_fee_amount = payment_amount / MARKETPLACE_FEE_PERCENTAGE as u64;
		let initial_recipient_fee_charged = Balances::free_balance(&PAYMENT_RECIPENT_FEE_CHARGED);
		let initial_fee_recipient_balance = Balances::free_balance(&FEE_RECIPIENT_ACCOUNT);

		// should be able to create a payment with available balance
		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT_FEE_CHARGED,
			payment_amount,
			None
		));
		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT_FEE_CHARGED),
			Some(PaymentDetail {
				amount: payment_amount,
				incentive_amount: expected_incentive_amount,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, expected_fee_amount)),
			})
		);
		// the payment amount should be reserved
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance - payment_amount - expected_fee_amount - expected_incentive_amount
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT_FEE_CHARGED), initial_recipient_fee_charged);

		// should succeed for valid payment
		assert_ok!(PaymentModule::release(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT_FEE_CHARGED
		));
		// the payment amount should be transferred
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance - payment_amount - expected_fee_amount
		);
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR)+Balances::reserved_balance(&PAYMENT_CREATOR),
			creator_initial_balance - payment_amount - expected_fee_amount
		);
		assert_eq!(
			Balances::free_balance(&PAYMENT_RECIPENT_FEE_CHARGED),
			payment_amount+initial_recipient_fee_charged
		);
		assert_eq!(
			Balances::free_balance(&FEE_RECIPIENT_ACCOUNT),
			expected_fee_amount+initial_fee_recipient_balance
		);
	});
}

#[test]
fn test_charging_fee_payment_works_when_canceled() {
	new_test_ext().execute_with(|| {
		let creator_initial_balance = 100_000_000_000;
		let payment_amount = 40;
		let expected_incentive_amount = payment_amount / INCENTIVE_PERCENTAGE as u64;
		let expected_fee_amount = payment_amount / MARKETPLACE_FEE_PERCENTAGE as u64;

		// should be able to create a payment with available balance
		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT_FEE_CHARGED,
			payment_amount,
			None
		));
		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT_FEE_CHARGED),
			Some(PaymentDetail {
				amount: payment_amount,
				incentive_amount: expected_incentive_amount,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, expected_fee_amount)),
			})
		);
		// the payment amount should be reserved
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance - payment_amount - expected_fee_amount - expected_incentive_amount
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT_FEE_CHARGED), 1);

		// should succeed for valid payment
		assert_ok!(PaymentModule::cancel(
			Origin::signed(PAYMENT_RECIPENT_FEE_CHARGED),
			PAYMENT_CREATOR
		));
		// the payment amount should be transferred
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance
		);
		assert_eq!(Balances::free_balance(&PAYMENT_CREATOR).saturating_add(Balances::reserved_balance(&PAYMENT_CREATOR)), 100_000_000_000);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT_FEE_CHARGED), 1);
		assert_eq!(Balances::free_balance(&FEE_RECIPIENT_ACCOUNT), 1);
	});
}


#[test]
fn test_pay_with_remark_works() {
	new_test_ext().execute_with(|| {
		let creator_initial_balance = 100_000_000_000;
		let payment_amount = 40;
		let expected_incentive_amount = payment_amount / INCENTIVE_PERCENTAGE as u64;

		// should be able to create a payment with available balance
		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			payment_amount,
			Some(vec![1u8; 10].try_into().unwrap())
		));
		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				amount: payment_amount,
				incentive_amount: expected_incentive_amount,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
			})
		);
		// the payment amount should be reserved correctly
		// the amount + incentive should be removed from the sender account
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance - payment_amount - expected_incentive_amount
		);
		// the incentive amount should be reserved in the sender account
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR).saturating_add(Balances::reserved_balance(&PAYMENT_CREATOR)),
			creator_initial_balance - payment_amount
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), 1);
		// the transferred amount should be reserved in the recipent account
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT).saturating_add(Balances::reserved_balance(&PAYMENT_RECIPENT)), Balances::free_balance(&PAYMENT_RECIPENT).saturating_add(payment_amount));

		// the payment should not be overwritten
		assert_noop!(
			PaymentModule::pay(
				Origin::signed(PAYMENT_CREATOR),
				PAYMENT_RECIPENT,
				payment_amount,
				None
			),
			crate::Error::<Test>::PaymentAlreadyInProcess
		);

		assert_eq!(
			last_event(),
			crate::Event::<Test>::PaymentCreated {
				from: PAYMENT_CREATOR,
				amount: payment_amount,
				remark: Some(vec![1u8; 10].try_into().unwrap())
			}
			.into()
		);
	});
}


#[test]
fn test_do_not_overwrite_logic_works() {
	new_test_ext().execute_with(|| {
		let payment_amount = 40;
		let expected_incentive_amount = payment_amount / INCENTIVE_PERCENTAGE as u64;

		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			payment_amount,
			None
		));

		assert_noop!(
			PaymentModule::pay(
				Origin::signed(PAYMENT_CREATOR),
				PAYMENT_RECIPENT,
				payment_amount,
				None
			),
			crate::Error::<Test>::PaymentAlreadyInProcess
		);

		// set payment state to NeedsReview
		PaymentStore::<Test>::insert(
			PAYMENT_CREATOR,
			PAYMENT_RECIPENT,
			PaymentDetail {
				amount: payment_amount,
				incentive_amount: expected_incentive_amount,
				state: PaymentState::NeedsReview,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
			},
		);

		// the payment should not be overwritten
		assert_noop!(
			PaymentModule::pay(
				Origin::signed(PAYMENT_CREATOR),
				PAYMENT_RECIPENT,
				payment_amount,
				None
			),
			crate::Error::<Test>::PaymentAlreadyInProcess
		);
	});
}

#[test]
fn test_request_refund() {
	new_test_ext().execute_with(|| {
		let payment_amount = 20;
		let expected_incentive_amount = payment_amount / INCENTIVE_PERCENTAGE as u64;
		let expected_cancel_block = CANCEL_BLOCK_BUFFER + 1;

		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			payment_amount,
			None
		));

		assert_ok!(PaymentModule::request_refund(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT
		));

		// do not overwrite payment
		assert_noop!(
			PaymentModule::pay(
				Origin::signed(PAYMENT_CREATOR),
				PAYMENT_RECIPENT,
				payment_amount,
				None
			),
			crate::Error::<Test>::PaymentAlreadyInProcess
		);

		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				amount: payment_amount,
				incentive_amount: expected_incentive_amount,
				state: PaymentState::RefundRequested {
					cancel_block: expected_cancel_block
				},
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
			})
		);

		assert_eq!(
			last_event(),
			crate::Event::<Test>::PaymentCreatorRequestedRefund {
				from: PAYMENT_CREATOR,
				to: PAYMENT_RECIPENT,
				expiry: expected_cancel_block
			}
			.into()
		);
	});
}

#[test]
fn test_dispute_refund() {
	new_test_ext().execute_with(|| {
		let payment_amount = 20;
		let expected_incentive_amount = payment_amount / INCENTIVE_PERCENTAGE as u64;
		let expected_cancel_block = CANCEL_BLOCK_BUFFER + 1;

		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			payment_amount,
			None
		));

		// cannot dispute if refund is not requested
		assert_noop!(
			PaymentModule::dispute_refund(Origin::signed(PAYMENT_RECIPENT), PAYMENT_CREATOR),
			Error::InvalidAction
		);
		// creator requests a refund
		assert_ok!(PaymentModule::request_refund(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT
		));
		// ensure the request is added to the refund queue
		let scheduled_tasks_list = ScheduledTasks::<Test>::get();
		assert_eq!(
			scheduled_tasks_list.get(&(PAYMENT_CREATOR, PAYMENT_RECIPENT)).unwrap(),
			&ScheduledTask {
				task: Task::Cancel,
				when: expected_cancel_block
			}
		);

		// recipient disputes the refund request
		assert_ok!(PaymentModule::dispute_refund(
			Origin::signed(PAYMENT_RECIPENT),
			PAYMENT_CREATOR
		));

		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				amount: payment_amount,
				incentive_amount: expected_incentive_amount,
				state: PaymentState::NeedsReview,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
			})
		);

		assert_eq!(
			last_event(),
			crate::Event::<Test>::PaymentRefundDisputed {
				from: PAYMENT_CREATOR,
				to: PAYMENT_RECIPENT,
			}
			.into()
		);

		// ensure the request is removed from the refund queue
		let scheduled_tasks_list = ScheduledTasks::<Test>::get();
		assert_eq!(scheduled_tasks_list.get(&(PAYMENT_CREATOR, PAYMENT_RECIPENT)), None);
	});
}


#[test]
fn test_request_payment() {
	new_test_ext().execute_with(|| {
		let payment_amount = 20;
		let expected_incentive_amount = 0;

		assert_ok!(PaymentModule::request_payment(
			Origin::signed(PAYMENT_RECIPENT),
			PAYMENT_CREATOR,
			payment_amount,
		));

		assert_noop!(
			PaymentModule::request_refund(Origin::signed(PAYMENT_CREATOR), PAYMENT_RECIPENT),
			crate::Error::<Test>::InvalidAction
		);

		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				amount: payment_amount,
				incentive_amount: expected_incentive_amount,
				state: PaymentState::PaymentRequested,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
			})
		);

		assert_eq!(
			last_event(),
			crate::Event::<Test>::PaymentRequestCreated {
				from: PAYMENT_CREATOR,
				to: PAYMENT_RECIPENT,
			}
			.into()
		);
	});
}

#[test]
fn test_requested_payment_cannot_be_released() {
	new_test_ext().execute_with(|| {
		let payment_amount = 20;

		assert_ok!(PaymentModule::request_payment(
			Origin::signed(PAYMENT_RECIPENT),
			PAYMENT_CREATOR,
			payment_amount,
		));

		// requested payment cannot be released
		assert_noop!(
			PaymentModule::release(Origin::signed(PAYMENT_CREATOR), PAYMENT_RECIPENT),
			Error::InvalidAction
		);
	});
}

#[test]
fn test_requested_payment_can_be_cancelled_by_requestor() {
	new_test_ext().execute_with(|| {
		let payment_amount = 20;

		assert_ok!(PaymentModule::request_payment(
			Origin::signed(PAYMENT_RECIPENT),
			PAYMENT_CREATOR,
			payment_amount,
		));

		assert_ok!(PaymentModule::cancel(Origin::signed(PAYMENT_RECIPENT), PAYMENT_CREATOR));

		// the request should be removed from storage
		assert_eq!(PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT), None);
	});
}

#[test]
fn test_accept_and_pay() {
	new_test_ext().execute_with(|| {
		let creator_initial_balance = 100_000_000_000;
		let payment_amount = 20;
		let expected_incentive_amount = 0;
		let recipient_initial_balance = 1;

		assert_ok!(PaymentModule::request_payment(
			Origin::signed(PAYMENT_RECIPENT),
			PAYMENT_CREATOR,
			payment_amount,
		));

		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				amount: payment_amount,
				incentive_amount: expected_incentive_amount,
				state: PaymentState::PaymentRequested,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
			})
		);

		assert_ok!(PaymentModule::accept_and_pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
		));

		// the payment amount should be transferred
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance - payment_amount
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), payment_amount.saturating_add(recipient_initial_balance));

		// should be deleted from storage
		assert_eq!(PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT), None);

		assert_eq!(
			last_event(),
			crate::Event::<Test>::PaymentRequestCompleted {
				from: PAYMENT_CREATOR,
				to: PAYMENT_RECIPENT,
			}
			.into()
		);
	});
}

#[test]
fn test_accept_and_pay_should_fail_for_non_payment_requested() {
	new_test_ext().execute_with(|| {
		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			20,
			None
		));

		assert_noop!(
			PaymentModule::accept_and_pay(Origin::signed(PAYMENT_CREATOR), PAYMENT_RECIPENT,),
			Error::InvalidAction
		);
	});
}

#[test]
fn test_accept_and_pay_should_charge_fee_correctly() {
	new_test_ext().execute_with(|| {
		let creator_initial_balance = 100_000_000_000;
		let recipient_initial_balance = 1;
		let payment_amount = 20;
		let expected_incentive_amount = 0;
		let expected_fee_amount = payment_amount / MARKETPLACE_FEE_PERCENTAGE as u64;

		assert_ok!(PaymentModule::request_payment(
			Origin::signed(PAYMENT_RECIPENT_FEE_CHARGED),
			PAYMENT_CREATOR,
			payment_amount,
		));

		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT_FEE_CHARGED),
			Some(PaymentDetail {
				amount: payment_amount,
				incentive_amount: expected_incentive_amount,
				state: PaymentState::PaymentRequested,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, expected_fee_amount)),
			})
		);

		assert_ok!(PaymentModule::accept_and_pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT_FEE_CHARGED,
		));

		// the payment amount should be transferred
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance - payment_amount - expected_fee_amount
		);
		assert_eq!(
			Balances::free_balance(&PAYMENT_RECIPENT_FEE_CHARGED),
			payment_amount.saturating_add(recipient_initial_balance)
		);
		assert_eq!(
			Balances::free_balance(&FEE_RECIPIENT_ACCOUNT),
			expected_fee_amount.saturating_add(recipient_initial_balance)
		);

		// should be deleted from storage
		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT_FEE_CHARGED),
			None
		);

		assert_eq!(
			last_event(),
			crate::Event::<Test>::PaymentRequestCompleted {
				from: PAYMENT_CREATOR,
				to: PAYMENT_RECIPENT_FEE_CHARGED,
			}
			.into()
		);
	});
}

#[test]
fn test_create_payment_works() {
	new_test_ext().execute_with(|| {
		let creator_initial_balance = 100_000_000_000;
		let payment_amount = 20;
		let expected_incentive_amount = payment_amount / INCENTIVE_PERCENTAGE as u64;
		let expected_fee_amount = 0;

		// the payment amount should not be reserved
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), 1);

		// should be able to create a payment with available balance within a
		// transaction
		assert_ok!(with_transaction(|| TransactionOutcome::Commit({
			<PaymentModule as PaymentHandler<Test>>::create_payment(
				&PAYMENT_CREATOR,
				&PAYMENT_RECIPENT,
				payment_amount,
				PaymentState::Created,
				Percent::from_percent(INCENTIVE_PERCENTAGE),
				Some(&[1u8; 10]),
			)
		})));

		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				amount: payment_amount,
				incentive_amount: expected_incentive_amount,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, expected_fee_amount)),
			})
		);

		// the payment should not be overwritten
		assert_noop!(
			with_transaction(|| TransactionOutcome::Commit({
				<PaymentModule as PaymentHandler<Test>>::create_payment(
					&PAYMENT_CREATOR,
					&PAYMENT_RECIPENT,
					payment_amount,
					PaymentState::Created,
					Percent::from_percent(INCENTIVE_PERCENTAGE),
					Some(&[1u8; 10]),
				)
			})),
			Error::PaymentAlreadyInProcess
		);

		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				amount: payment_amount,
				incentive_amount: expected_incentive_amount,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, expected_fee_amount)),
			})
		);
	});
}

#[test]
fn test_reserve_payment_amount_works() {
	new_test_ext().execute_with(|| {
		let creator_initial_balance = 100_000_000_000;
		let payment_amount = 20;
		let expected_incentive_amount = payment_amount / INCENTIVE_PERCENTAGE as u64;
		let expected_fee_amount = 0;

		// the payment amount should not be reserved
		assert_eq!(Balances::free_balance(&PAYMENT_CREATOR), 100_000_000_000);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), 1);

		// should be able to create a payment with available balance within a
		// transaction
		assert_ok!(with_transaction(|| TransactionOutcome::Commit({
			<PaymentModule as PaymentHandler<Test>>::create_payment(
				&PAYMENT_CREATOR,
				&PAYMENT_RECIPENT,
				payment_amount,
				PaymentState::Created,
				Percent::from_percent(INCENTIVE_PERCENTAGE),
				Some(&[1u8; 10]),
			)
		})));

		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				amount: payment_amount,
				incentive_amount: expected_incentive_amount,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, expected_fee_amount)),
			})
		);

		assert_ok!(with_transaction(|| TransactionOutcome::Commit({
			<PaymentModule as PaymentHandler<Test>>::reserve_payment_amount(
				&PAYMENT_CREATOR,
				&PAYMENT_RECIPENT,
				PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT).unwrap(),
			)
		})));
		// the payment amount should be reserved correctly
		// the amount + incentive should be removed from the sender account
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance - payment_amount - expected_incentive_amount
		);
		// the incentive amount should be reserved in the sender account
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR).saturating_add(Balances::reserved_balance(&PAYMENT_CREATOR)),
			creator_initial_balance - payment_amount
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), 1);
		// the transferred amount should be reserved in the recipent account
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT).saturating_add(Balances::reserved_balance(&PAYMENT_RECIPENT)), payment_amount.saturating_add(Balances::free_balance(&PAYMENT_RECIPENT)));

		// the payment should not be overwritten
		assert_noop!(
			with_transaction(|| TransactionOutcome::Commit({
				<PaymentModule as PaymentHandler<Test>>::create_payment(
					&PAYMENT_CREATOR,
					&PAYMENT_RECIPENT,
					payment_amount,
					PaymentState::Created,
					Percent::from_percent(INCENTIVE_PERCENTAGE),
					Some(&[1u8; 10]),
				)
			})),
			Error::PaymentAlreadyInProcess
		);

		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				amount: payment_amount,
				incentive_amount: expected_incentive_amount,
				state: PaymentState::Created,
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, expected_fee_amount)),
			})
		);
	});
}


#[test]
fn test_settle_payment_works_for_cancel() {
	new_test_ext().execute_with(|| {
		let creator_initial_balance = 100_000_000_000;
		let payment_amount = 20;
		let recipient_initial_balance = 1;

		// the payment amount should not be reserved
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), 1);

		// should be able to create a payment with available balance within a
		// transaction
		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			payment_amount,
			None
		));

		assert_ok!(with_transaction(|| TransactionOutcome::Commit({
			<PaymentModule as PaymentHandler<Test>>::settle_payment(
				&PAYMENT_CREATOR,
				&PAYMENT_RECIPENT,
				Percent::from_percent(0),
			)
		})));

		// the payment amount should be released back to creator
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), recipient_initial_balance);

		// should be released from storage
		assert_eq!(PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT), None);
	});
}

#[test]
fn test_settle_payment_works_for_release() {
	new_test_ext().execute_with(|| {
		let creator_initial_balance = 100_000_000_000;
		let payment_amount = 20;
		let recipient_initial_balance = 1;

		// the payment amount should not be reserved
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), 1);

		// should be able to create a payment with available balance within a
		// transaction
		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			payment_amount,
			None
		));

		assert_ok!(with_transaction(|| TransactionOutcome::Commit({
			<PaymentModule as PaymentHandler<Test>>::settle_payment(
				&PAYMENT_CREATOR,
				&PAYMENT_RECIPENT,
				Percent::from_percent(100),
			)
		})));

		// the payment amount should be transferred
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance - payment_amount
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), payment_amount.saturating_add(recipient_initial_balance));

		// should be deleted from storage
		assert_eq!(PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT), None);
	});
}

#[test]
fn test_settle_payment_works_for_70_30() {
	new_test_ext().execute_with(|| {
		let creator_initial_balance = 100_000_000_000;
		let payment_amount = 10;
		let expected_fee_amount = payment_amount / MARKETPLACE_FEE_PERCENTAGE as u64;
		let recipient_initial_balance = 1;

		// the payment amount should not be reserved
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT_FEE_CHARGED), 1);

		// should be able to create a payment with available balance within a
		// transaction
		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT_FEE_CHARGED,
			payment_amount,
			None
		));

		assert_ok!(with_transaction(|| TransactionOutcome::Commit({
			<PaymentModule as PaymentHandler<Test>>::settle_payment(
				&PAYMENT_CREATOR,
				&PAYMENT_RECIPENT_FEE_CHARGED,
				Percent::from_percent(70),
			)
		})));

		let expected_amount_for_creator = creator_initial_balance - payment_amount - expected_fee_amount
			+ (Percent::from_percent(30) * payment_amount);
		let expected_amount_for_recipient = Percent::from_percent(70) * payment_amount;

		// the payment amount should be transferred
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			expected_amount_for_creator
		);
		assert_eq!(
			Balances::free_balance(&PAYMENT_RECIPENT_FEE_CHARGED),
			expected_amount_for_recipient.saturating_add(recipient_initial_balance)
		);
		assert_eq!(
			Balances::free_balance(&FEE_RECIPIENT_ACCOUNT),
			expected_fee_amount.saturating_add(recipient_initial_balance)
		);

		// should be deleted from storage
		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT_FEE_CHARGED),
			None
		);
	});
}

#[test]
fn test_settle_payment_works_for_50_50() {
	new_test_ext().execute_with(|| {
		let creator_initial_balance = 100_000_000_000;
		let recipient_initial_balance = 1;
		let payment_amount = 10;
		let expected_fee_amount = payment_amount / MARKETPLACE_FEE_PERCENTAGE as u64;

		// the payment amount should not be reserved
		assert_eq!(Balances::free_balance(&PAYMENT_CREATOR), 100_000_000_000);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT_FEE_CHARGED), 1);

		// should be able to create a payment with available balance within a
		// transaction
		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT_FEE_CHARGED,
			payment_amount,
			None
		));

		assert_ok!(with_transaction(|| TransactionOutcome::Commit({
			<PaymentModule as PaymentHandler<Test>>::settle_payment(
				&PAYMENT_CREATOR,
				&PAYMENT_RECIPENT_FEE_CHARGED,
				Percent::from_percent(50),
			)
		})));

		let expected_amount_for_creator = creator_initial_balance - payment_amount - expected_fee_amount
			+ (Percent::from_percent(50) * payment_amount);
		let expected_amount_for_recipient = Percent::from_percent(50) * payment_amount;

		// the payment amount should be transferred
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			expected_amount_for_creator
		);
		assert_eq!(
			Balances::free_balance(&PAYMENT_RECIPENT_FEE_CHARGED),
			expected_amount_for_recipient.saturating_add(recipient_initial_balance)
		);
		assert_eq!(
			Balances::free_balance(&FEE_RECIPIENT_ACCOUNT),
			expected_fee_amount.saturating_add(recipient_initial_balance)
		);

		// should be deleted from storage
		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT_FEE_CHARGED),
			None
		);
	});
}

#[test]
fn test_automatic_refund_works() {
	new_test_ext().execute_with(|| {
		let creator_initial_balance = 100_000_000_000;
		let payment_amount = 20;
		let expected_incentive_amount = payment_amount / INCENTIVE_PERCENTAGE as u64;
		const CANCEL_PERIOD: u64 = 600;
		const CANCEL_BLOCK: u64 = CANCEL_PERIOD + 1;

		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			payment_amount,
			None
		));

		assert_ok!(PaymentModule::request_refund(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT
		));

		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT),
			Some(PaymentDetail {
				amount: payment_amount,
				incentive_amount: expected_incentive_amount,
				state: PaymentState::RefundRequested {
					cancel_block: CANCEL_BLOCK
				},
				resolver_account: RESOLVER_ACCOUNT,
				fee_detail: Some((FEE_RECIPIENT_ACCOUNT, 0)),
			})
		);

		let scheduled_tasks_list = ScheduledTasks::<Test>::get();
		assert_eq!(
			scheduled_tasks_list.get(&(PAYMENT_CREATOR, PAYMENT_RECIPENT)).unwrap(),
			&ScheduledTask {
				task: Task::Cancel,
				when: CANCEL_BLOCK
			}
		);

		// run to one block before cancel and make sure data is same
		assert_eq!(run_n_blocks(CANCEL_PERIOD - 1), 600);
		let scheduled_tasks_list = ScheduledTasks::<Test>::get();
		assert_eq!(
			scheduled_tasks_list.get(&(PAYMENT_CREATOR, PAYMENT_RECIPENT)).unwrap(),
			&ScheduledTask {
				task: Task::Cancel,
				when: CANCEL_BLOCK
			}
		);

		// run to after cancel block but odd blocks are busy
		assert_eq!(run_n_blocks(1), 601);
		// the payment is still not processed since the block was busy
		assert!(PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT).is_some());

		// next block has spare weight to process the payment
		assert_eq!(run_n_blocks(1), 602);
		// the payment should be removed from storage
		assert_eq!(PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT), None);

		// the scheduled storage should be cleared
		let scheduled_tasks_list = ScheduledTasks::<Test>::get();
		assert_eq!(scheduled_tasks_list.get(&(PAYMENT_CREATOR, PAYMENT_RECIPENT)), None);

		// test that the refund happened correctly
		assert_eq!(
			last_event(),
			crate::Event::<Test>::PaymentCancelled {
				from: PAYMENT_CREATOR,
				to: PAYMENT_RECIPENT
			}
			.into()
		);
		// the payment amount should be released back to creator
		assert_eq!(
			Balances::free_balance(&PAYMENT_CREATOR),
			creator_initial_balance
		);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), 1);
	});
}

#[test]
fn test_automatic_refund_works_for_multiple_payments() {
	new_test_ext().execute_with(|| {
		const CANCEL_PERIOD: u64 = 600;

		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT,
			20,
			None
		));

		assert_ok!(PaymentModule::pay(
			Origin::signed(PAYMENT_CREATOR_TWO),
			PAYMENT_RECIPENT_TWO,
			20,
			None
		));

		assert_ok!(PaymentModule::request_refund(
			Origin::signed(PAYMENT_CREATOR),
			PAYMENT_RECIPENT
		));
		run_n_blocks(1);
		assert_ok!(PaymentModule::request_refund(
			Origin::signed(PAYMENT_CREATOR_TWO),
			PAYMENT_RECIPENT_TWO
		));

		assert_eq!(run_n_blocks(CANCEL_PERIOD - 1), 601);

		// Odd block 601 was busy so we still haven't processed the first payment
		assert_ok!(PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT).ok_or(()));

		// Even block 602 has enough room to process both pending payments
		assert_eq!(run_n_blocks(1), 602);
		assert_eq!(PaymentStore::<Test>::get(PAYMENT_CREATOR, PAYMENT_RECIPENT), None);
		assert_eq!(
			PaymentStore::<Test>::get(PAYMENT_CREATOR_TWO, PAYMENT_RECIPENT_TWO),
			None
		);

		// the scheduled storage should be cleared
		let scheduled_tasks_list = ScheduledTasks::<Test>::get();
		assert_eq!(scheduled_tasks_list.get(&(PAYMENT_CREATOR, PAYMENT_RECIPENT)), None);
		assert_eq!(
			scheduled_tasks_list.get(&(PAYMENT_CREATOR_TWO, PAYMENT_RECIPENT_TWO)),
			None
		);

		// test that the refund happened correctly
		assert_eq!(Balances::free_balance(&PAYMENT_CREATOR), 100_000_000_000);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT), 1);

		assert_eq!(Balances::free_balance(&PAYMENT_CREATOR_TWO), 100_000_000_000);
		assert_eq!(Balances::free_balance(&PAYMENT_RECIPENT_TWO), 1);
	});
}