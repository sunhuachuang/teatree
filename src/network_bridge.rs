use std::collections::HashMap;
use std::marker::Send;

use crate::actor::prelude::*;
use crate::p2p::P2PActor;
use crate::primitives::functions::{try_resend_times, DEFAULT_TIMES};
use crate::primitives::types::GroupID;
use crate::rpc::RPCActor;
use crate::traits::actor::{BridgeActor, P2PBridgeActor, RPCBridgeActor};
use crate::traits::message::bridge_message::*;
use crate::traits::message::p2p_message::*;
use crate::traits::message::rpc_message::*;

#[derive(Clone)]
pub struct NetworkBridgeActor<B: BridgeActor> {
    p2p_addr: Addr<P2PActor<Self>>,
    rpc_addr: Addr<RPCActor<Self>>,
    bridges: HashMap<GroupID, Addr<B>>,
}

impl<B: BridgeActor> NetworkBridgeActor<B> {
    pub fn load(p2p_addr: Addr<P2PActor<Self>>, rpc_addr: Addr<RPCActor<Self>>) -> Self {
        let bridges = HashMap::new();

        Self {
            p2p_addr,
            rpc_addr,
            bridges,
        }
    }

    /// try send received event to bridge actor
    fn send_bridge<M: 'static>(&self, group_id: GroupID, message: M)
    where
        B: Handler<M>,
        M: Message + Send + Clone,
        <M as Message>::Result: Send,
        <B as Actor>::Context: ToEnvelope<B, M>,
    {
        if self.bridges.contains_key(&group_id) {
            let addr = self.bridges.get(&group_id).unwrap().clone();
            let _ = try_resend_times(addr, message, DEFAULT_TIMES)
                .map_err(|_| println!("Send Message to udp fail"));
        }
    }

    /// try send received event to p2p actor
    fn send_p2p<M: 'static>(&self, message: M)
    where
        P2PActor<Self>: Handler<M>,
        M: Message + Send + Clone,
        <M as Message>::Result: Send,
        <P2PActor<Self> as Actor>::Context: ToEnvelope<P2PActor<Self>, M>,
    {
        let _ = try_resend_times(self.p2p_addr.clone(), message, DEFAULT_TIMES)
            .map_err(|_| println!("Send Message to udp fail"));
    }

    /// try send received event to rpc actor
    fn send_rpc<M: 'static>(&self, message: M)
    where
        RPCActor<Self>: Handler<M>,
        M: Message + Send + Clone,
        <M as Message>::Result: Send,
        <RPCActor<Self> as Actor>::Context: ToEnvelope<RPCActor<Self>, M>,
    {
        let _ = try_resend_times(self.rpc_addr.clone(), message, DEFAULT_TIMES)
            .map_err(|_| println!("Send Message to udp fail"));
    }
}

/// impl Actor for NetworkBridgeActor
impl<B: BridgeActor> Actor for NetworkBridgeActor<B> {
    type Context = Context<Self>;

    /// when start register to p2p and rpc actor
    fn started(&mut self, ctx: &mut Self::Context) {
        self.send_p2p(P2PBridgeAddrMessage(ctx.address()));
        self.send_rpc(RPCBridgeAddrMessage(ctx.address()));
    }
}

/// impl BridgeActor for NetworkBridgeActor
impl<B: BridgeActor> BridgeActor for NetworkBridgeActor<B> {}

impl<B: BridgeActor> Handler<BridgeAddrMessage<B>> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(&mut self, msg: BridgeAddrMessage<B>, _ctx: &mut Self::Context) -> Self::Result {
        self.bridges.insert(msg.0, msg.1);
    }
}

/// receive local rpc request from bridge actor, and send to rpc
impl<B: BridgeActor> Handler<LocalMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(&mut self, msg: LocalMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_rpc(ReceiveLocalMessage(msg.0, msg.1, msg.2, msg.3));
    }
}

/// receive send to upper rpc request from bridge actor, and send to rpc
impl<B: BridgeActor> Handler<UpperMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(&mut self, msg: UpperMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_rpc(ReceiveUpperMessage(msg.0, msg.1, msg.2));
    }
}

/// receive send to lower rpc request from bridge actor, and send to rpc
impl<B: BridgeActor> Handler<LowerMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(&mut self, msg: LowerMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_rpc(ReceiveLowerMessage(msg.0, msg.1, msg.2));
    }
}

/// receive send to lower rpc request from bridge actor, and send to rpc
impl<B: BridgeActor> Handler<LevelPermissionMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(&mut self, msg: LevelPermissionMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_rpc(ReceiveLevelPermissionMessage(msg.0, msg.1, msg.2, msg.3));
    }
}

impl<B: BridgeActor> Handler<LocalResponseMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(&mut self, msg: LocalResponseMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_rpc(ReceiveLocalResponseMessage(msg.0, msg.1, msg.2));
    }
}

impl<B: BridgeActor> Handler<UpperResponseMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(&mut self, msg: UpperResponseMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_rpc(ReceiveUpperResponseMessage(msg.0, msg.1, msg.2));
    }
}

impl<B: BridgeActor> Handler<LowerResponseMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(&mut self, msg: LowerResponseMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_rpc(ReceiveLowerResponseMessage(msg.0, msg.1, msg.2));
    }
}

impl<B: BridgeActor> Handler<LevelPermissionResponseMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(
        &mut self,
        msg: LevelPermissionResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.send_rpc(ReceiveLevelPermissionResponseMessage(msg.0, msg.1, msg.2));
    }
}

/// receive event message from bridge actor, and send to p2p
impl<B: BridgeActor> Handler<EventMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(&mut self, msg: EventMessage, _ctx: &mut Self::Context) {
        self.send_p2p(ReceiveEventMessage(msg.0, msg.1, msg.2));
    }
}

/// receive peer join message from bridge actor, and send to p2p
impl<B: BridgeActor> Handler<PeerJoinMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(&mut self, msg: PeerJoinMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_p2p(ReceivePeerJoinMessage(msg.0, msg.1, msg.2, msg.3));
    }
}

/// receive peer join result from bridge actor, and send to p2p
impl<B: BridgeActor> Handler<PeerJoinResultMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(&mut self, msg: PeerJoinResultMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_p2p(ReceivePeerJoinResultMessage(msg.0, msg.1, msg.2, msg.3));
    }
}

/// receive peer leave message from bridge actor, and send to p2p
impl<B: BridgeActor> Handler<PeerLeaveMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(&mut self, msg: PeerLeaveMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_p2p(ReceivePeerLeaveMessage(msg.0, msg.1, msg.2));
    }
}

/// impl RPCBridgeActor for NetworkBridgeActor {}
impl<B: BridgeActor> P2PBridgeActor for NetworkBridgeActor<B> {}

/// receive event message from p2p actor, and send to bridge
impl<B: BridgeActor> Handler<ReceiveEventMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(&mut self, msg: ReceiveEventMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_bridge(msg.0.clone(), EventMessage(msg.0, msg.1, msg.2));
    }
}

/// receive peer join message from p2p actor, and send to bridge
impl<B: BridgeActor> Handler<ReceivePeerJoinMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(&mut self, msg: ReceivePeerJoinMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_bridge(msg.0.clone(), PeerJoinMessage(msg.0, msg.1, msg.2, msg.3));
    }
}

/// receive peer join result from bridge actor, and send to p2p
impl<B: BridgeActor> Handler<ReceivePeerJoinResultMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(
        &mut self,
        msg: ReceivePeerJoinResultMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.send_bridge(
            msg.0.clone(),
            PeerJoinResultMessage(msg.0, msg.1, msg.2, msg.3),
        );
    }
}

/// receive peer leave message from p2p actor, and send to bridge
impl<B: BridgeActor> Handler<ReceivePeerLeaveMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(&mut self, msg: ReceivePeerLeaveMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_bridge(msg.0.clone(), PeerLeaveMessage(msg.0, msg.1, msg.2));
    }
}

/// impl RPCBridgeActor for NetworkBridgeActor
impl<B: BridgeActor> RPCBridgeActor for NetworkBridgeActor<B> {}

/// receive local rpc request from bridge actor, and send to rpc
impl<B: BridgeActor> Handler<ReceiveLocalMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(&mut self, msg: ReceiveLocalMessage, _ctx: &mut Self::Context) -> Self::Result {
        if !self.bridges.contains_key(&msg.0) {
            return self.send_rpc(ReceiveLevelPermissionResponseMessage(
                msg.0.clone(),
                msg.1,
                false,
            ));
        }

        self.send_bridge(msg.0.clone(), LocalMessage(msg.0, msg.1, msg.2, msg.3));
    }
}

/// receive send to upper rpc request from bridge actor, and send to rpc
impl<B: BridgeActor> Handler<ReceiveUpperMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(&mut self, msg: ReceiveUpperMessage, _ctx: &mut Self::Context) -> Self::Result {
        if !self.bridges.contains_key(&msg.0) {
            return self.send_rpc(ReceiveLevelPermissionResponseMessage(
                msg.0.clone(),
                msg.1,
                false,
            ));
        }

        self.send_bridge(msg.0.clone(), UpperMessage(msg.0, msg.1, msg.2));
    }
}

/// receive send to lower rpc request from bridge actor, and send to rpc
impl<B: BridgeActor> Handler<ReceiveLowerMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(&mut self, msg: ReceiveLowerMessage, _ctx: &mut Self::Context) -> Self::Result {
        if !self.bridges.contains_key(&msg.0) {
            return self.send_rpc(ReceiveLevelPermissionResponseMessage(
                msg.0.clone(),
                msg.1,
                false,
            ));
        }

        self.send_bridge(msg.0.clone(), LowerMessage(msg.0, msg.1, msg.2));
    }
}

impl<B: BridgeActor> Handler<ReceiveLevelPermissionMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(
        &mut self,
        msg: ReceiveLevelPermissionMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        if !self.bridges.contains_key(&msg.0) {
            return self.send_rpc(ReceiveLevelPermissionResponseMessage(
                msg.0.clone(),
                msg.1,
                false,
            ));
        }

        self.send_bridge(
            msg.0.clone(),
            LevelPermissionMessage(msg.0, msg.1, msg.2, msg.3),
        );
    }
}

impl<B: BridgeActor> Handler<ReceiveLocalResponseMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(
        &mut self,
        msg: ReceiveLocalResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.send_bridge(msg.0.clone(), LocalResponseMessage(msg.0, msg.1, msg.2));
    }
}

impl<B: BridgeActor> Handler<ReceiveUpperResponseMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(
        &mut self,
        msg: ReceiveUpperResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.send_bridge(msg.0.clone(), UpperResponseMessage(msg.0, msg.1, msg.2));
    }
}

impl<B: BridgeActor> Handler<ReceiveLowerResponseMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(
        &mut self,
        msg: ReceiveLowerResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.send_bridge(msg.0.clone(), LowerResponseMessage(msg.0, msg.1, msg.2));
    }
}

impl<B: BridgeActor> Handler<ReceiveLevelPermissionResponseMessage> for NetworkBridgeActor<B> {
    type Result = ();

    fn handle(
        &mut self,
        msg: ReceiveLevelPermissionResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.send_bridge(
            msg.0.clone(),
            LevelPermissionResponseMessage(msg.0, msg.1, msg.2),
        );
    }
}
