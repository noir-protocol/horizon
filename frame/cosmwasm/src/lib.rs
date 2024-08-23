use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

#[allow(clippy::too_many_arguments)]
#[frame_support::pallet]
pub mod pallet {
	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::pallet]
	pub struct Pallet<T>(_);
}
