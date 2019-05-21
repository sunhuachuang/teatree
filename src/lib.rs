use std::net::SocketAddr;

mod config;
mod network_bridge;

use actor::prelude::{Actor, Addr, System, SystemRunner};
use crypto::keypair::PrivateKey;
use p2p::p2p_start;
use rpc::rpc_start;
use traits::actor::BridgeActor;

pub use actor;
pub use crypto;
pub use primitives;
pub use traits;

pub use config::Configure;
pub use network_bridge::NetworkBridgeActor;

pub fn system_init() -> SystemRunner {
    System::new("Teatree")
}

pub fn system_run(runner: SystemRunner) {
    let _ = runner.run();
}

pub fn network_start<A: BridgeActor>(
    p2p_socket: SocketAddr,
    rpc_socket: SocketAddr,
    psk: Option<PrivateKey>,
) -> Addr<NetworkBridgeActor<A>> {
    let p2p_addr = p2p_start::<NetworkBridgeActor<A>>(p2p_socket, psk);
    let rpc_addr = rpc_start::<NetworkBridgeActor<A>>(rpc_socket);

    NetworkBridgeActor::create(|ctx| {
        ctx.set_mailbox_capacity(100);
        NetworkBridgeActor::load(p2p_addr, rpc_addr)
    })
}
