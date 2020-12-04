// Copyright 2019 The Grin Developers
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

/// File Output 'plugin' implementation
use std::fs::File;
use std::io::{Read, Write};

use crate::error::{Error, ErrorKind};
use crate::libwallet::{Slate, SlateVersion, VersionedSlate};
use crate::{SlateGetter, SlatePutter};
use std::path::PathBuf;

#[derive(Clone)]
pub struct PathToSlate(pub PathBuf);

impl SlatePutter for PathToSlate {
	fn put_tx(&self, slate: &Slate) -> Result<(), Error> {
		let file_name = self.0.to_str().unwrap_or("INVALID PATH");
		let mut pub_tx = File::create(&self.0).map_err(|e| {
			ErrorKind::IO(format!("Unable to create proof file {}, {}", file_name, e))
		})?;
		let out_slate = {
			// slate.lowest_version()
			// Mark for https://github.com/mwcproject/mwc-qt-wallet/issues/663
			if false {
				warn!("Transaction contains features that require grin-wallet 4.0.0 or later");
				warn!("Please ensure the other party is running grin-wallet v4.0.0 or later before sending");
				VersionedSlate::into_version(slate.clone(), SlateVersion::V4)
			} else if slate.payment_proof.is_some() || slate.ttl_cutoff_height.is_some() {
				warn!("Transaction contains features that require mwc-wallet 3.0.0 or later");
				warn!("Please ensure the other party is running mwc-wallet v3.0.0 or later before sending");
				let mut s = slate.clone();
				s.version_info.version = 3;
				s.version_info.orig_version = 3;
				VersionedSlate::into_version(s, SlateVersion::V3)
			} else {
				let mut s = slate.clone();
				s.version_info.version = 2;
				s.version_info.orig_version = 2;
				VersionedSlate::into_version(s, SlateVersion::V2)
			}
		};
		pub_tx
			.write_all(
				serde_json::to_string(&out_slate)
					.map_err(|e| {
						ErrorKind::GenericError(format!("Failed convert Slate to Json, {}", e))
					})?
					.as_bytes(),
			)
			.map_err(|e| {
				ErrorKind::IO(format!(
					"Unable to store data at proof file {}, {}",
					file_name, e
				))
			})?;

		pub_tx.sync_all().map_err(|e| {
			ErrorKind::IO(format!(
				"Unable to store data at proof file {}, {}",
				file_name, e
			))
		})?;

		Ok(())
	}
}

impl SlateGetter for PathToSlate {
	fn get_tx(&self) -> Result<Slate, Error> {
		let file_name = self.0.to_str().unwrap_or("INVALID PATH");
		let mut pub_tx_f = File::open(&self.0).map_err(|e| {
			ErrorKind::IO(format!("Unable to open proof file {}, {}", file_name, e))
		})?;
		let mut content = String::new();
		pub_tx_f.read_to_string(&mut content).map_err(|e| {
			ErrorKind::IO(format!(
				"Unable to read data from file {}, {}",
				file_name, e
			))
		})?;

		Ok(Slate::deserialize_upgrade(&content).map_err(|e| {
			ErrorKind::IO(format!(
				"Unable to build slate from json, file {}, {}",
				file_name, e
			))
		})?)
	}
}
