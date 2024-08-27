use crate::{
	prelude::*,
	runtimes::vm::CosmwasmVMError,
	types::{AccountIdOf, DefaultCosmwasmVM},
	Config, Pallet,
};
use cosmwasm_vm::{
	executor::{
		ibc::{
			IbcChannelCloseCall, IbcChannelConnectCall, IbcChannelOpenCall, IbcPacketAckCall,
			IbcPacketReceiveCall, IbcPacketTimeoutCall,
		},
		AllocateCall, AsFunctionName, CosmwasmCallInput, CosmwasmCallWithoutInfoInput,
		DeallocateCall, HasInfo, Unit,
	},
	input::Input,
	memory::PointerOf,
	vm::{VmErrorOf, VmInputOf, VmOutputOf},
};
use ibc::{
	core::{
		client::types::Height,
		host::types::identifiers::{ChannelId, PortId},
	},
	primitives::Timestamp,
};
use ibc_primitives::HandlerMessage;

use crate::types::EntryPoint::{self, *};

pub struct ChannelOpenCall;
impl Input for ChannelOpenCall {
	type Output = cosmwasm_vm::executor::ibc::IbcChannelOpenResult;
}
impl AsFunctionName for ChannelOpenCall {
	const NAME: &'static str = "ibc_channel_open";
}
impl HasInfo for ChannelOpenCall {
	const HAS_INFO: bool = false;
}

impl cosmwasm_vm::system::EventIsTyped for ChannelOpenCall {
	const TYPE: cosmwasm_vm::system::SystemEventType =
		cosmwasm_vm::system::SystemEventType::IbcChannelConnect;
}

impl cosmwasm_vm::system::EventHasCodeId for ChannelOpenCall {
	const HAS_CODE_ID: bool = false;
}

impl<T: Config> Pallet<T> {
	/// Check whether a contract export the mandatory IBC functions and is consequently IBC capable.
	pub(crate) fn do_check_ibc_capability(_module: &parity_wasm::elements::Module) -> bool {
		false
	}

	pub fn do_ibc_transfer(
		_vm: &mut DefaultCosmwasmVM<T>,
		_channel_id: String,
		_to_address: String,
		_amount: cosmwasm_std::Coin,
		_timeout: cosmwasm_std::IbcTimeout,
	) -> Result<(), CosmwasmVMError<T>> {
		Err(CosmwasmVMError::<T>::Unsupported)
	}

	pub(crate) fn do_ibc_send_packet(
		_vm: &mut DefaultCosmwasmVM<T>,
		_channel_id: String,
		_data: cosmwasm_std::Binary,
		_timeout: cosmwasm_std::IbcTimeout,
	) -> Result<(), CosmwasmVMError<T>> {
		Err(CosmwasmVMError::<T>::Unsupported)
	}

	pub(crate) fn do_ibc_close_channel(
		_vm: &mut DefaultCosmwasmVM<T>,
		_channel_id: String,
	) -> Result<(), CosmwasmVMError<T>> {
		Err(CosmwasmVMError::<T>::Unsupported)
	}

	pub(crate) fn do_compute_ibc_contract_port(address: AccountIdOf<T>) -> String {
		format!("wasm.{}", Pallet::<T>::account_to_cosmwasm_addr(address))
	}
}

use cosmwasm_vm::system::CosmwasmBaseVM;
pub trait CosmwasmCallVMSingle<I> = CosmwasmBaseVM
where
	I: Input + HasInfo,
	for<'x> Unit: TryFrom<VmOutputOf<'x, Self>, Error = VmErrorOf<Self>>,
	for<'x> VmInputOf<'x, Self>: TryFrom<AllocateCall<PointerOf<Self>>, Error = VmErrorOf<Self>>
		+ TryFrom<DeallocateCall<PointerOf<Self>>, Error = VmErrorOf<Self>>
		+ TryFrom<CosmwasmCallInput<'x, PointerOf<Self>, I>, Error = VmErrorOf<Self>>
		+ TryFrom<CosmwasmCallWithoutInfoInput<'x, PointerOf<Self>, I>, Error = VmErrorOf<Self>>;

pub trait AsEntryName {
	const ENTRY: EntryPoint;
}

impl AsEntryName for IbcChannelOpenCall {
	const ENTRY: EntryPoint = IbcChannelOpen;
}

impl AsEntryName for IbcPacketReceiveCall {
	const ENTRY: EntryPoint = IbcPacketReceive;
}

impl AsEntryName for IbcChannelConnectCall {
	const ENTRY: EntryPoint = IbcChannelConnect;
}

impl AsEntryName for IbcChannelCloseCall {
	const ENTRY: EntryPoint = IbcChannelClose;
}

impl AsEntryName for IbcPacketTimeoutCall {
	const ENTRY: EntryPoint = IbcPacketTimeout;
}

impl AsEntryName for IbcPacketAckCall {
	const ENTRY: EntryPoint = IbcPacketAck;
}

pub struct NoRelayer<T> {
	_marker: sp_std::marker::PhantomData<T>,
}

impl<T: Config> ibc_primitives::IbcHandler<AccountIdOf<T>> for NoRelayer<T> {
	fn latest_height_and_timestamp(
		_port_id: &PortId,
		_channel_id: &ChannelId,
	) -> Result<(Height, Timestamp), ibc_primitives::Error> {
		Err(ibc_primitives::Error::Other { msg: Some("not supported".to_string()) })
	}

	fn handle_message(_msg: HandlerMessage<AccountIdOf<T>>) -> Result<(), ibc_primitives::Error> {
		Err(ibc_primitives::Error::Other { msg: Some("not supported".to_string()) })
	}
}
