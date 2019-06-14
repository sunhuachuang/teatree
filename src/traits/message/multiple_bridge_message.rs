use std::net::SocketAddr;

use crate::actor::prelude::Message;
use crate::primitives::types::{
    BlockByte, EventByte, EventID, LevelPermissionByte, PeerAddr, PeerInfoByte, RPCParams,
};

/// event from p2p network self group.
/// Params is PeerAddr (p2p Node), Event Byte.
#[derive(Clone)]
pub struct MultipleEventMessage(pub PeerAddr, pub EventByte);

impl Message for MultipleEventMessage {
    type Result = ();
}

/// peer join from p2p network.
/// Params is PeerAddr (p2p Node), Peer Join Info Byte.
#[derive(Clone)]
pub struct MultiplePeerJoinMessage(pub PeerAddr, pub PeerInfoByte, pub Option<SocketAddr>);

impl Message for MultiplePeerJoinMessage {
    type Result = ();
}

/// peer join result when receive join request between p2p & bridge.
/// Params is PeerAddr (p2p Node), bool (join ok or not), help some peer addr.
#[derive(Clone)]
pub struct MultiplePeerJoinResultMessage(pub PeerAddr, pub bool, pub Vec<PeerAddr>);

impl Message for MultiplePeerJoinResultMessage {
    type Result = ();
}

/// peer leave from p2p network.
/// Params is PeerAddr (p2p Node), bool if is true, lost by all peers,
/// if false, only first lost by self lost.
#[derive(Clone)]
pub struct MultiplePeerLeaveMessage(pub PeerAddr, pub bool);

impl Message for MultiplePeerLeaveMessage {
    type Result = ();
}

/// rpc request from local outside, or send actor.
/// Params is SoocketAddr, RPCParams.
#[derive(Clone)]
pub struct MultipleLocalMessage(pub usize, pub RPCParams, pub SocketAddr);

impl Message for MultipleLocalMessage {
    type Result = ();
}

/// rpc response from local outside or send to outsize.
/// Params is RPCParams.
#[derive(Clone)]
pub struct MultipleLocalResponseMessage(pub usize, pub Option<RPCParams>);

impl Message for MultipleLocalResponseMessage {
    type Result = ();
}

/// rpc request from upper level group (send block for subscribed).
/// Params is rpc session_id, Block Byte.
#[derive(Clone)]
pub struct MultipleUpperMessage(pub usize, pub BlockByte);

impl Message for MultipleUpperMessage {
    type Result = ();
}

/// rpc request from upper level group (send block for subscribed).
/// Params is EventID.
#[derive(Clone)]
pub struct MultipleUpperResponseMessage(pub usize, pub Option<EventID>);

impl Message for MultipleUpperResponseMessage {
    type Result = ();
}

/// rpc request from lower level group (send block get more security).
/// Params is rpc session_id, Block Byte.
#[derive(Clone)]
pub struct MultipleLowerMessage(pub usize, pub BlockByte);

impl Message for MultipleLowerMessage {
    type Result = ();
}

/// rpc request from lower level group (send block get more security).
/// Params is EventID.
#[derive(Clone)]
pub struct MultipleLowerResponseMessage(pub usize, pub Option<EventID>);

impl Message for MultipleLowerResponseMessage {
    type Result = ();
}

/// rpc level permission request.
/// Params is LevelPermissionByte.
#[derive(Clone)]
pub struct MultipleLevelPermissionMessage(pub usize, pub LevelPermissionByte, pub SocketAddr);

impl Message for MultipleLevelPermissionMessage {
    type Result = ();
}

/// rpc level permission response.
/// Params is LevelPermissionByte.
#[derive(Clone)]
pub struct MultipleLevelPermissionResponseMessage(pub usize, pub bool);

impl Message for MultipleLevelPermissionResponseMessage {
    type Result = ();
}
