use frame_support::weights::Weight;

#[derive(Clone)]
pub struct CosmosError {
	pub weight: Weight,
	pub error: CosmosErrorCode,
}

#[derive(Clone, Copy)]
pub enum CosmosErrorCode {
	// ErrUnauthorized is used whenever a request without sufficient
	// authorization is handled.
	ErrUnauthorized = 4,
	// ErrInsufficientFunds is used when the account cannot pay requested amount.
	ErrInsufficientFunds = 5,
	// ErrOutOfGas to doc
	ErrOutOfGas = 11,
	// ErrInsufficientFee to doc
	ErrInsufficientFee = 13,
	// ErrInvalidType defines an error an invalid type.
	ErrInvalidType = 29,
}
