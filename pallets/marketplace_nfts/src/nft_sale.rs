//! Soulbound nft sale
use frame_support::{
	ensure,
	traits::{
		tokens::{nonfungibles::InspectEnumerable},
		Currency,
	},
	transactional, BoundedVec,
};

use frame_system::{ensure_signed, pallet_prelude::*, Origin};

use sp_std::prelude::*;

pub use pallet_rmrk_core::types::*;
pub use pallet_rmrk_market;
use rmrk_traits::{Nft};
use rmrk_traits::primitives::*;

pub use crate::types::{
	StatusType
};

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    use frame_support::{
		dispatch::DispatchResult, pallet_prelude::*, traits::ReservableCurrency,
	};
	use pallet_rmrk_core::BoundedCollectionSymbolOf;

    #[pallet::config]
	pub trait Config: frame_system::Config + pallet_rmrk_core::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The origin which may forcibly buy, sell, list/unlist, offer & withdraw offer on Tokens
		type GovernanceOrigin: EnsureOrigin<Self::Origin>;

		type Currency: ReservableCurrency<Self::AccountId>;

		/// minimum amount of token to claim Soulbound		#[pallet::constant]
		type MinBalanceToClaimSoulbound: Get<BalanceOf<Self>>;
	}

    #[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Manager<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn can_claim_soulbound)]
	pub type CanClaimSoulbound<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn trador_soulbound_collection_id)]
	pub type TradorSoulboundCollectionId<T: Config> = StorageValue<_, CollectionId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn trador_soulbound_metadata)]
	pub type TradorSoulboundMetadata<T: Config> = StorageValue<_, BoundedVec<u8, T::StringLimit>>;

    #[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// admin account
		pub manager: Option<T::AccountId>,
		/// bool if trador Soulbound is clamaible
		pub can_claim_soulbound: bool,
		/// collection id of trador Soulbound
		pub trador_soulbound_collection_id: Option<CollectionId>,
	}

    #[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				manager: None,
				can_claim_soulbound: false,
				trador_soulbound_collection_id: None,
			}
		}
	}

    #[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T>
	where
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
	{
		fn build(&self) {
			if let Some(ref manager) = self.manager {
				<Manager<T>>::put(manager);
			}

			<CanClaimSoulbound<T>>::put(self.can_claim_soulbound);

			if let Some(trador_soulboundcollection_id) = self.trador_soulbound_collection_id {
				<TradorSoulboundCollectionId<T>>::put(trador_soulboundcollection_id);
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TradorSoulboundClaimed {
			owner: T::AccountId,
			collection_id: CollectionId,
			nft_id: NftId,
		},

		TradorSoulboundCollectionIdSet {
			collection_id: CollectionId,
		},

		ClaimSoulboundStatusChanged {
			status: bool,
		},

		ManagerChanged {
			old_manager: Option<T::AccountId>,
			new_manager: T::AccountId,
		},

		TradorSoulboundMetadataSet {
			soulbound_metadata: BoundedVec<u8, T::StringLimit>,
		},
	}

    #[pallet::error]
	pub enum Error<T> {
		SoulboundClaimNotAvailable,
		SoulboundAlreadyClaimed,
		BelowMinimumBalanceThreshold,
		SoulboundCollectionNotSet,
		SoulboundCollectionIdAlreadySet,
		ManagerNotSet,
		RequireManagerAccount,
		SoulboundMetadataNotSet,
	}

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
	{
        // claim soulbound for any account with founds in their account
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn claim_soulbound(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			ensure!(CanClaimSoulbound::<T>::get(), Error::<T>::SoulboundClaimNotAvailable);
			let manager = Self::manager()?;

			ensure!(
				<T as pallet::Config>::Currency::can_reserve(
					&sender,
					T::MinBalanceToClaimSoulbound::get()
				),
				Error::<T>::BelowMinimumBalanceThreshold
			);
			Self::do_mint_soulbound_nft(manager, sender)?;

			Ok(())
		}

        /// Privileged function to set the metadata for the Soulbound in the StorageValue
		/// `SoulboundMetadata` where the value is a `BoundedVec<u8, T::StringLimit`.
		///
		/// Parameters:
		/// - `origin`: Expected to be called from the Manager account
		/// - `soulbound_metadata`: `BoundedVec<u8, T::StringLimit>` to be added in storage
		#[pallet::weight(0)]
		pub fn set_soulbound_metadata(
			origin: OriginFor<T>,
			soulbound_metadata: BoundedVec<u8, T::StringLimit>,
		) -> DispatchResultWithPostInfo {
			// Ensure Manager account makes call
			let manager = ensure_signed(origin.clone())?;
			Self::ensure_manager(&manager)?;
			// Set Soulbound Metadata
			TradorSoulboundMetadata::<T>::put(soulbound_metadata.clone());

			Self::deposit_event(Event::TradorSoulboundMetadataSet { soulbound_metadata });

			Ok(Pays::No.into())
		}

        /// Privileged function to allow Manager to mint the Soulbound
		/// Collections. This allows for only Manager to be able to mint Collections &
		/// prevents other users from calling the RMRK Core `create_collection` function.
		///
		/// Parameters:
		/// - `origin`: Expected to be called by Manager
		/// - `metadata`: Metadata to the collection
		/// - `max`: Optional max u32 for the size of the collection
		/// - `symbol`: BoundedString of the collection's symbol for example 'TRDR'
		#[pallet::weight(T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn trador_create_collection(
			origin: OriginFor<T>,
			metadata: BoundedVec<u8, T::StringLimit>,
			max: Option<u32>,
			symbol: BoundedCollectionSymbolOf<T>,
		) -> DispatchResult {
			// Ensure Manager account makes call
			let sender = ensure_signed(origin.clone())?;
			Self::ensure_manager(&sender)?;

			pallet_rmrk_core::Pallet::<T>::create_collection(origin, metadata, max, symbol)?;

			Ok(())
		}

        /// Privileged function to set the collection id for the Soubound collection
		///
		/// Parameters:
		/// - `origin` - Expected Manager admin account to set the Soulbound Collection ID
		/// - `collection_id` - Collection ID of the Soulbound Collection
		#[pallet::weight(0)]
		pub fn set_soulbound_collection_id(
			origin: OriginFor<T>,
			collection_id: CollectionId,
		) -> DispatchResultWithPostInfo {
			// Ensure Manager account makes call
			let sender = ensure_signed(origin)?;
			Self::ensure_manager(&sender)?;
			// If Soulbound Collection ID is greater than 0 then the collection ID was already set
			ensure!(
				TradorSoulboundCollectionId::<T>::get().is_none(),
				Error::<T>::SoulboundCollectionIdAlreadySet
			);
			<TradorSoulboundCollectionId<T>>::put(collection_id);

			Self::deposit_event(Event::TradorSoulboundCollectionIdSet { collection_id });

			Ok(Pays::No.into())
		}

        /// Privileged function set the Manager Admin account of Trador
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

        /// Privileged function to set the status for one of the defined StatusTypes like
		/// ClaimSoulbound
		///
		/// Parameters:
		/// - `origin` - Expected Manager admin account to set the status
		/// - `status` - `bool` to set the status to
		/// - `status_type` - `StatusType` to set the status for
		#[pallet::weight(0)]
		pub fn set_status_type(
			origin: OriginFor<T>,
			status: bool,
			status_type: StatusType,
		) -> DispatchResultWithPostInfo {
			// Ensure Manager account makes call
			let sender = ensure_signed(origin)?;
			Self::ensure_manager(&sender)?;
			// Match StatusType and call helper function to set status
			match status_type {
				StatusType::ClaimSoulbound => Self::set_claim_soulbound_status(status)?,
			}
			Ok(Pays::No.into())
		}

    }

    impl<T: Config> Pallet<T>
	where
		T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
	{
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

		/// mint trador soulbound nft
		/// parameters:
		/// manager: account owns the nft collection and will mint the nft and freeze it
		/// sender: new owner of the soulbound nft
		fn do_mint_soulbound_nft(manager: T::AccountId, sender: T::AccountId) -> DispatchResult {
			let soulbound_collection_id = Self::get_soulbound_collection_id()?;

			ensure!(
				!Self::owns_nft_in_collection(&sender, soulbound_collection_id),
				Error::<T>::SoulboundAlreadyClaimed
			);

			let metadata =
				TradorSoulboundMetadata::<T>::get().ok_or(Error::<T>::SoulboundMetadataNotSet)?;
			let collection = pallet_rmrk_core::Pallet::<T>::collections(soulbound_collection_id)
				.ok_or(pallet_rmrk_core::Error::<T>::CollectionUnknown)?;
			let nft_id = collection.nfts_count + 1;

			let (_, soulbound_nft_id) = pallet_rmrk_core::Pallet::<T>::nft_mint(
				sender.clone(),
				sender.clone(),
				nft_id,
				soulbound_collection_id,
				None,
				None,
				metadata,
				true,
				None,
			)?;

			pallet_uniques::Pallet::<T>::freeze(
				Origin::<T>::Signed(manager).into(),
				soulbound_collection_id,
				soulbound_nft_id,
			)?;

			Self::deposit_event(Event::TradorSoulboundClaimed {
				owner: sender,
				collection_id: soulbound_collection_id,
				nft_id: soulbound_nft_id,
			});

			Ok(())
		}

		/// Helper function to check if owner has a NFT within a collection
		fn owns_nft_in_collection(sender: &T::AccountId, collection_id: CollectionId) -> bool {
			pallet_uniques::Pallet::<T>::owned_in_collection(&collection_id, sender).count() > 0
		}

		/// helper function to get the soulbound collection id
		fn get_soulbound_collection_id() -> Result<CollectionId, Error<T>> {
			let soulbound_collection_id =
                TradorSoulboundCollectionId::<T>::get().ok_or(Error::<T>::SoulboundCollectionNotSet)?;
			Ok(soulbound_collection_id)
		}

		/// Set Soulbound Claims with the Manager admin Account to allow users to claim their
		/// Soulbound through the `claim_soulbounds()` function
		///
		/// Parameters:
		/// - `status`: Status to set CanClaimSoulbounds StorageValue
		fn set_claim_soulbound_status(status: bool) -> DispatchResult {
			<CanClaimSoulbound<T>>::put(status);

			Self::deposit_event(Event::ClaimSoulboundStatusChanged { status });

			Ok(())
		}
	}

}