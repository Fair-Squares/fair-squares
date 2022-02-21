#![cfg(feature = "runtime-benchmarks")]

use crate::Pallet as NFT;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use sp_runtime::traits::StaticLookup;

pub use crate::*;

const SEED: u32 = 0;

benchmarks! {
	// create simple NFT class
	create_class {
		let alice: T::AccountId = account("alice", 0, SEED);
		<T as pallet::Config>::Currency::make_free_balance_be(&alice, (CREATION_FEE + 10).into());
	}: _(RawOrigin::Signed(alice),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Simple(999999999)
		)

	// mint simple NFT instances
	mint {
		let i in 1 .. 1000;

		let alice: T::AccountId = account("alice", 0, SEED);
		let bob: T::AccountId = account("bob", 0, SEED);
		let bob_lookup = T::Lookup::unlookup(bob);

		<T as pallet::Config>::Currency::make_free_balance_be(&alice, (CREATION_FEE + 10).into());

		crate::Pallet::<T>::create_class(
			RawOrigin::Signed(alice.clone()).into(),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Simple(999999999)
		)?;
	}: _(RawOrigin::Signed(alice), bob_lookup, 0u32.into(), vec![1], i)

	// TODO: use a more realistic Merkle tree
	// claim a claim class NFT
	claim {
		let alice: T::AccountId = account("alice", 0, SEED);

		// account id of bob 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
		let bob_bytes = [
			0xd4, 0x35, 0x93, 0xc7, 0x15, 0xfd, 0xd3, 0x1c, 0x61, 0x14, 0x1a, 0xbd, 0x04, 0xa9,
			0x9f, 0xd6, 0x82, 0x2c, 0x85, 0x58, 0x85, 0x4c, 0xcd, 0xe3, 0x9a, 0x56, 0x84, 0xe7,
			0xa5, 0x6d, 0xa2, 0x7d,
		];
		let bob = T::AccountId::decode(&mut &bob_bytes[..]).expect("32 bytes can always construct an AccountId32");

		// root is 0xa8a5ec29a3df3c5a8aa6fd2935d2414cf0ce4f748a13bb2833214c3b94a6d3b3
		let merkle_root = [
			0xa8, 0xa5, 0xec, 0x29, 0xa3, 0xdf, 0x3c, 0x5a, 0x8a, 0xa6, 0xfd, 0x29, 0x35, 0xd2,
			0x41, 0x4c, 0xf0, 0xce, 0x4f, 0x74, 0x8a, 0x13, 0xbb, 0x28, 0x33, 0x21, 0x4c, 0x3b,
			0x94, 0xa6, 0xd3, 0xb3,
		];

		// proof of bob is 0x5182a73e48bd6e814d0c2b41672d9cb8c87c4221b55bc08e0943198e90caad1f
		let bob_proof = vec![[
			0x51u8, 0x82u8, 0xa7u8, 0x3eu8, 0x48u8, 0xbdu8, 0x6eu8, 0x81u8, 0x4du8, 0x0cu8, 0x2bu8,
			0x41u8, 0x67u8, 0x2du8, 0x9cu8, 0xb8u8, 0xc8u8, 0x7cu8, 0x42u8, 0x21u8, 0xb5u8, 0x5bu8,
			0xc0u8, 0x8eu8, 0x09u8, 0x43u8, 0x19u8, 0x8eu8, 0x90u8, 0xcau8, 0xadu8, 0x1fu8,
		]];

		<T as pallet::Config>::Currency::make_free_balance_be(&alice, (CREATION_FEE + 10).into());

		crate::Pallet::<T>::create_class(
			RawOrigin::Signed(alice.clone()).into(),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Claim(merkle_root)
		)?;
	}: _(RawOrigin::Signed(bob), 0, 0u32.into(), bob_proof)

	// merge two simple NFT instances
	merge {
		let total = 1000;
		let i in 0 .. 999;

		let alice: T::AccountId = account("alice", 0, SEED);
		let bob: T::AccountId = account("bob", 0, SEED);
		let bob_lookup = T::Lookup::unlookup(bob.clone());

		<T as pallet::Config>::Currency::make_free_balance_be(&alice, (3 * CREATION_FEE + 10).into());

		crate::Pallet::<T>::create_class(
			RawOrigin::Signed(alice.clone()).into(),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Simple(999999999)
		)?;

		crate::Pallet::<T>::create_class(
			RawOrigin::Signed(alice.clone()).into(),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Simple(999999999)
		)?;

		crate::Pallet::<T>::mint(RawOrigin::Signed(alice.clone()).into(), bob_lookup.clone(), 0u32.into(), vec![1], total)?;

		crate::Pallet::<T>::create_class(
			RawOrigin::Signed(alice.clone()).into(),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Merge(0u32.into(), 1u32.into(), false)
		)?;

		crate::Pallet::<T>::mint(RawOrigin::Signed(alice).into(), bob_lookup, 1u32.into(), vec![1], total)?;

	}: _(RawOrigin::Signed(bob), 2u32.into(),  (0u32.into(), i.into()), (1u32.into(), i.into()))

	// transfer a simple NFT instance
	transfer {
		let alice: T::AccountId = account("alice", 0, SEED);
		let alice_lookup = T::Lookup::unlookup(alice.clone());
		let bob: T::AccountId = account("bob", 0, SEED);
		let bob_lookup = T::Lookup::unlookup(bob.clone());

		<T as pallet::Config>::Currency::make_free_balance_be(&alice, (CREATION_FEE + 10).into());

		crate::Pallet::<T>::create_class(
			RawOrigin::Signed(alice.clone()).into(),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Simple(999999999)
		)?;

		crate::Pallet::<T>::mint(RawOrigin::Signed(alice).into(), bob_lookup, 0u32.into(), vec![1], 1)?;
	}: _(RawOrigin::Signed(bob), alice_lookup, (0u32.into(), 0u32.into()))

	// burn a simple NFT instance
	burn {
		let alice: T::AccountId = account("alice", 0, SEED);
		let bob: T::AccountId = account("bob", 0, SEED);
		let bob_lookup = T::Lookup::unlookup(bob.clone());

		<T as pallet::Config>::Currency::make_free_balance_be(&alice, (CREATION_FEE + 10).into());

		crate::Pallet::<T>::create_class(
			RawOrigin::Signed(alice.clone()).into(),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Simple(999999999)
		)?;
		crate::Pallet::<T>::mint(RawOrigin::Signed(alice).into(), bob_lookup, 0u32.into(), vec![1], 1)?;
	}: _(RawOrigin::Signed(bob), (0u32.into(), 0u32.into()))

}

impl_benchmark_test_suite!(NFT, crate::mock::new_test_ext(), crate::mock::Test,);
