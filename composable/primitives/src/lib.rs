#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::string::String;
use codec::{Decode, Encode};
use ibc::{
	apps::transfer::types::PrefixedCoin,
	core::{
		channel::types::{channel::ChannelEnd, packet::Packet},
		client::types::Height,
		host::types::identifiers::{ChannelId, PortId},
	},
	primitives::{Signer, Timestamp},
};
use sp_runtime::RuntimeDebug;

pub mod runtime_interface;

/// Packet timeout, could be an offset, or absolute value.
#[derive(RuntimeDebug, PartialEq, Eq, scale_info::TypeInfo, Encode, Decode, Clone)]
pub enum Timeout {
	Offset {
		/// Timestamp at which this packet should timeout in counterparty in seconds
		/// relative to the latest time stamp
		timestamp: Option<u64>,
		/// Block height at which this packet should timeout on counterparty
		/// relative to the latest height
		height: Option<u64>,
	},
	/// Absolute value
	Absolute {
		/// Timestamp at which this packet should timeout on the counterparty in nanoseconds
		timestamp: Option<u64>,
		/// Block height at which this packet should timeout on the counterparty
		height: Option<u64>,
	},
}

pub enum HandlerMessage<AccountId> {
	OpenChannel {
		port_id: PortId,
		channel_end: ChannelEnd,
	},
	CloseChannel {
		channel_id: ChannelId,
		port_id: PortId,
	},
	Transfer {
		channel_id: ChannelId,
		coin: PrefixedCoin,
		timeout: Timeout,
		from: AccountId,
		to: Signer,
		memo: String,
	},
	SendPacket {
		/// packet data
		data: Vec<u8>,
		/// Packet timeout
		timeout: Timeout,
		/// port id as utf8 string bytes
		port_id: PortId,
		/// channel id as utf8 string bytes
		channel_id: ChannelId,
	},
	WriteAck {
		/// Raw acknowledgement bytes
		ack: Vec<u8>,
		/// Packet
		packet: Packet,
	},
}

#[derive(core::fmt::Debug, Clone, PartialEq, Eq)]
/// Error definition for module
pub enum Error {
	/// Failed to register a new packet
	SendPacketError { msg: Option<String> },
	/// An error involving the connection id
	ConnectionIdError { msg: Option<String> },
	/// An error involving the client id
	ClientIdError { msg: Option<String> },
	/// An error involving channel or port
	ChannelOrPortError { msg: Option<String> },
	/// An error involving Client state
	ClientStateError { msg: Option<String> },
	/// An Error Involving the Timestamp and height
	TimestampOrHeightNotFound { msg: Option<String> },
	/// Failed to register a token transfer packet
	SendTransferError { msg: Option<String> },
	/// Ics20 receive packet processing error
	ReceivePacketError { msg: Option<String> },
	/// Write acknowledgement error
	WriteAcknowledgementError { msg: Option<String> },
	/// Ics20 packet acknowledgement processing error
	AcknowledgementError { msg: Option<String> },
	/// Ics20 packet timeout processing error
	TimeoutError { msg: Option<String> },
	/// Failed to bind port
	BindPortError { msg: Option<String> },
	/// Failed to initialize a new channel
	ChannelInitError { msg: Option<String> },
	/// Failed to close a channel
	ChannelCloseError { msg: Option<String> },
	/// Failed to decode a value
	DecodingError { msg: Option<String> },
	/// Failed to decode commitment prefix
	ErrorDecodingPrefix,
	/// Some other error
	Other { msg: Option<String> },
}

/// Captures the functions modules can use to interact with the ibc pallet
/// Currently allows modules to register packets and create channels
pub trait IbcHandler<AccountId> {
	/// Get the latest height and latest timestamp for the client paired to the channel and port
	/// combination
	fn latest_height_and_timestamp(
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<(Height, Timestamp), Error>;
	/// Handle a message
	fn handle_message(msg: HandlerMessage<AccountId>) -> Result<(), Error>;
	/// testing related methods
	#[cfg(feature = "runtime-benchmarks")]
	fn create_client() -> Result<ClientId, Error>;
	#[cfg(feature = "runtime-benchmarks")]
	fn create_connection(client_id: ClientId, connection_id: ConnectionId) -> Result<(), Error>;
}
