use futures::stream::SplitSink;
use futures::Sink;
use std::net::SocketAddr;
use tokio::net::UdpFramed;

use crate::actor::prelude::*;
use crate::primitives::functions::{try_resend_times, DEFAULT_TIMES};
use crate::traits::actor::P2PBridgeActor;

use super::codec::{P2PBody, P2PCodec, P2PHead};
use super::content::P2PContent;
use super::p2p::P2PActor;

/// message between session and p2p actor.
#[derive(Clone)]
pub struct P2PMessage(pub P2PHead, pub P2PContent, pub SocketAddr);

impl Message for P2PMessage {
    type Result = ();
}

/// p2p addr message, need register to p2p session
#[derive(Clone)]
pub(crate) struct P2PAddrMessage<A: P2PBridgeActor>(pub Addr<P2PActor<A>>);

impl<A: P2PBridgeActor> Message for P2PAddrMessage<A> {
    type Result = ();
}

pub struct P2PSessionActor<A: P2PBridgeActor> {
    pub sinks: Vec<SplitSink<UdpFramed<P2PCodec>>>,
    pub p2p_addr: Option<Addr<P2PActor<A>>>,
    pub waitings: Vec<(P2PHead, P2PBody, SocketAddr)>,
}

impl<A: P2PBridgeActor> P2PSessionActor<A> {
    fn send_udp(&mut self, mut bytes: Vec<u8>, socket: SocketAddr, ctx: &mut Context<Self>) {
        let (now, next) = if bytes.len() > 65500 {
            let now = bytes.drain(0..65500).as_slice().into();
            (now, bytes)
        } else {
            (bytes, vec![])
        };

        self.sinks.pop().and_then(|sink| {
            let _ = sink
                .send((now, socket.clone()))
                .into_actor(self)
                .then(move |res, act, ctx| {
                    match res {
                        Ok(sink) => {
                            act.sinks.push(sink);
                            if !next.is_empty() {
                                act.send_udp(next, socket, ctx);
                            }
                        }
                        Err(_) => panic!("DEBUG: NETWORK HAVE ERROR"),
                    }

                    actor_ok(())
                })
                .wait(ctx);
            Some(())
        });
    }
}

impl<A: P2PBridgeActor> Actor for P2PSessionActor<A> {
    type Context = Context<Self>;
}

/// when receive P2PMessage, send it to that socket.
impl<A: P2PBridgeActor> Handler<P2PAddrMessage<A>> for P2PSessionActor<A> {
    type Result = ();

    fn handle(&mut self, msg: P2PAddrMessage<A>, _ctx: &mut Context<Self>) {
        self.p2p_addr = Some(msg.0);
    }
}

/// when receive from upd stream, send to p2p actor to handle.
impl<A: P2PBridgeActor> StreamHandler<P2PMessage, std::io::Error> for P2PSessionActor<A> {
    fn handle(&mut self, msg: P2PMessage, _ctx: &mut Context<Self>) {
        if self.p2p_addr.is_some() {
            let _ = try_resend_times(self.p2p_addr.clone().unwrap(), msg, DEFAULT_TIMES).map_err(
                |_| {
                    println!("Send Message to p2p fail");
                },
            );
        }
    }
}

/// when receive P2PMessage, send it to that socket.
impl<A: P2PBridgeActor> Handler<P2PMessage> for P2PSessionActor<A> {
    type Result = ();

    fn handle(&mut self, msg: P2PMessage, ctx: &mut Context<Self>) {
        self.waitings.push((msg.0, P2PBody(msg.1), msg.2));
        if self.sinks.is_empty() {
            return;
        }

        while !self.waitings.is_empty() {
            let w = self.waitings.remove(0);
            if self.sinks.is_empty() {
                self.waitings.push(w);
                break;
            }
            let (mut head, body, socket) = (w.0, w.1, w.2);

            let mut body_bytes: Vec<u8> = bincode::serialize(&body).unwrap_or(vec![]);
            head.update_len(body_bytes.len() as u32);
            let mut head_bytes = head.encode().to_vec();
            let mut bytes = vec![];
            bytes.append(&mut head_bytes);
            bytes.append(&mut body_bytes);

            self.send_udp(bytes, socket, ctx);
        }
    }
}
