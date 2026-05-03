use axum::extract::State;
use ruma::api::client::rtc::transports::{self, v1::RtcTransport};
use serde_json::Value;
use tuwunel_core::{Result, err, error::inspect_log};
use tuwunel_service::Services;

use crate::Ruma;

/// # `GET /_matrix/client/unstable/org.matrix.msc4143/rtc/transports`
///
/// Get MatrixRTC transports for MSC4143
pub(crate) async fn get_transports_route(
	State(services): State<crate::State>,
	_body: Ruma<transports::v1::Request>,
) -> Result<transports::v1::Response> {
	let transports = get_transports(&services)?;

	Ok(transports::v1::Response { rtc_transports: transports })
}

pub(crate) fn get_transports(services: &Services) -> Result<Vec<RtcTransport>> {
	// Add RTC transport configuration if available (MSC4143 / Element Call)
	// Element Call has evolved through several versions with different field
	// expectations
	services
		.server
		.config
		.well_known
		.rtc_transports
		.iter()
		.map(|transport| {
			let focus_type = transport
				.get("type")
				.and_then(Value::as_str)
				.ok_or_else(|| err!("`type` is not a valid string"))?;

			let transport = transport
				.as_object()
				.cloned()
				.ok_or_else(|| err!("`rtc_transport` is not a valid object"))?;

			RtcTransport::new(focus_type.to_owned(), transport).map_err(Into::into)
		})
		.map(|transport: Result<_>| {
			transport.map_err(|e| {
				err!(Config("global.well_known.rtc_transports", "Malformed value(s): {e:?}"))
			})
		})
		.chain(
			services
				.config
				.well_known
				.livekit_url
				.iter()
				.map(|livekit_url| {
					// MSC4143 split out a typed `LivekitMultiSfuTransport`; this legacy
					// single-URL config maps to a custom transport for now.
					let mut data = serde_json::Map::new();
					data.insert("livekit_service_url".into(), Value::String(livekit_url.clone()));
					RtcTransport::new("livekit".to_owned(), data).map_err(Into::into)
				}),
		)
		.collect::<Result<_>>()
		.inspect_err(inspect_log)
}
