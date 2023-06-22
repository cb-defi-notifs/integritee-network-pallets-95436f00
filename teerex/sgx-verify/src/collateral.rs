/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG

	Licensed under the MICROSOFT REFERENCE SOURCE LICENSE (MS-RSL) (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		https://referencesource.microsoft.com/license.html

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.

*/

#![cfg_attr(not(feature = "std"), no_std)]
pub extern crate alloc;

use alloc::string::String;
use chrono::prelude::{DateTime, Utc};
use log;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use sp_std::prelude::*;
use teerex_primitives::{
	Fmspc, MrSigner, Pcesvn, QeTcb, QuotingEnclave, TcbInfoOnChain, TcbVersionStatus,
};

/// The data structures in here are designed such that they can be used to serialize/deserialize
/// the "TCB info" and "enclave identity" collateral data in JSON format provided by intel
/// See https://api.portal.trustedservices.intel.com/documentation for further information and examples

#[derive(Serialize, Deserialize)]
pub struct Tcb {
	isvsvn: u16,
}

impl Tcb {
	pub fn is_valid(&self) -> bool {
		// At the time of writing this code everything older than 6 is outdated
		// Intel does the same check in their DCAP implementation
		self.isvsvn >= 6
	}
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcbLevel {
	tcb: Tcb,
	/// Intel does not verify the tcb_date in their code and their API documentation also does
	/// not mention it needs verification.
	tcb_date: DateTime<Utc>,
	tcb_status: String,
	#[serde(rename = "advisoryIDs")]
	#[serde(skip_serializing_if = "Option::is_none")]
	advisory_ids: Option<Vec<String>>,
}

impl TcbLevel {
	pub fn is_valid(&self) -> bool {
		// UpToDate is the only valid status (the other being OutOfDate and Revoked)
		// A possible extension would be to also verify that the advisory_ids list is empty,
		// but I think this could also lead to all TcbLevels being invalid
		self.tcb.is_valid() && self.tcb_status == "UpToDate"
	}
}

#[derive(Serialize, Deserialize, Debug)]
struct TcbComponentV2(u8);
#[derive(Serialize, Deserialize, Debug)]
struct TcbComponentV3 {
	svn: u8,
	#[serde(skip_serializing_if = "Option::is_none")]
	category: Option<String>,
	#[serde(rename = "type")] //type is a keyword so we rename the field
	#[serde(skip_serializing_if = "Option::is_none")]
	tcb_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TcbFullV2 {
	sgxtcbcomp01svn: TcbComponentV2,
	sgxtcbcomp02svn: TcbComponentV2,
	sgxtcbcomp03svn: TcbComponentV2,
	sgxtcbcomp04svn: TcbComponentV2,
	sgxtcbcomp05svn: TcbComponentV2,
	sgxtcbcomp06svn: TcbComponentV2,
	sgxtcbcomp07svn: TcbComponentV2,
	sgxtcbcomp08svn: TcbComponentV2,
	sgxtcbcomp09svn: TcbComponentV2,
	sgxtcbcomp10svn: TcbComponentV2,
	sgxtcbcomp11svn: TcbComponentV2,
	sgxtcbcomp12svn: TcbComponentV2,
	sgxtcbcomp13svn: TcbComponentV2,
	sgxtcbcomp14svn: TcbComponentV2,
	sgxtcbcomp15svn: TcbComponentV2,
	sgxtcbcomp16svn: TcbComponentV2,
	pcesvn: Pcesvn,
}
impl TcbFullV2 {
	pub fn get_sgx_tcb_comp_svn_as_slice(&self) -> [u8; 16] {
		[
			self.sgxtcbcomp01svn.0,
			self.sgxtcbcomp02svn.0,
			self.sgxtcbcomp03svn.0,
			self.sgxtcbcomp04svn.0,
			self.sgxtcbcomp05svn.0,
			self.sgxtcbcomp06svn.0,
			self.sgxtcbcomp07svn.0,
			self.sgxtcbcomp08svn.0,
			self.sgxtcbcomp09svn.0,
			self.sgxtcbcomp10svn.0,
			self.sgxtcbcomp11svn.0,
			self.sgxtcbcomp12svn.0,
			self.sgxtcbcomp13svn.0,
			self.sgxtcbcomp14svn.0,
			self.sgxtcbcomp15svn.0,
			self.sgxtcbcomp16svn.0,
		]
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TcbFullV3 {
	sgxtcbcomponents: [TcbComponentV3; 16],
	pcesvn: Pcesvn,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TcbLevelFullV2 {
	tcb: TcbFullV2,
	/// Intel does not verify the tcb_date in their code and their API documentation also does
	/// not mention it needs verification.
	tcb_date: DateTime<Utc>,
	tcb_status: String,
	#[serde(rename = "advisoryIDs")]
	#[serde(skip_serializing_if = "Option::is_none")]
	advisory_ids: Option<Vec<String>>,
}

impl TcbLevelFullV2 {
	pub fn is_valid(&self) -> bool {
		// A possible extension would be to also verify that the advisory_ids list is empty,
		// but I think this could also lead to all TcbLevels being invalid
		self.tcb_status == "UpToDate" || self.tcb_status == "SWHardeningNeeded"
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TcbLevelFullV3 {
	tcb: TcbFullV3,
	/// Intel does not verify the tcb_date in their code and their API documentation also does
	/// not mention it needs verification.
	tcb_date: DateTime<Utc>,
	tcb_status: String,
	#[serde(rename = "advisoryIDs")]
	#[serde(skip_serializing_if = "Option::is_none")]
	advisory_ids: Option<Vec<String>>,
}

impl TcbLevelFullV3 {
	pub fn is_valid(&self) -> bool {
		// A possible extension would be to also verify that the advisory_ids list is empty,
		// but I think this could also lead to all TcbLevels being invalid
		self.tcb_status == "UpToDate" || self.tcb_status == "SWHardeningNeeded"
	}
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnclaveIdentity {
	id: String,
	version: u16,
	issue_date: DateTime<Utc>,
	next_update: DateTime<Utc>,
	tcb_evaluation_data_number: u16,
	#[serde(deserialize_with = "deserialize_from_hex::<_, 4>")]
	#[serde(serialize_with = "serialize_to_hex::<_, 4>")]
	miscselect: [u8; 4],
	#[serde(deserialize_with = "deserialize_from_hex::<_, 4>")]
	#[serde(serialize_with = "serialize_to_hex::<_, 4>")]
	miscselect_mask: [u8; 4],
	#[serde(deserialize_with = "deserialize_from_hex::<_, 16>")]
	#[serde(serialize_with = "serialize_to_hex::<_, 16>")]
	attributes: [u8; 16],
	#[serde(deserialize_with = "deserialize_from_hex::<_, 16>")]
	#[serde(serialize_with = "serialize_to_hex::<_, 16>")]
	attributes_mask: [u8; 16],
	#[serde(deserialize_with = "deserialize_from_hex::<_, 32>")]
	#[serde(serialize_with = "serialize_to_hex::<_, 32>")]
	mrsigner: MrSigner,
	pub isvprodid: u16,
	pub tcb_levels: Vec<TcbLevel>,
}

fn serialize_to_hex<S, const N: usize>(x: &[u8; N], s: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	s.serialize_str(&hex::encode(x).to_uppercase())
}

fn deserialize_from_hex<'de, D, const N: usize>(deserializer: D) -> Result<[u8; N], D::Error>
where
	D: Deserializer<'de>,
{
	let s: &str = Deserialize::deserialize(deserializer)?;
	let hex = hex::decode(s).map_err(|_| D::Error::custom("Failed to deserialize hex string"))?;
	hex.try_into().map_err(|_| D::Error::custom("Invalid hex length"))
}

impl EnclaveIdentity {
	/// This extracts the necessary information into the struct that we actually store in the chain
	pub fn to_quoting_enclave(&self) -> QuotingEnclave {
		let mut valid_tcbs: Vec<QeTcb> = Vec::new();
		for tcb in &self.tcb_levels {
			if tcb.is_valid() {
				valid_tcbs.push(QeTcb::new(tcb.tcb.isvsvn));
			}
		}
		QuotingEnclave::new(
			self.issue_date
				.timestamp_millis()
				.try_into()
				.expect("no support for negative unix timestamps"),
			self.next_update
				.timestamp_millis()
				.try_into()
				.expect("no support for negative unix timestamps"),
			self.miscselect,
			self.miscselect_mask,
			self.attributes,
			self.attributes_mask,
			self.mrsigner,
			self.isvprodid,
			valid_tcbs,
		)
	}

	pub fn is_valid(&self, timestamp_millis: i64) -> bool {
		self.id == "QE" &&
			self.version == 2 &&
			self.issue_date.timestamp_millis() < timestamp_millis &&
			timestamp_millis < self.next_update.timestamp_millis()
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TcbInfo {
	V2(TcbInfoV2),
	V3(TcbInfoV3),
}

impl TcbInfo {
	/// This extracts the necessary information into a tuple (`(Key, Value)`) that we actually store in the chain
	pub fn to_chain_tcb_info(&self) -> (Fmspc, TcbInfoOnChain) {
		match &self {
			TcbInfo::V3(v3) => v3.to_chain_tcb_info(),
			TcbInfo::V2(v2) => v2.to_chain_tcb_info(),
		}
	}

	pub fn is_valid(&self, timestamp_millis: i64) -> bool {
		match &self {
			TcbInfo::V3(v3) => v3.is_valid(timestamp_millis),
			TcbInfo::V2(v2) => v2.is_valid(timestamp_millis),
		}
	}

	pub fn from_byte_slice(slice: &[u8]) -> Option<TcbInfo> {
		if let Some(v2) = TcbInfoV2::from_byte_slice(slice) {
			Some(TcbInfo::V2(v2))
		} else if let Some(v3) = TcbInfoV3::from_byte_slice(slice) {
			Some(TcbInfo::V3(v3))
		} else {
			None
		}
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TcbInfoV2 {
	version: u8,
	issue_date: DateTime<Utc>,
	next_update: DateTime<Utc>,
	#[serde(deserialize_with = "deserialize_from_hex::<_, 6>")]
	#[serde(serialize_with = "serialize_to_hex::<_, 6>")]
	pub fmspc: teerex_primitives::Fmspc,
	pce_id: String,
	tcb_type: u16,
	tcb_evaluation_data_number: u16,
	tcb_levels: Vec<TcbLevelFullV2>,
}

impl TcbInfoV2 {
	pub fn to_chain_tcb_info(&self) -> (Fmspc, TcbInfoOnChain) {
		let valid_tcbs: Vec<TcbVersionStatus> = self
			.tcb_levels
			.iter()
			// Only store TCB levels on chain that are currently valid
			.filter(|tcb| tcb.is_valid())
			.map(|tcb| {
				let components = tcb.tcb.get_sgx_tcb_comp_svn_as_slice();
				TcbVersionStatus::new(components, tcb.tcb.pcesvn)
			})
			.collect();
		(
			self.fmspc,
			TcbInfoOnChain::new(
				self.issue_date
					.timestamp_millis()
					.try_into()
					.expect("no support for negative unix timestamps"),
				self.next_update
					.timestamp_millis()
					.try_into()
					.expect("no support for negative unix timestamps"),
				valid_tcbs,
			),
		)
	}
	pub fn is_valid(&self, timestamp_millis: i64) -> bool {
		log::info!("teerex: called into runtime call v2::is_valid().");
		log::info!(
			"teerex: called into runtime call v2::is_valid(), timestamp_millis: {:#?}",
			timestamp_millis
		);
		log::info!("teerex: called into runtime call v2::is_valid(), self: {:#?}", self);
		self.version == 2 &&
			self.issue_date.timestamp_millis() < timestamp_millis &&
			timestamp_millis < self.next_update.timestamp_millis()
	}

	pub fn from_byte_slice(slice: &[u8]) -> Option<TcbInfoV2> {
		if let Ok(v2) = serde_json::from_slice::<TcbInfoV2>(slice) {
			return Some(v2)
		}
		None
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TcbInfoV3 {
	id: String,
	version: u8,
	issue_date: DateTime<Utc>,
	next_update: DateTime<Utc>,
	#[serde(deserialize_with = "deserialize_from_hex::<_, 6>")]
	#[serde(serialize_with = "serialize_to_hex::<_, 6>")]
	pub fmspc: teerex_primitives::Fmspc,
	pce_id: String,
	tcb_type: u16,
	tcb_evaluation_data_number: u16,
	tcb_levels: Vec<TcbLevelFullV3>,
}

impl TcbInfoV3 {
	pub fn to_chain_tcb_info(&self) -> (Fmspc, TcbInfoOnChain) {
		let valid_tcbs: Vec<TcbVersionStatus> = self
			.tcb_levels
			.iter()
			// Only store TCB levels on chain that are currently valid
			.filter(|tcb| tcb.is_valid())
			.map(|tcb| {
				let mut components = [0u8; 16];
				for (i, t) in tcb.tcb.sgxtcbcomponents.iter().enumerate() {
					components[i] = t.svn;
				}
				TcbVersionStatus::new(components, tcb.tcb.pcesvn)
			})
			.collect();
		(
			self.fmspc,
			TcbInfoOnChain::new(
				self.issue_date
					.timestamp_millis()
					.try_into()
					.expect("no support for negative unix timestamps"),
				self.next_update
					.timestamp_millis()
					.try_into()
					.expect("no support for negative unix timestamps"),
				valid_tcbs,
			),
		)
	}
	pub fn is_valid(&self, timestamp_millis: i64) -> bool {
		self.id == "SGX" &&
			self.version == 3 &&
			self.issue_date.timestamp_millis() < timestamp_millis &&
			timestamp_millis < self.next_update.timestamp_millis()
	}
	pub fn from_byte_slice(slice: &[u8]) -> Option<TcbInfoV3> {
		let res: Option<TcbInfoV3> = serde_json::from_slice(slice).unwrap_or_default();
		res
	}
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcbInfoSigned {
	pub tcb_info: TcbInfoV3,
	pub signature: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnclaveIdentitySigned {
	pub enclave_identity: EnclaveIdentity,
	pub signature: String,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn tcb_level_is_valid() {
		let t: TcbLevel = serde_json::from_str(
			r#"{"tcb":{"isvsvn":6}, "tcbDate":"2021-11-10T00:00:00Z", "tcbStatus":"UpToDate" }"#,
		)
		.unwrap();
		assert!(t.is_valid());

		let t: TcbLevel = serde_json::from_str(
			r#"{"tcb":{"isvsvn":6}, "tcbDate":"2021-11-10T00:00:00Z", "tcbStatus":"OutOfDate" }"#,
		)
		.unwrap();
		assert!(!t.is_valid());

		let t: TcbLevel = serde_json::from_str(
			r#"{"tcb":{"isvsvn":5}, "tcbDate":"2021-11-10T00:00:00Z", "tcbStatus":"UpToDate" }"#,
		)
		.unwrap();
		assert!(!t.is_valid());
	}
	#[test]
	fn parse_tcb_info_v2() {
		let byte_slice: &[u8] = &[
			123, 34, 118, 101, 114, 115, 105, 111, 110, 34, 58, 50, 44, 34, 105, 115, 115, 117,
			101, 68, 97, 116, 101, 34, 58, 34, 50, 48, 50, 51, 45, 48, 54, 45, 50, 48, 84, 49, 49,
			58, 48, 50, 58, 49, 56, 90, 34, 44, 34, 110, 101, 120, 116, 85, 112, 100, 97, 116, 101,
			34, 58, 34, 50, 48, 50, 51, 45, 48, 55, 45, 50, 48, 84, 49, 49, 58, 48, 50, 58, 49, 56,
			90, 34, 44, 34, 102, 109, 115, 112, 99, 34, 58, 34, 48, 48, 65, 48, 54, 53, 53, 49, 48,
			48, 48, 48, 34, 44, 34, 112, 99, 101, 73, 100, 34, 58, 34, 48, 48, 48, 48, 34, 44, 34,
			116, 99, 98, 84, 121, 112, 101, 34, 58, 48, 44, 34, 116, 99, 98, 69, 118, 97, 108, 117,
			97, 116, 105, 111, 110, 68, 97, 116, 97, 78, 117, 109, 98, 101, 114, 34, 58, 49, 53,
			44, 34, 116, 99, 98, 76, 101, 118, 101, 108, 115, 34, 58, 91, 123, 34, 116, 99, 98, 34,
			58, 123, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 49, 115, 118, 110, 34,
			58, 49, 52, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 50, 115, 118,
			110, 34, 58, 49, 52, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 51,
			115, 118, 110, 34, 58, 50, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48,
			52, 115, 118, 110, 34, 58, 50, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112,
			48, 53, 115, 118, 110, 34, 58, 50, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109,
			112, 48, 54, 115, 118, 110, 34, 58, 49, 50, 56, 44, 34, 115, 103, 120, 116, 99, 98, 99,
			111, 109, 112, 48, 55, 115, 118, 110, 34, 58, 49, 50, 44, 34, 115, 103, 120, 116, 99,
			98, 99, 111, 109, 112, 48, 56, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116,
			99, 98, 99, 111, 109, 112, 48, 57, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120,
			116, 99, 98, 99, 111, 109, 112, 49, 48, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103,
			120, 116, 99, 98, 99, 111, 109, 112, 49, 49, 115, 118, 110, 34, 58, 48, 44, 34, 115,
			103, 120, 116, 99, 98, 99, 111, 109, 112, 49, 50, 115, 118, 110, 34, 58, 48, 44, 34,
			115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 49, 51, 115, 118, 110, 34, 58, 48, 44,
			34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 49, 52, 115, 118, 110, 34, 58, 48,
			44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 49, 53, 115, 118, 110, 34, 58,
			48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 49, 54, 115, 118, 110, 34,
			58, 48, 44, 34, 112, 99, 101, 115, 118, 110, 34, 58, 49, 51, 125, 44, 34, 116, 99, 98,
			68, 97, 116, 101, 34, 58, 34, 50, 48, 50, 51, 45, 48, 50, 45, 49, 53, 84, 48, 48, 58,
			48, 48, 58, 48, 48, 90, 34, 44, 34, 116, 99, 98, 83, 116, 97, 116, 117, 115, 34, 58,
			34, 83, 87, 72, 97, 114, 100, 101, 110, 105, 110, 103, 78, 101, 101, 100, 101, 100, 34,
			125, 44, 123, 34, 116, 99, 98, 34, 58, 123, 34, 115, 103, 120, 116, 99, 98, 99, 111,
			109, 112, 48, 49, 115, 118, 110, 34, 58, 49, 52, 44, 34, 115, 103, 120, 116, 99, 98,
			99, 111, 109, 112, 48, 50, 115, 118, 110, 34, 58, 49, 52, 44, 34, 115, 103, 120, 116,
			99, 98, 99, 111, 109, 112, 48, 51, 115, 118, 110, 34, 58, 50, 44, 34, 115, 103, 120,
			116, 99, 98, 99, 111, 109, 112, 48, 52, 115, 118, 110, 34, 58, 50, 44, 34, 115, 103,
			120, 116, 99, 98, 99, 111, 109, 112, 48, 53, 115, 118, 110, 34, 58, 50, 44, 34, 115,
			103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 54, 115, 118, 110, 34, 58, 49, 50, 56,
			44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 55, 115, 118, 110, 34, 58,
			48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 56, 115, 118, 110, 34,
			58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 57, 115, 118, 110,
			34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 49, 48, 115, 118,
			110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 49, 49, 115,
			118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 49, 50,
			115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 49,
			51, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112,
			49, 52, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109,
			112, 49, 53, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111,
			109, 112, 49, 54, 115, 118, 110, 34, 58, 48, 44, 34, 112, 99, 101, 115, 118, 110, 34,
			58, 49, 51, 125, 44, 34, 116, 99, 98, 68, 97, 116, 101, 34, 58, 34, 50, 48, 50, 51, 45,
			48, 50, 45, 49, 53, 84, 48, 48, 58, 48, 48, 58, 48, 48, 90, 34, 44, 34, 116, 99, 98,
			83, 116, 97, 116, 117, 115, 34, 58, 34, 67, 111, 110, 102, 105, 103, 117, 114, 97, 116,
			105, 111, 110, 65, 110, 100, 83, 87, 72, 97, 114, 100, 101, 110, 105, 110, 103, 78,
			101, 101, 100, 101, 100, 34, 125, 44, 123, 34, 116, 99, 98, 34, 58, 123, 34, 115, 103,
			120, 116, 99, 98, 99, 111, 109, 112, 48, 49, 115, 118, 110, 34, 58, 49, 49, 44, 34,
			115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 50, 115, 118, 110, 34, 58, 49, 49,
			44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 51, 115, 118, 110, 34, 58,
			50, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 52, 115, 118, 110, 34,
			58, 50, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 53, 115, 118, 110,
			34, 58, 50, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 54, 115, 118,
			110, 34, 58, 49, 50, 56, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 55,
			115, 118, 110, 34, 58, 52, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48,
			56, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112,
			48, 57, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109,
			112, 49, 48, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111,
			109, 112, 49, 49, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99,
			111, 109, 112, 49, 50, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98,
			99, 111, 109, 112, 49, 51, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99,
			98, 99, 111, 109, 112, 49, 52, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116,
			99, 98, 99, 111, 109, 112, 49, 53, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120,
			116, 99, 98, 99, 111, 109, 112, 49, 54, 115, 118, 110, 34, 58, 48, 44, 34, 112, 99,
			101, 115, 118, 110, 34, 58, 49, 49, 125, 44, 34, 116, 99, 98, 68, 97, 116, 101, 34, 58,
			34, 50, 48, 50, 49, 45, 49, 49, 45, 49, 48, 84, 48, 48, 58, 48, 48, 58, 48, 48, 90, 34,
			44, 34, 116, 99, 98, 83, 116, 97, 116, 117, 115, 34, 58, 34, 79, 117, 116, 79, 102, 68,
			97, 116, 101, 34, 125, 44, 123, 34, 116, 99, 98, 34, 58, 123, 34, 115, 103, 120, 116,
			99, 98, 99, 111, 109, 112, 48, 49, 115, 118, 110, 34, 58, 49, 49, 44, 34, 115, 103,
			120, 116, 99, 98, 99, 111, 109, 112, 48, 50, 115, 118, 110, 34, 58, 49, 49, 44, 34,
			115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 51, 115, 118, 110, 34, 58, 50, 44,
			34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 52, 115, 118, 110, 34, 58, 50,
			44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 53, 115, 118, 110, 34, 58,
			50, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 54, 115, 118, 110, 34,
			58, 49, 50, 56, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 55, 115,
			118, 110, 34, 58, 52, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 56,
			115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48,
			57, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112,
			49, 48, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109,
			112, 49, 49, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111,
			109, 112, 49, 50, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99,
			111, 109, 112, 49, 51, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98,
			99, 111, 109, 112, 49, 52, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99,
			98, 99, 111, 109, 112, 49, 53, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116,
			99, 98, 99, 111, 109, 112, 49, 54, 115, 118, 110, 34, 58, 48, 44, 34, 112, 99, 101,
			115, 118, 110, 34, 58, 49, 48, 125, 44, 34, 116, 99, 98, 68, 97, 116, 101, 34, 58, 34,
			50, 48, 50, 48, 45, 49, 49, 45, 49, 49, 84, 48, 48, 58, 48, 48, 58, 48, 48, 90, 34, 44,
			34, 116, 99, 98, 83, 116, 97, 116, 117, 115, 34, 58, 34, 79, 117, 116, 79, 102, 68, 97,
			116, 101, 34, 125, 44, 123, 34, 116, 99, 98, 34, 58, 123, 34, 115, 103, 120, 116, 99,
			98, 99, 111, 109, 112, 48, 49, 115, 118, 110, 34, 58, 49, 49, 44, 34, 115, 103, 120,
			116, 99, 98, 99, 111, 109, 112, 48, 50, 115, 118, 110, 34, 58, 49, 49, 44, 34, 115,
			103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 51, 115, 118, 110, 34, 58, 50, 44, 34,
			115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 52, 115, 118, 110, 34, 58, 50, 44,
			34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 53, 115, 118, 110, 34, 58, 50,
			44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 54, 115, 118, 110, 34, 58,
			49, 50, 56, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 55, 115, 118,
			110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 56, 115,
			118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 57,
			115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 49,
			48, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112,
			49, 49, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109,
			112, 49, 50, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111,
			109, 112, 49, 51, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99,
			111, 109, 112, 49, 52, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98,
			99, 111, 109, 112, 49, 53, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99,
			98, 99, 111, 109, 112, 49, 54, 115, 118, 110, 34, 58, 48, 44, 34, 112, 99, 101, 115,
			118, 110, 34, 58, 49, 49, 125, 44, 34, 116, 99, 98, 68, 97, 116, 101, 34, 58, 34, 50,
			48, 50, 49, 45, 49, 49, 45, 49, 48, 84, 48, 48, 58, 48, 48, 58, 48, 48, 90, 34, 44, 34,
			116, 99, 98, 83, 116, 97, 116, 117, 115, 34, 58, 34, 79, 117, 116, 79, 102, 68, 97,
			116, 101, 67, 111, 110, 102, 105, 103, 117, 114, 97, 116, 105, 111, 110, 78, 101, 101,
			100, 101, 100, 34, 125, 44, 123, 34, 116, 99, 98, 34, 58, 123, 34, 115, 103, 120, 116,
			99, 98, 99, 111, 109, 112, 48, 49, 115, 118, 110, 34, 58, 49, 49, 44, 34, 115, 103,
			120, 116, 99, 98, 99, 111, 109, 112, 48, 50, 115, 118, 110, 34, 58, 49, 49, 44, 34,
			115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 51, 115, 118, 110, 34, 58, 50, 44,
			34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 52, 115, 118, 110, 34, 58, 50,
			44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 53, 115, 118, 110, 34, 58,
			50, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 54, 115, 118, 110, 34,
			58, 49, 50, 56, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 55, 115,
			118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 56,
			115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48,
			57, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112,
			49, 48, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109,
			112, 49, 49, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111,
			109, 112, 49, 50, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99,
			111, 109, 112, 49, 51, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98,
			99, 111, 109, 112, 49, 52, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99,
			98, 99, 111, 109, 112, 49, 53, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116,
			99, 98, 99, 111, 109, 112, 49, 54, 115, 118, 110, 34, 58, 48, 44, 34, 112, 99, 101,
			115, 118, 110, 34, 58, 49, 48, 125, 44, 34, 116, 99, 98, 68, 97, 116, 101, 34, 58, 34,
			50, 48, 50, 48, 45, 49, 49, 45, 49, 49, 84, 48, 48, 58, 48, 48, 58, 48, 48, 90, 34, 44,
			34, 116, 99, 98, 83, 116, 97, 116, 117, 115, 34, 58, 34, 79, 117, 116, 79, 102, 68, 97,
			116, 101, 67, 111, 110, 102, 105, 103, 117, 114, 97, 116, 105, 111, 110, 78, 101, 101,
			100, 101, 100, 34, 125, 44, 123, 34, 116, 99, 98, 34, 58, 123, 34, 115, 103, 120, 116,
			99, 98, 99, 111, 109, 112, 48, 49, 115, 118, 110, 34, 58, 49, 48, 44, 34, 115, 103,
			120, 116, 99, 98, 99, 111, 109, 112, 48, 50, 115, 118, 110, 34, 58, 49, 48, 44, 34,
			115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 51, 115, 118, 110, 34, 58, 50, 44,
			34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 52, 115, 118, 110, 34, 58, 50,
			44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 53, 115, 118, 110, 34, 58,
			50, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 54, 115, 118, 110, 34,
			58, 49, 50, 56, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 55, 115,
			118, 110, 34, 58, 52, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 56,
			115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48,
			57, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112,
			49, 48, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109,
			112, 49, 49, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111,
			109, 112, 49, 50, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99,
			111, 109, 112, 49, 51, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98,
			99, 111, 109, 112, 49, 52, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99,
			98, 99, 111, 109, 112, 49, 53, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116,
			99, 98, 99, 111, 109, 112, 49, 54, 115, 118, 110, 34, 58, 48, 44, 34, 112, 99, 101,
			115, 118, 110, 34, 58, 49, 48, 125, 44, 34, 116, 99, 98, 68, 97, 116, 101, 34, 58, 34,
			50, 48, 50, 48, 45, 48, 54, 45, 49, 48, 84, 48, 48, 58, 48, 48, 58, 48, 48, 90, 34, 44,
			34, 116, 99, 98, 83, 116, 97, 116, 117, 115, 34, 58, 34, 79, 117, 116, 79, 102, 68, 97,
			116, 101, 34, 125, 44, 123, 34, 116, 99, 98, 34, 58, 123, 34, 115, 103, 120, 116, 99,
			98, 99, 111, 109, 112, 48, 49, 115, 118, 110, 34, 58, 49, 48, 44, 34, 115, 103, 120,
			116, 99, 98, 99, 111, 109, 112, 48, 50, 115, 118, 110, 34, 58, 49, 48, 44, 34, 115,
			103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 51, 115, 118, 110, 34, 58, 50, 44, 34,
			115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 52, 115, 118, 110, 34, 58, 50, 44,
			34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 53, 115, 118, 110, 34, 58, 50,
			44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 54, 115, 118, 110, 34, 58,
			49, 50, 56, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 55, 115, 118,
			110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 56, 115,
			118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 57,
			115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 49,
			48, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112,
			49, 49, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109,
			112, 49, 50, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111,
			109, 112, 49, 51, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99,
			111, 109, 112, 49, 52, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98,
			99, 111, 109, 112, 49, 53, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99,
			98, 99, 111, 109, 112, 49, 54, 115, 118, 110, 34, 58, 48, 44, 34, 112, 99, 101, 115,
			118, 110, 34, 58, 49, 48, 125, 44, 34, 116, 99, 98, 68, 97, 116, 101, 34, 58, 34, 50,
			48, 50, 48, 45, 48, 54, 45, 49, 48, 84, 48, 48, 58, 48, 48, 58, 48, 48, 90, 34, 44, 34,
			116, 99, 98, 83, 116, 97, 116, 117, 115, 34, 58, 34, 79, 117, 116, 79, 102, 68, 97,
			116, 101, 67, 111, 110, 102, 105, 103, 117, 114, 97, 116, 105, 111, 110, 78, 101, 101,
			100, 101, 100, 34, 125, 44, 123, 34, 116, 99, 98, 34, 58, 123, 34, 115, 103, 120, 116,
			99, 98, 99, 111, 109, 112, 48, 49, 115, 118, 110, 34, 58, 49, 44, 34, 115, 103, 120,
			116, 99, 98, 99, 111, 109, 112, 48, 50, 115, 118, 110, 34, 58, 49, 44, 34, 115, 103,
			120, 116, 99, 98, 99, 111, 109, 112, 48, 51, 115, 118, 110, 34, 58, 50, 44, 34, 115,
			103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 52, 115, 118, 110, 34, 58, 50, 44, 34,
			115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 53, 115, 118, 110, 34, 58, 50, 44,
			34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 54, 115, 118, 110, 34, 58, 49,
			50, 56, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 55, 115, 118, 110,
			34, 58, 54, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 56, 115, 118,
			110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 57, 115,
			118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 49, 48,
			115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 49,
			49, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112,
			49, 50, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109,
			112, 49, 51, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111,
			109, 112, 49, 52, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99,
			111, 109, 112, 49, 53, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98,
			99, 111, 109, 112, 49, 54, 115, 118, 110, 34, 58, 48, 44, 34, 112, 99, 101, 115, 118,
			110, 34, 58, 57, 125, 44, 34, 116, 99, 98, 68, 97, 116, 101, 34, 58, 34, 50, 48, 49,
			57, 45, 49, 49, 45, 49, 51, 84, 48, 48, 58, 48, 48, 58, 48, 48, 90, 34, 44, 34, 116,
			99, 98, 83, 116, 97, 116, 117, 115, 34, 58, 34, 79, 117, 116, 79, 102, 68, 97, 116,
			101, 34, 125, 44, 123, 34, 116, 99, 98, 34, 58, 123, 34, 115, 103, 120, 116, 99, 98,
			99, 111, 109, 112, 48, 49, 115, 118, 110, 34, 58, 49, 44, 34, 115, 103, 120, 116, 99,
			98, 99, 111, 109, 112, 48, 50, 115, 118, 110, 34, 58, 49, 44, 34, 115, 103, 120, 116,
			99, 98, 99, 111, 109, 112, 48, 51, 115, 118, 110, 34, 58, 50, 44, 34, 115, 103, 120,
			116, 99, 98, 99, 111, 109, 112, 48, 52, 115, 118, 110, 34, 58, 50, 44, 34, 115, 103,
			120, 116, 99, 98, 99, 111, 109, 112, 48, 53, 115, 118, 110, 34, 58, 50, 44, 34, 115,
			103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 54, 115, 118, 110, 34, 58, 49, 50, 56,
			44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 55, 115, 118, 110, 34, 58,
			48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 56, 115, 118, 110, 34,
			58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 48, 57, 115, 118, 110,
			34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 49, 48, 115, 118,
			110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 49, 49, 115,
			118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 49, 50,
			115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112, 49,
			51, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109, 112,
			49, 52, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111, 109,
			112, 49, 53, 115, 118, 110, 34, 58, 48, 44, 34, 115, 103, 120, 116, 99, 98, 99, 111,
			109, 112, 49, 54, 115, 118, 110, 34, 58, 48, 44, 34, 112, 99, 101, 115, 118, 110, 34,
			58, 57, 125, 44, 34, 116, 99, 98, 68, 97, 116, 101, 34, 58, 34, 50, 48, 49, 57, 45, 49,
			49, 45, 49, 51, 84, 48, 48, 58, 48, 48, 58, 48, 48, 90, 34, 44, 34, 116, 99, 98, 83,
			116, 97, 116, 117, 115, 34, 58, 34, 79, 117, 116, 79, 102, 68, 97, 116, 101, 67, 111,
			110, 102, 105, 103, 117, 114, 97, 116, 105, 111, 110, 78, 101, 101, 100, 101, 100, 34,
			125, 93, 125,
		];
		let str_format = String::from_utf8_lossy(&byte_slice);
		println!("data string: {:#?}", str_format);
		let res: TcbInfo = TcbInfo::from_byte_slice(byte_slice).unwrap();
	}
}
