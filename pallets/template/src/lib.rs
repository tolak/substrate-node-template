#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error, debug, dispatch, traits::Get, Parameter};
use frame_system::{ensure_root, ensure_signed};
use sp_std::{prelude::*, fmt::Debug};
use sp_runtime::{
	traits::{AtLeast32Bit, MaybeSerializeDeserialize, MaybeDisplay, Member},
	RuntimeDebug,
};

use codec::{Decode, Encode};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	type Balance: Parameter + Member + AtLeast32Bit + Default + Copy + MaybeSerializeDeserialize;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as TemplateModule {
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		Something get(fn something): Option<u32>;
		
		/// The current set of members, ordered.
		pub Members get(fn members) build(|config: &GenesisConfig<T>| {
			let mut m = config.members.clone();
			m.sort();
			m
		}): Vec<T::AccountId>;

		/// The balance of memebers
		pub Balances get(fn balances): map hasher(twox_64_concat) T::AccountId => T::Balance;

	}

	add_extra_genesis {
		config(members): Vec<T::AccountId>;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where 
		AccountId = <T as frame_system::Trait>::AccountId,
		Balance = <T as Trait>::Balance
	{
		/// Event documentation should end with an array that provides descriptive names for event. [something, who]
		SomethingStored(u32),

		/// Add account into member list. [member]
		MemberAdded(AccountId),

		/// Remove account from memeber list. [member]
		MemberRemoved(AccountId),

		/// Update balance of memebers. [member, old_balance, new_balance]
		BalanceUpdated(AccountId, Balance, Balance),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			// Update storage.
			Something::put(something);

			// Emit an event.
			Self::deposit_event(RawEvent::SomethingStored(something));
			// Return a successful DispatchResult
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn cause_error(origin) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match Something::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					Something::put(new);
					Ok(())
				},
			}
		}

		/// Method to insert account into member list.
		#[weight = 10]
		pub fn insert_member(origin, member: T::AccountId) {
			ensure_root(origin)?;

			// if memeber doesn't exist, insert into member list
			if let Some(pos) = Self::members().iter().position(|x| *x == member) {
				debug::info!("Member {:?} already exist in memeber list.", member);
			} else {
				let mut members = Self::members();
				members.push(member.clone());
				<Members::<T>>::put(members);

				Self::deposit_event(RawEvent::MemberAdded(member.clone()));
				debug::info!("New member {:?} has been added into memeber list.", member);
			}
		}

		/// Method to remove account from member list
		#[weight = 10]
		pub fn remove_member(origin, member: T::AccountId) {
			ensure_root(origin)?;

			
			let mut members = Self::members();
			if let Some(pos) = Self::members().iter().position(|x| *x == member) {
				members.remove(pos);
				<Members::<T>>::put(members);
				Self::deposit_event(RawEvent::MemberRemoved(member.clone()));
				debug::info!("New member {:?} has been removed from memeber list.", member);
			}
		}
	}
}
