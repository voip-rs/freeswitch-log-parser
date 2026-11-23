//! Message shapes the tracker reads out of dialplan, originate and bridge lines.

use freeswitch_types::commands::{
    DestinationEncoding, DialStringCarrier, DialStringTarget, Endpoint, FlattenedDialString,
    FlattenedLeg, LegTarget,
};
use freeswitch_types::{BlockParse, ChannelVariable};

use crate::fields::processing_parts;
use crate::message::new_channel_name;

/// The bracket-block parser revision a `bridge()` argument is read for unless the caller
/// names one. A log does not record which switch version wrote it.
pub const DEFAULT_BLOCK_PARSE: BlockParse = BlockParse::PairSplitCleans;

pub(super) struct DialplanContext {
    pub(super) from: String,
    /// Display name of the caller side, `None` unless the line brackets a number.
    pub(super) from_name: Option<String>,
    /// Number inside the caller side's `<>`, brackets excluded.
    pub(super) from_number: Option<String>,
    pub(super) to: String,
    pub(super) context: String,
}

/// The context of a `parsing [<context>-><extension>]` line.
///
/// The extension is deliberately dropped: it names a dialplan entry point, not
/// the dialed number `dialplan_to` carries, and folding the two into one field
/// left it meaning whichever shape was logged last.
pub(super) fn parse_dialplan_context(detail: &str) -> Option<&str> {
    let rest = detail.strip_prefix("parsing [")?;
    let bracket_end = rest.find(']')?;
    let inner = &rest[..bracket_end];
    let arrow = inner.find("->")?;
    Some(&inner[..arrow])
}

pub(super) fn parse_processing_line(msg: &str) -> Option<DialplanContext> {
    let parts = processing_parts(msg)?;
    Some(DialplanContext {
        from: msg[parts.head].to_string(),
        from_name: parts.name.map(|r| msg[r].to_string()),
        from_number: parts.number.map(|r| msg[r].to_string()),
        to: msg[parts.dest].to_string(),
        context: msg[parts.context].to_string(),
    })
}

pub(super) fn parse_new_channel(detail: &str) -> Option<&str> {
    new_channel_name(detail)
}

/// The cause in the last bracket of a `Hangup <channel> [<state>] [<cause>]`
/// detail. The line's shape is the classifier's finding, not this one's.
pub(super) fn parse_hangup(detail: &str) -> Option<&str> {
    let start = detail.rfind('[')?;
    let end = detail[start..].find(']')?;
    Some(&detail[start + 1..start + end])
}

/// What `bridge()` arguments say about the first leg they create: its forced UUID and
/// the channel names it may report.
///
/// `block_parse` is the revision of the switch that wrote the log, [`DEFAULT_BLOCK_PARSE`]
/// when unknown. Returns `None` when the arguments do not parse as a dial string or name
/// no leg at all.
pub fn parse_bridge_args(arguments: &str, block_parse: BlockParse) -> Option<BridgeInfo> {
    let target = DialStringTarget::new(DialStringCarrier::Dialplan, block_parse);
    let dial = FlattenedDialString::parse_for(arguments, target).ok()?;
    let leg = dial.legs().next()?;
    let origination_uuid = leg
        .endpoint_variable(ChannelVariable::OriginationUuid)
        .map(str::to_string);
    Some(BridgeInfo {
        origination_uuid,
        target_channels: target_channels(leg),
    })
}

/// The names a sofia profile leg may take. With no block setting the destination
/// encoding, the A leg's exports decide it, so both readings are candidates.
fn target_channels(leg: &FlattenedLeg) -> Vec<String> {
    let LegTarget::Endpoint(Endpoint::Sofia(endpoint)) = leg.target() else {
        return Vec::new();
    };
    let encodings = match leg.destination_encoding() {
        Some(declared) => vec![declared],
        None => vec![
            DestinationEncoding::Encoded,
            DestinationEncoding::Suppressed,
        ],
    };
    let mut names: Vec<String> = Vec::new();
    for name in encodings
        .into_iter()
        .filter_map(|e| endpoint.channel_name(e))
    {
        if !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

/// What a `bridge()` argument list says about the leg it is about to create.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BridgeInfo {
    /// The UUID the new leg is being forced to take, when the dial string sets one.
    pub origination_uuid: Option<String>,
    /// Channel names the new leg may report on its `New Channel` line, the switch's naming
    /// applied to the first endpoint. Empty when no name can be predicted.
    pub target_channels: Vec<String>,
}
