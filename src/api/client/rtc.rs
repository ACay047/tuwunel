use axum::extract::State;
use ruma::api::client::rtc::{RtcTransport, transports};
use tuwunel_core::{Result, err, error::inspect_log};
use tuwunel_service::Services;

use crate::Ruma;

/// # `GET /_matrix/client/unstable/org.matrix.msc4143/rtc/transports`
///
/// Get MatrixRTC transports for MSC4143.
pub(crate) async fn get_transports_route(
	State(services): State<crate::State>,
	_body: Ruma<transports::v1::Request>,
) -> Result<transports::v1::Response> {
	Ok(transports::v1::Response {
		rtc_transports: get_transports(&services)?,
	})
}

/// Build the configured RTC transports as `RtcTransport` values, the typed
/// form shared between `.well-known/matrix/client.rtc_foci` and the
/// `/rtc/transports` endpoint.
pub(crate) fn get_transports(services: &Services) -> Result<Vec<RtcTransport>> {
	let custom = services
		.server
		.config
		.well_known
		.rtc_transports
		.iter()
		.map(|item| {
			let mut data = item
				.as_object()
				.cloned()
				.ok_or_else(|| err!("`rtc_transport` is not a valid object"))?;

			let transport_type = data
				.remove("type")
				.and_then(|v| v.as_str().map(str::to_owned))
				.ok_or_else(|| err!("`type` is not a valid string"))?;

			RtcTransport::new(&transport_type, data).map_err(|e| {
				err!(Config("global.well_known.rtc_transports", "Malformed value(s): {e:?}"))
			})
		});

	let livekit_url = services
		.config
		.well_known
		.livekit_url
		.iter()
		.cloned()
		.map(|url| Ok(RtcTransport::livekit(url)));

	custom
		.chain(livekit_url)
		.collect::<Result<Vec<_>>>()
		.inspect_err(inspect_log)
}
