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

