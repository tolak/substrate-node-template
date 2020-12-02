#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{ensure, decl_module, decl_storage, decl_event, decl_error, debug, dispatch, 
	traits::{Get, Currency, ExistenceRequirement, WithdrawReason}, 
	Parameter
};
use frame_system::{ensure_root, ensure_signed};
use sp_std::{prelude::*, fmt::Debug};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, MaybeDisplay, Member, CheckedAdd, CheckedSub},
	RuntimeDebug, DispatchResult
};

use codec::{Decode, Encode};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
	<<T as Trait>::OwnedCurrency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;

	type OwnedCurrency: Currency<Self::AccountId>;
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
		pub Members get(fn members): Vec<T::AccountId>;

		/// The balance of memebers
		pub Balances get(fn balances): map hasher(twox_64_concat) T::AccountId => BalanceOf<T>;

	}

	add_extra_genesis {
		config(balances): Vec<(T::AccountId, BalanceOf<T>)>;
		config(members): Vec<T::AccountId>;
		build(|config: &GenesisConfig<T>| {
			for &(ref who, balance) in config.balances.iter() {
				assert!(
					balance >= 1.into(),
					"the balance of any account should always be more than existential deposit.",
				);
				Balances::<T>::insert(who, balance);
			}

			let mut m = config.members.clone();
			m.sort();
			Members::<T>::put(m);
		});
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where 
		AccountId = <T as frame_system::Trait>::AccountId,
		Balance = BalanceOf<T>
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
		/// Erros user does not exist when change balance
		UserDoesNotExist,
		/// Erros banalce overflow
		BalanceOverFlow,
		/// Erros balance insufficient
		BalanceInsufficient,
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

		/// Method to add balance of given account
		#[weight = 10]
		pub fn balance_add(origin, balance: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Balances::<T>::contains_key(who.clone()), Error::<T>::UserDoesNotExist);

			let mut old_value: BalanceOf<T> = 0.into();
			let mut new_value: BalanceOf<T> = 0.into();
			Balances::<T>::mutate(who.clone(), |b| -> DispatchResult {
				old_value = *b;
				*b = b.checked_add(&balance).ok_or(Error::<T>::BalanceOverFlow)?;
				new_value = *b;

				Ok(())
			})?;

			T::OwnedCurrency::deposit_creating(&who, balance);

			// Emit transfer event.
			Self::deposit_event(RawEvent::BalanceUpdated(who.clone(), old_value, new_value));
			Ok(())
		}

		/// Method to sub balance of given account
		#[weight = 10]
		pub fn balance_sub(origin, balance: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Balances::<T>::contains_key(who.clone()), Error::<T>::UserDoesNotExist);

			let mut old_value: BalanceOf<T> = 0.into();
			let mut new_value: BalanceOf<T> = 0.into();
			Balances::<T>::mutate(who.clone(), |b| -> DispatchResult {
				old_value = *b;
				*b = b.checked_sub(&balance).ok_or(Error::<T>::BalanceInsufficient)?;
				new_value = *b;

				Ok(())
			})?;

			let _ = T::OwnedCurrency::withdraw(
				&who,
				balance,
				WithdrawReason::Transfer.into(),
				ExistenceRequirement::AllowDeath,
			)?;


			// Emit transfer event.
			Self::deposit_event(RawEvent::BalanceUpdated(who.clone(), old_value, new_value));

			Ok(())
		}

		/// Method to insert account with given balance
		#[weight = 10]
		pub fn insert_account(origin, balance: BalanceOf<T>) -> DispatchResult {

			let who = ensure_signed(origin)?;
			if !Balances::<T>::contains_key(who.clone()) {
				Balances::<T>::insert(who.clone(), balance);
			}

			Ok(())
		}
	}
}
