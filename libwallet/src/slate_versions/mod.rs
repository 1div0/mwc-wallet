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

//! This module contains old slate versions and conversions to the newest slate version
//! Used for serialization and deserialization of slates in a backwards compatible way.
//! Versions earlier than V2 are removed for the 2.0.0 release, but versioning code
//! remains for future needs

use crate::slate::Slate;
use crate::slate_versions::v2::{CoinbaseV2, SlateV2};
use crate::slate_versions::v3::{CoinbaseV3, SlateV3};
use crate::slate_versions::v4::{CoinbaseV4, SlateV4};
use crate::types::CbData;

pub mod ser;

#[allow(missing_docs)]
pub mod v2;
#[allow(missing_docs)]
pub mod v3;
#[allow(missing_docs)]
pub mod v4;

/// The most recent version of the slate
pub const CURRENT_SLATE_VERSION: u16 = 4;

/// The grin block header this slate is intended to be compatible with
pub const GRIN_BLOCK_HEADER_VERSION: u16 = 3;

/// Existing versions of the slate
#[derive(EnumIter, Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum SlateVersion {
	/// V4 (most current)
	V4,
	/// V3b (most current) the difference between V3b and V3 is that the way to do payment proof is different
	/// V3b support both mqs public key and dalek public key; V3 only support mqs public key.
	/// they have the same format of slate though.
	V3B,
	/// V3
	V3,
	/// V2 (2.0.0 - Onwards)
	V2,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
/// Versions are ordered newest to oldest so serde attempts to
/// deserialize newer versions first, then falls back to older versions.
pub enum VersionedSlate {
	/// Current (4.0.0 Onwards )
	V4(SlateV4),
	/// V3 (2.0.0 - 3.0.0)
	V3(SlateV3),
	/// V2 (2.0.0 - Onwards)
	V2(SlateV2),
}

impl VersionedSlate {
	/// Return slate version
	pub fn version(&self) -> SlateVersion {
		match *self {
			VersionedSlate::V4(_) => SlateVersion::V4,
			VersionedSlate::V3(_) => SlateVersion::V3,
			VersionedSlate::V2(_) => SlateVersion::V2,
		}
	}

	/// convert this slate type to a specified older version
	pub fn into_version(slate: Slate, version: SlateVersion) -> VersionedSlate {
		match version {
			SlateVersion::V4 => VersionedSlate::V4(slate.into()),
			SlateVersion::V3B | SlateVersion::V3 => {
				let s = SlateV4::from(slate);
				let s = SlateV3::from(&s);
				VersionedSlate::V3(s)
			}
			SlateVersion::V2 => {
				let s = SlateV4::from(slate);
				let s = SlateV3::from(&s);
				let s = SlateV2::from(&s);
				VersionedSlate::V2(s)
			}
		}
	}
}

impl From<VersionedSlate> for Slate {
	fn from(slate: VersionedSlate) -> Slate {
		match slate {
			VersionedSlate::V4(s) => Slate::from(s),
			VersionedSlate::V3(s) => {
				let s = SlateV4::from(s);
				Slate::from(s)
			}
			VersionedSlate::V2(s) => {
				let s = SlateV3::from(s);
				let s = SlateV4::from(s);
				Slate::from(s)
			}
		}
	}
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
/// Versions are ordered newest to oldest so serde attempts to
/// deserialize newer versions first, then falls back to older versions.
pub enum VersionedCoinbase {
	/// Current supported coinbase version.
	V4(CoinbaseV4),
	/// Previous supported coinbase version.
	V3(CoinbaseV3),
	/// Previous
	V2(CoinbaseV2),
}

impl VersionedCoinbase {
	/// convert this coinbase data to a specific versioned representation for the json api.
	pub fn into_version(cb: CbData, version: SlateVersion) -> VersionedCoinbase {
		match version {
			SlateVersion::V4 => VersionedCoinbase::V4(cb.into()),
			SlateVersion::V3B => VersionedCoinbase::V3(cb.into()),
			SlateVersion::V3 => VersionedCoinbase::V3(cb.into()),
			SlateVersion::V2 => VersionedCoinbase::V2(cb.into()),
		}
	}
}
