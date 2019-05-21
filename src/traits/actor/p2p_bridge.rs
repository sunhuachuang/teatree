use actor::prelude::*;

use crate::message::p2p_message::*;

pub trait P2PBridgeActor<R = Context<Self>>
where
    Self: Clone
        + Actor<Context = R>
        + Handler<ReceiveEventMessage>
        + Handler<ReceivePeerJoinMessage>
        + Handler<ReceivePeerLeaveMessage>
        + Handler<ReceivePeerJoinResultMessage>,
    R: ActorContext
        + ToEnvelope<Self, ReceiveEventMessage>
        + ToEnvelope<Self, ReceivePeerJoinMessage>
        + ToEnvelope<Self, ReceivePeerLeaveMessage>
        + ToEnvelope<Self, ReceivePeerJoinResultMessage>,
{
}
