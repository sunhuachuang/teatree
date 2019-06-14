use std::marker::Send;
use std::net::SocketAddr;

use crate::actor::prelude::*;
use crate::crypto::keypair::PrivateKey;
use crate::primitives::types::GroupID;
use crate::traits::actor::BridgeActor;
use crate::traits::actor::MultipleBridgeActor;
use crate::traits::message::bridge_message::*;
use crate::traits::message::multiple_bridge_message::*;
use crate::{network_start, NetworkBridgeActor};

#[derive(Clone)]
pub struct MultipleNetworkBridgeActor {
    group_id: GroupID,
    network_bridge: Addr<NetworkBridgeActor<Self>>,

    recipient_event: Recipient<MultipleEventMessage>,
    recipient_peer_join: Recipient<MultiplePeerJoinMessage>,
    recipient_peer_join_result: Recipient<MultiplePeerJoinResultMessage>,
    recipient_peer_leave: Recipient<MultiplePeerLeaveMessage>,

    recipient_local: Recipient<MultipleLocalMessage>,
    recipient_upper: Recipient<MultipleUpperMessage>,
    recipient_lower: Recipient<MultipleLowerMessage>,

    recipient_local_response: Recipient<MultipleLocalResponseMessage>,
    recipient_upper_response: Recipient<MultipleUpperResponseMessage>,
    recipient_lower_response: Recipient<MultipleLowerResponseMessage>,
    recipient_level_permission: Recipient<MultipleLevelPermissionMessage>,
    recipient_level_permission_response: Recipient<MultipleLevelPermissionResponseMessage>,
}

impl MultipleNetworkBridgeActor {
    pub fn init(
        group_id: GroupID,
        p2p_socket: SocketAddr,
        rpc_socket: SocketAddr,
        psk: Option<PrivateKey>,
        addr: &Addr<impl MultipleBridgeActor>,
    ) -> Self {
        let network_bridge = network_start::<Self>(p2p_socket, rpc_socket, psk);

        Self::new(group_id, network_bridge, addr)
    }

    pub fn network_addr(&self) -> &Addr<NetworkBridgeActor<Self>> {
        &self.network_bridge
    }

    pub fn new(
        group_id: GroupID,
        network_bridge: Addr<NetworkBridgeActor<Self>>,
        addr: &Addr<impl MultipleBridgeActor>,
    ) -> Self {
        Self {
            group_id: group_id,
            network_bridge: network_bridge,
            recipient_event: addr.clone().recipient::<MultipleEventMessage>(),
            recipient_peer_join: addr.clone().recipient::<MultiplePeerJoinMessage>(),
            recipient_peer_join_result: addr.clone().recipient::<MultiplePeerJoinResultMessage>(),
            recipient_peer_leave: addr.clone().recipient::<MultiplePeerLeaveMessage>(),

            recipient_local: addr.clone().recipient::<MultipleLocalMessage>(),
            recipient_upper: addr.clone().recipient::<MultipleUpperMessage>(),
            recipient_lower: addr.clone().recipient::<MultipleLowerMessage>(),

            recipient_local_response: addr.clone().recipient::<MultipleLocalResponseMessage>(),
            recipient_upper_response: addr.clone().recipient::<MultipleUpperResponseMessage>(),
            recipient_lower_response: addr.clone().recipient::<MultipleLowerResponseMessage>(),
            recipient_level_permission: addr.clone().recipient::<MultipleLevelPermissionMessage>(),
            recipient_level_permission_response: addr
                .clone()
                .recipient::<MultipleLevelPermissionResponseMessage>(),
        }
    }

    /// try send received event to bridge actor
    fn send_network<M: 'static>(&self, message: M)
    where
        NetworkBridgeActor<Self>: Handler<M>,
        M: Message + Send,
        <M as Message>::Result: Send,
        <NetworkBridgeActor<Self> as Actor>::Context: ToEnvelope<NetworkBridgeActor<Self>, M>,
    {
        self.network_bridge.do_send(message);
    }
}

/// impl Actor for NetworkBridgeActor
impl Actor for MultipleNetworkBridgeActor {
    type Context = Context<Self>;

    /// when start register to p2p and rpc actor
    fn started(&mut self, ctx: &mut Self::Context) {
        self.send_network(BridgeAddrMessage(self.group_id.clone(), ctx.address()));
    }
}

/// impl BridgeActor for MultipleNetworkBridgeActor
impl BridgeActor for MultipleNetworkBridgeActor {}

/// receive local rpc request from bridge actor, and send to rpc
impl Handler<LocalMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: LocalMessage, _ctx: &mut Self::Context) -> Self::Result {
        let _ = self
            .recipient_local
            .do_send(MultipleLocalMessage(msg.1, msg.2, msg.3));
    }
}

/// receive send to upper rpc request from bridge actor, and send to rpc
impl Handler<UpperMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: UpperMessage, _ctx: &mut Self::Context) -> Self::Result {
        let _ = self
            .recipient_upper
            .do_send(MultipleUpperMessage(msg.1, msg.2));
    }
}

/// receive send to lower rpc request from bridge actor, and send to rpc
impl Handler<LowerMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: LowerMessage, _ctx: &mut Self::Context) -> Self::Result {
        let _ = self
            .recipient_lower
            .do_send(MultipleLowerMessage(msg.1, msg.2));
    }
}

/// receive send to lower rpc request from bridge actor, and send to rpc
impl Handler<LevelPermissionMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: LevelPermissionMessage, _ctx: &mut Self::Context) -> Self::Result {
        let _ = self
            .recipient_level_permission
            .do_send(MultipleLevelPermissionMessage(msg.1, msg.2, msg.3));
    }
}

impl Handler<LocalResponseMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: LocalResponseMessage, _ctx: &mut Self::Context) -> Self::Result {
        let _ = self
            .recipient_local_response
            .do_send(MultipleLocalResponseMessage(msg.1, msg.2));
    }
}

impl Handler<UpperResponseMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: UpperResponseMessage, _ctx: &mut Self::Context) -> Self::Result {
        let _ = self
            .recipient_upper_response
            .do_send(MultipleUpperResponseMessage(msg.1, msg.2));
    }
}

impl Handler<LowerResponseMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: LowerResponseMessage, _ctx: &mut Self::Context) -> Self::Result {
        let _ = self
            .recipient_lower_response
            .do_send(MultipleLowerResponseMessage(msg.1, msg.2));
    }
}

impl Handler<LevelPermissionResponseMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: LevelPermissionResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let _ = self
            .recipient_level_permission_response
            .do_send(MultipleLevelPermissionResponseMessage(msg.1, msg.2));
    }
}

/// receive event message from bridge actor, and send to p2p
impl Handler<EventMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: EventMessage, _ctx: &mut Self::Context) {
        let _ = self
            .recipient_event
            .do_send(MultipleEventMessage(msg.1, msg.2));
    }
}

/// receive peer join message from bridge actor, and send to p2p
impl Handler<PeerJoinMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: PeerJoinMessage, _ctx: &mut Self::Context) -> Self::Result {
        let _ = self
            .recipient_peer_join
            .do_send(MultiplePeerJoinMessage(msg.1, msg.2, msg.3));
    }
}

/// receive peer join message from bridge actor, and send to p2p
impl Handler<PeerJoinResultMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: PeerJoinResultMessage, _ctx: &mut Self::Context) -> Self::Result {
        let _ = self
            .recipient_peer_join_result
            .do_send(MultiplePeerJoinResultMessage(msg.1, msg.2, msg.3));
    }
}

/// receive peer leave message from bridge actor, and send to p2p
impl Handler<PeerLeaveMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: PeerLeaveMessage, _ctx: &mut Self::Context) -> Self::Result {
        let _ = self
            .recipient_peer_leave
            .do_send(MultiplePeerLeaveMessage(msg.1, msg.2));
    }
}

/// receive local rpc request from bridge actor, and send to rpc
impl Handler<MultipleLocalMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: MultipleLocalMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_network(LocalMessage(self.group_id.clone(), msg.0, msg.1, msg.2))
    }
}

/// receive send to upper rpc request from bridge actor, and send to rpc
impl Handler<MultipleUpperMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: MultipleUpperMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_network(UpperMessage(self.group_id.clone(), msg.0, msg.1))
    }
}

/// receive send to lower rpc request from bridge actor, and send to rpc
impl Handler<MultipleLowerMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: MultipleLowerMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_network(LowerMessage(self.group_id.clone(), msg.0, msg.1))
    }
}

/// receive send to lower rpc request from bridge actor, and send to rpc
impl Handler<MultipleLevelPermissionMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: MultipleLevelPermissionMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.send_network(LevelPermissionMessage(
            self.group_id.clone(),
            msg.0,
            msg.1,
            msg.2,
        ))
    }
}

impl Handler<MultipleLocalResponseMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: MultipleLocalResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.send_network(LocalResponseMessage(self.group_id.clone(), msg.0, msg.1))
    }
}

impl Handler<MultipleUpperResponseMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: MultipleUpperResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.send_network(UpperResponseMessage(self.group_id.clone(), msg.0, msg.1))
    }
}

impl Handler<MultipleLowerResponseMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: MultipleLowerResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.send_network(LowerResponseMessage(self.group_id.clone(), msg.0, msg.1))
    }
}

impl Handler<MultipleLevelPermissionResponseMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: MultipleLevelPermissionResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.send_network(LevelPermissionResponseMessage(
            self.group_id.clone(),
            msg.0,
            msg.1,
        ))
    }
}

/// receive event message from bridge actor, and send to p2p
impl Handler<MultipleEventMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: MultipleEventMessage, _ctx: &mut Self::Context) {
        self.send_network(EventMessage(self.group_id.clone(), msg.0, msg.1))
    }
}

/// receive peer join message from bridge actor, and send to p2p
impl Handler<MultiplePeerJoinMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: MultiplePeerJoinMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_network(PeerJoinMessage(self.group_id.clone(), msg.0, msg.1, msg.2))
    }
}

/// receive peer join message from bridge actor, and send to p2p
impl Handler<MultiplePeerJoinResultMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: MultiplePeerJoinResultMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.send_network(PeerJoinResultMessage(
            self.group_id.clone(),
            msg.0,
            msg.1,
            msg.2,
        ))
    }
}

/// receive peer leave message from bridge actor, and send to p2p
impl Handler<MultiplePeerLeaveMessage> for MultipleNetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: MultiplePeerLeaveMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_network(PeerLeaveMessage(self.group_id.clone(), msg.0, msg.1))
    }
}
