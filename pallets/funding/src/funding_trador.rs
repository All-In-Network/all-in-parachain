use frame_support::{
	ensure,
	traits::{
		tokens::{nonfungibles::InspectEnumerable, ExistenceRequirement},
		Currency, UnixTime, InstanceFilter, IsSubType
	},
	transactional, BoundedVec,
	weights::GetDispatchInfo,
	weights::Pays,
	dispatch::Dispatchable
};
use frame_system::{ensure_signed, pallet_prelude::*, Origin};

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::{sr25519, H256};
use sp_runtime::DispatchResult;
use sp_std::prelude::*;
use sp_runtime::{RuntimeDebug};
use serde::{Deserialize, Serialize};

pub use pallet::*;

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TradorInfo<AccountId> {
	proxy_funding_account: AccountId,
	is_funded: bool,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::{
		dispatch::DispatchResult, pallet_prelude::*, traits::ReservableCurrency, Blake2_128Concat, PalletId
	};

	type TypeProxyOf<T> = <T as pallet_proxy::Config>::ProxyType;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_proxy::Config {

		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The funding pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The origin which may forcibly buy, sell, list/unlist, offer & withdraw offer on Tokens
		type GovernanceOrigin: EnsureOrigin<Self::Origin>;

		type Currency: ReservableCurrency<Self::AccountId>;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// manager admin acc
	#[pallet::storage]
	pub(super) type Manager<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn trador_is_funded)]
	pub(super) type IsTradorFunded<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, TradorInfo<T::AccountId>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ManagerChanged {
			old_manager: Option<T::AccountId>,
			new_manager: T::AccountId,
		},
		NewAccountCreate,
		NewTradorRegistered {
			trador_account: T::AccountId,
			proxy_funding_account: T::AccountId,
			is_funded: bool,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		ManagerNotSet,
		RequireManagerAccount,
		TradorAlreadyFunded,
		TradorInfoProxyAccountNotMatch,
		TradorNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Privileged function set the Manager Admin account
		///
		/// Parameters:
		/// - origin: Expected to be called by `GovernanceOrigin`
		/// - new_manager: T::AccountId
		#[pallet::weight(0)]
		pub fn set_manager(
			origin: OriginFor<T>,
			new_manager: T::AccountId,
		) -> DispatchResultWithPostInfo {
			// This is a public call, so we ensure that the origin is some signed account.
			T::GovernanceOrigin::ensure_origin(origin)?;
			let old_manager = <Manager<T>>::get();

			Manager::<T>::put(&new_manager);
			Self::deposit_event(Event::ManagerChanged { old_manager, new_manager });
			// Manager user does not pay a fee
			Ok(Pays::No.into())
		}

        /// Privileged function that create a new proxy account
		/// from manager account, this new proxy is ready for link a new trader
        ///
		/// Parameters:
		/// - origin: manager account
		/// - proxy_type: permission type to create a new proxy account
        /// - delay: timer duration for creation
        /// - index: index to create a new proxy account
		#[pallet::weight(10_000)]
		pub fn create_new_funding_account(
			origin: OriginFor<T>,
			proxy_type: TypeProxyOf<T>,
			delay: T::BlockNumber,
			index: u16,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			Self::ensure_manager(&sender)?;

			pallet_proxy::Pallet::<T>::anonymous(origin.clone(), proxy_type, delay, index)?;

			Self::deposit_event(Event::<T>::NewAccountCreate);
			Ok(())
		}

        /// Privileged function that register new trader account
        ///
		/// Parameters:
		/// - origin: manager account
		/// - proxy_funding_account: the proxy account recently created
        /// - trador_account: the account for the trader "user"
        /// - proxy_type: Optional permission, this must be Notranfer for default
		#[pallet::weight(10_000)]
		pub fn register_new_trador(
			origin: OriginFor<T>,
			proxy_funding_account: T::AccountId,
			trador_account: T::AccountId,
			proxy_type: Option<TypeProxyOf<T>>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin.clone())?;
			Self::ensure_manager(&sender)?;

			if let Some(trador_info) = <IsTradorFunded::<T>>::get(&trador_account) {
				ensure!(!trador_info.is_funded, Error::<T>::TradorAlreadyFunded);
			} else {
                // check if proxy already exists in storage
				pallet_proxy::Pallet::<T>::find_proxy(&proxy_funding_account, &sender, proxy_type)?;

				let new_trador = TradorInfo {
					proxy_funding_account: proxy_funding_account.clone(),
					is_funded: true,
				};
				<IsTradorFunded::<T>>::insert(trador_account.clone(), new_trador.clone());
				Self::deposit_event(Event::<T>::NewTradorRegistered {
					trador_account: trador_account,
					proxy_funding_account: new_trador.proxy_funding_account,
					is_funded: new_trador.is_funded,
				});
			}
			Ok(Pays::No.into())
		}


        /// function that initialize the proxy account with
		/// trader account, only the proxy account created can
        /// call this extrinsic
        ///
		/// Parameters:
		/// - proxy_funding_account: the proxy account recently created
        /// - trador_account: the account for the trader "user"
        /// - proxy_type: Optional permission, this must be Notranfer for default
        /// - delay: timer duration for creation
		#[pallet::weight(0)]
		pub fn initialize_trador_account(
			proxy_funding_account: OriginFor<T>,
			trador_account: T::AccountId,
			proxy_type: TypeProxyOf<T>,
			delay: T::BlockNumber,
		) -> DispatchResult {
			let sender = ensure_signed(proxy_funding_account.clone())?;
			if let Some(trador_info) = <IsTradorFunded::<T>>::get(&trador_account) {
				ensure!(trador_info.is_funded, Error::<T>::TradorNotFound);
				ensure!(trador_info.proxy_funding_account == sender, Error::<T>::TradorInfoProxyAccountNotMatch);

				pallet_proxy::Pallet::<T>::add_proxy_delegate(&sender, trador_account, proxy_type, delay)?;
			}
			Ok(())
		}
	}

	// helper functions
	impl<T: Config> Pallet<T> {
		pub(crate) fn manager() -> Result<T::AccountId, Error<T>> {
			Manager::<T>::get().ok_or(Error::<T>::ManagerNotSet)
		}

		/// Helper function to ensure Manager account is the sender
		///
		/// Parameters:
		/// - `sender`: Account origin that made the call to check if Manager account
		pub(crate) fn ensure_manager(sender: &T::AccountId) -> DispatchResult {
			ensure!(
				Self::manager().map_or(false, |k| sender == &k),
				Error::<T>::RequireManagerAccount
			);
			Ok(())
		}
	}
}