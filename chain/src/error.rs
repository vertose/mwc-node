// Copyright 2020 The Grin Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Error types for chain
use crate::core::core::hash::Hash;
use crate::core::core::{block, committed, transaction};
use crate::core::ser;
use crate::keychain;
use crate::util::secp;
use failure::{Backtrace, Context, Fail};
use grin_store as store;
use std::fmt::{self, Display};
use std::io;

/// Error definition
#[derive(Debug, Fail)]
pub struct Error {
	inner: Context<ErrorKind>,
}

/// Chain error definitions
#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
	/// The block doesn't fit anywhere in our chain
	#[fail(display = "Block is unfit: {}", _0)]
	Unfit(String),
	/// Special case of orphan blocks
	#[fail(display = "Orphan, {}", _0)]
	Orphan(String),
	/// Difficulty is too low either compared to ours or the block PoW hash
	#[fail(display = "Difficulty is too low compared to ours or the block PoW hash")]
	DifficultyTooLow,
	/// Addition of difficulties on all previous block is wrong
	#[fail(display = "Addition of difficulties on all previous blocks is wrong")]
	WrongTotalDifficulty,
	/// Block header edge_bits is lower than our min
	#[fail(display = "Cuckoo Size too small")]
	LowEdgebits,
	/// Block header invalid hash, explicitly rejected
	#[fail(display = "Block hash explicitly rejected by chain")]
	InvalidHash,
	/// Scaling factor between primary and secondary PoW is invalid
	#[fail(display = "Wrong scaling factor")]
	InvalidScaling,
	/// The proof of work is invalid
	#[fail(display = "Invalid PoW")]
	InvalidPow,
	/// Peer abusively sending us an old block we already have
	#[fail(display = "Old Block")]
	OldBlock,
	/// The block doesn't sum correctly or a tx signature is invalid
	#[fail(display = "Invalid Block Proof, {}", _0)]
	InvalidBlockProof(block::Error),
	/// Block time is too old
	#[fail(display = "Invalid Block Time")]
	InvalidBlockTime,
	/// Block height is invalid (not previous + 1)
	#[fail(display = "Invalid Block Height")]
	InvalidBlockHeight,
	/// One of the root hashes in the block is invalid
	#[fail(display = "Invalid Root, {}", _0)]
	InvalidRoot(String),
	/// One of the MMR sizes in the block header is invalid
	#[fail(display = "Invalid MMR Size")]
	InvalidMMRSize,
	/// Error from underlying keychain impl
	#[fail(display = "Keychain Error, {}", _0)]
	Keychain(keychain::Error),
	/// Error from underlying secp lib
	#[fail(display = "Secp Lib Error, {}", _0)]
	Secp(secp::Error),
	/// One of the inputs in the block has already been spent
	#[fail(display = "Already Spent: {:?}", _0)]
	AlreadySpent(Hash),
	/// An output with that ID already exists (should be unique)
	#[fail(display = "Duplicate Output ID: {:?}", _0)]
	DuplicateOutputId(Hash),
	/// Attempt to spend a coinbase output before it sufficiently matures.
	#[fail(display = "Attempt to spend immature coinbase")]
	ImmatureCoinbase,
	/// Error validating a Merkle proof (coinbase output)
	#[fail(display = "Error validating merkle proof, {}", _0)]
	MerkleProof(String),
	/// Output not found
	#[fail(display = "Output not found, {}", _0)]
	OutputNotFound(String),
	/// Rangeproof not found
	#[fail(display = "Rangeproof not found, {}", _0)]
	RangeproofNotFound(String),
	/// Tx kernel not found
	#[fail(display = "Tx kernel not found")]
	TxKernelNotFound,
	/// output spent
	#[fail(display = "Output is spent")]
	OutputSpent,
	/// Invalid block version, either a mistake or outdated software
	#[fail(display = "Invalid Block Version: {:?}", _0)]
	InvalidBlockVersion(block::HeaderVersion),
	/// We've been provided a bad txhashset
	#[fail(display = "Invalid TxHashSet: {}", _0)]
	InvalidTxHashSet(String),
	/// Internal issue when trying to save or load data from store
	#[fail(display = "Chain Store Error: {}, reason: {}", _1, _0)]
	StoreErr(store::Error, String),
	/// Internal issue when trying to save or load data from append only files
	#[fail(display = "Chain File Read Error: {}", _0)]
	FileReadErr(String),
	/// Error serializing or deserializing a type
	#[fail(display = "Chain Serialization Error, {}", _0)]
	SerErr(ser::Error),
	/// Error with the txhashset
	#[fail(display = "TxHashSetErr: {}", _0)]
	TxHashSetErr(String),
	/// Tx not valid based on lock_height.
	#[fail(display = "Invalid Transaction Lock Height")]
	TxLockHeight,
	/// Tx is not valid due to NRD relative_height restriction.
	#[fail(display = "NRD Relative Height")]
	NRDRelativeHeight,
	/// No chain exists and genesis block is required
	#[fail(display = "Genesis Block Required")]
	GenesisBlockRequired,
	/// Error from underlying tx handling
	#[fail(display = "Transaction Validation Error: {:?}", _0)]
	Transaction(transaction::Error),
	/// Error from underlying block handling
	#[fail(display = "Block Validation Error: {:?}", _0)]
	Block(block::Error),
	/// Anything else
	#[fail(display = "Chain other Error: {}", _0)]
	Other(String),
	/// Error from summing and verifying kernel sums via committed trait.
	#[fail(
		display = "Committed Trait: Error summing and verifying kernel sums, {}",
		_0
	)]
	Committed(committed::Error),
	/// We cannot process data once the Grin server has been stopped.
	#[fail(display = "Stopped (MWC Shutting Down)")]
	Stopped,
	/// Internal Roaring Bitmap error
	#[fail(display = "Roaring Bitmap error")]
	Bitmap,
	/// Error during chain sync
	#[fail(display = "Sync error")]
	SyncError(String),
}

impl Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let cause = match self.cause() {
			Some(c) => format!("{}", c),
			None => String::from("Unknown"),
		};
		let backtrace = match self.backtrace() {
			Some(b) => format!("{}", b),
			None => String::from("Unknown"),
		};
		let output = format!(
			"{} \n Cause: {} \n Backtrace: {}",
			self.inner, cause, backtrace
		);
		Display::fmt(&output, f)
	}
}

impl Error {
	/// get kind
	pub fn kind(&self) -> ErrorKind {
		self.inner.get_context().clone()
	}
	/// get cause
	pub fn cause(&self) -> Option<&dyn Fail> {
		self.inner.cause()
	}
	/// get backtrace
	pub fn backtrace(&self) -> Option<&Backtrace> {
		self.inner.backtrace()
	}

	/// Whether the error is due to a block that was intrinsically wrong
	pub fn is_bad_data(&self) -> bool {
		// shorter to match on all the "not the block's fault" errors
		match self.kind() {
			ErrorKind::Unfit(_)
			| ErrorKind::Orphan(_)
			| ErrorKind::StoreErr(_, _)
			| ErrorKind::SerErr(_)
			| ErrorKind::TxHashSetErr(_)
			| ErrorKind::GenesisBlockRequired
			| ErrorKind::Other(_) => false,
			_ => true,
		}
	}
}

impl From<ErrorKind> for Error {
	fn from(kind: ErrorKind) -> Error {
		Error {
			inner: Context::new(kind),
		}
	}
}

impl From<Context<ErrorKind>> for Error {
	fn from(inner: Context<ErrorKind>) -> Error {
		Error { inner: inner }
	}
}

impl From<block::Error> for Error {
	fn from(error: block::Error) -> Error {
		let ec = error.clone();
		Error {
			inner: error.context(ErrorKind::InvalidBlockProof(ec)),
		}
	}
}

impl From<store::Error> for Error {
	fn from(error: store::Error) -> Error {
		let ec = error.clone();
		Error {
			//inner: error.context();Context::new(ErrorKind::StoreErr(error.clone(),
			// format!("{:?}", error))),
			inner: error.context(ErrorKind::StoreErr(ec.clone(), format!("{:?}", ec))),
		}
	}
}

impl From<keychain::Error> for Error {
	fn from(error: keychain::Error) -> Error {
		Error {
			inner: Context::new(ErrorKind::Keychain(error)),
		}
	}
}

impl From<transaction::Error> for Error {
	fn from(error: transaction::Error) -> Error {
		Error {
			inner: Context::new(ErrorKind::Transaction(error)),
		}
	}
}

impl From<committed::Error> for Error {
	fn from(error: committed::Error) -> Error {
		Error {
			inner: Context::new(ErrorKind::Committed(error)),
		}
	}
}

impl From<io::Error> for Error {
	fn from(e: io::Error) -> Error {
		Error {
			inner: Context::new(ErrorKind::TxHashSetErr(e.to_string())),
		}
	}
}

impl From<ser::Error> for Error {
	fn from(error: ser::Error) -> Error {
		Error {
			inner: Context::new(ErrorKind::SerErr(error)),
		}
	}
}

impl From<secp::Error> for Error {
	fn from(e: secp::Error) -> Error {
		Error {
			inner: Context::new(ErrorKind::Secp(e)),
		}
	}
}
