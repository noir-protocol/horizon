// This file simply re-exports some methods from serde_json
// The reason is two fold:
// 1. To easily ensure that all calling libraries use the same version (minimize code size)
// 2. To allow us to switch out to eg. serde-json-core more easily

use core::any::type_name;
use serde::{de::DeserializeOwned, Serialize};

use crate::{Binary, StdError, StdResult};

#[deprecated = "use from_json instead"]
pub fn from_slice<T: DeserializeOwned>(value: &[u8]) -> StdResult<T> {
	from_json(value)
}

#[deprecated = "use from_json instead"]
pub fn from_binary<T: DeserializeOwned>(value: &Binary) -> StdResult<T> {
	from_json(value)
}

/// Deserializes the given JSON bytes to a data structure.
///
/// Errors if the input is not valid JSON or cannot be deserialized to the given type.
pub fn from_json<T: DeserializeOwned>(value: impl AsRef<[u8]>) -> StdResult<T> {
	serde_json_wasm::from_slice(value.as_ref())
		.map_err(|e| StdError::parse_err(type_name::<T>(), e))
}

#[deprecated = "use to_json_vec instead"]
pub fn to_vec<T>(data: &T) -> StdResult<Vec<u8>>
where
	T: Serialize + ?Sized,
{
	to_json_vec(data)
}

#[deprecated = "use to_json_binary instead"]
pub fn to_binary<T>(data: &T) -> StdResult<Binary>
where
	T: Serialize + ?Sized,
{
	to_json_binary(data)
}

/// Serializes the given data structure as a JSON byte vector.
pub fn to_json_vec<T>(data: &T) -> StdResult<Vec<u8>>
where
	T: Serialize + ?Sized,
{
	serde_json_wasm::to_vec(data).map_err(|e| StdError::serialize_err(type_name::<T>(), e))
}

/// Serializes the given data structure as a JSON string.
pub fn to_json_string<T>(data: &T) -> StdResult<String>
where
	T: Serialize + ?Sized,
{
	serde_json_wasm::to_string(data).map_err(|e| StdError::serialize_err(type_name::<T>(), e))
}

/// Serializes the given data structure as JSON bytes.
pub fn to_json_binary<T>(data: &T) -> StdResult<Binary>
where
	T: Serialize + ?Sized,
{
	to_json_vec(data).map(Binary::new)
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde::Deserialize;

	#[derive(Serialize, Deserialize, Debug, PartialEq)]
	#[serde(rename_all = "snake_case")]
	enum SomeMsg {
		Refund {},
		ReleaseAll { image: String, amount: u32, time: u64, karma: i32 },
		Cowsay { text: String },
	}

	#[test]
	fn to_json_vec_works() {
		let msg = SomeMsg::Refund {};
		let serialized = to_json_vec(&msg).unwrap();
		assert_eq!(serialized, br#"{"refund":{}}"#);

		let msg = SomeMsg::ReleaseAll {
			image: "foo".to_string(),
			amount: 42,
			time: 9007199254740999, // Number.MAX_SAFE_INTEGER + 7
			karma: -17,
		};
		let serialized = String::from_utf8(to_json_vec(&msg).unwrap()).unwrap();
		assert_eq!(
			serialized,
			r#"{"release_all":{"image":"foo","amount":42,"time":9007199254740999,"karma":-17}}"#
		);
	}

	#[test]
	fn from_json_works() {
		let deserialized: SomeMsg = from_json(br#"{"refund":{}}"#).unwrap();
		assert_eq!(deserialized, SomeMsg::Refund {});

		let expected = SomeMsg::ReleaseAll {
			image: "foo".to_string(),
			amount: 42,
			time: 18446744073709551615,
			karma: -17,
		};
		// &[u8]
		let deserialized: SomeMsg = from_json(
            br#"{"release_all":{"image":"foo","amount":42,"time":18446744073709551615,"karma":-17}}"#,
        )
        .unwrap();
		assert_eq!(deserialized, expected);

		// &str
		let deserialized: SomeMsg = from_json(
			r#"{"release_all":{"image":"foo","amount":42,"time":18446744073709551615,"karma":-17}}"#,
		)
		.unwrap();
		assert_eq!(deserialized, expected);
	}

	#[test]
	fn from_json_or_binary() {
		let msg = SomeMsg::Refund {};
		let serialized: Binary = to_json_binary(&msg).unwrap();

		let parse_binary: SomeMsg = from_json(&serialized).unwrap();
		assert_eq!(parse_binary, msg);

		let parse_slice: SomeMsg = from_json(serialized.as_slice()).unwrap();
		assert_eq!(parse_slice, msg);
	}

	#[test]
	fn to_json_vec_works_for_special_chars() {
		let msg = SomeMsg::Cowsay { text: "foo\"bar\\\"bla".to_string() };
		let serialized = String::from_utf8(to_json_vec(&msg).unwrap()).unwrap();
		assert_eq!(serialized, r#"{"cowsay":{"text":"foo\"bar\\\"bla"}}"#);
	}

	#[test]
	fn from_json_works_for_special_chars() {
		let deserialized: SomeMsg = from_json(br#"{"cowsay":{"text":"foo\"bar\\\"bla"}}"#).unwrap();
		assert_eq!(deserialized, SomeMsg::Cowsay { text: "foo\"bar\\\"bla".to_string() });
	}

	#[test]
	fn to_json_string_works() {
		let msg = SomeMsg::Refund {};
		let serialized = to_json_string(&msg).unwrap();
		assert_eq!(serialized, r#"{"refund":{}}"#);

		let msg = SomeMsg::ReleaseAll {
			image: "foo".to_string(),
			amount: 42,
			time: 9007199254740999, // Number.MAX_SAFE_INTEGER + 7
			karma: -17,
		};
		let serialized = to_json_string(&msg).unwrap();
		assert_eq!(
			serialized,
			r#"{"release_all":{"image":"foo","amount":42,"time":9007199254740999,"karma":-17}}"#
		);
	}
}
