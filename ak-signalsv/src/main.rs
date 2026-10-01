use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use ak_lib::{AddressInfo, AddressRequest, StreamObject};


const DEFAULT_ADDR: &str = "0.0.0.0:3000";


struct PeerId {
    name: String,
    addr: SocketAddr,
}

struct SigServ {
    listener: TcpListener,
    ids: HashMap<SocketAddr, PeerId>
}

impl SigServ {
    pub async fn try_new() -> std::io::Result<Self>
    {
        Ok(Self {
            ids: HashMap::new(),
            listener: TcpListener::bind(DEFAULT_ADDR).await?,
        })
    }

    pub async fn accept_and_process(&mut self) -> std::io::Result<()>
    {
        let (mut socket, peer_addr) = self.listener.accept().await?;

        let addr_req = AddressRequest::read_from_stream(&mut socket).await?;

        self.ids
            .entry(peer_addr)
            .and_modify(|e| {
                e.name = addr_req.peer_name().to_string();
                e.addr = peer_addr;
            })
            .or_insert(PeerId { addr: peer_addr, name: addr_req.peer_name().to_string() });

        println!("! New connection: {} for peer \"{}\" looking for target \"{}\"",
            peer_addr,
            addr_req.peer_name(),
            addr_req.peer_target());

        AddressInfo::from_socket_addr(&peer_addr)
            .write_to_stream(&mut socket)
            .await?;

        match self.ids.iter().find(|(_, id)| id.name == addr_req.peer_target()) {
            Some(id) => {
                println!("! Target \"{}\" exists, sending connection info.", addr_req.peer_target());
                AddressInfo::from_socket_addr(id.0)
                    .write_to_stream(&mut socket)
                    .await?;
            }
            None => {
                println!("! Target \"{}\" still doesn't exist", addr_req.peer_target());
                AddressInfo::empty().write_to_stream(&mut socket).await?;
            }
        };

        Ok(())
    }
}


#[tokio::main]
async fn main() -> std::io::Result<()>
{
    let mut sv = SigServ::try_new().await?;
    println!("# Server listening on {}...", DEFAULT_ADDR);

    loop {
        sv.accept_and_process().await?
    }
}
