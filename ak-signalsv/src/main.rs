use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::io::{AsyncWriteExt, AsyncReadExt};


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
        let (mut socket, addr) = self.listener.accept().await?;

        // Request:
        // 50bytes => the name of the peer
        // 50bytes => the name of the target
        let mut name_buf = [0;50];
        socket.read_exact(&mut name_buf).await?;
        let name = String::from_utf8_lossy(&name_buf);

        self.ids
            .entry(addr)
            .and_modify(|e| {
                e.name = name.to_string();
                e.addr = addr;
            })
            .or_insert(PeerId { addr, name: name.to_string() });

        let mut target_buf = [0;50];
        socket.read_exact(&mut target_buf).await?;
        let target = String::from_utf8_lossy(&target_buf);

        println!("! New connection: {} for peer \"{}\" looking for target \"{}\"", addr, name, target);

        // Response:
        // 1byte => peer address size
        // nbute => peer address
        // 1byte => 0 if target doesn't exist, >0 if it does (1byte is the size of next packet);
        // nbyte => the target address
        let peer_addr = addr.to_string().into_bytes();
        let mut payload = Vec::new();
        payload.push(peer_addr.len() as u8);
        let payload = payload.into_iter().chain(peer_addr.into_iter()).collect::<Vec<u8>>();
        socket.write_all(&payload).await?;

        match self.ids.iter().find(|(_, id)| id.name == target) {
            Some(id) => {
                println!("! Target \"{}\" exists, sending connection info.", target);
                let addr = id.0.to_string().into_bytes();

                let mut payload = Vec::new();
                payload.push(addr.len() as u8);
                let payload = payload.into_iter().chain(addr.into_iter()).collect::<Vec<u8>>();
                socket.write_all(&payload).await?;
            }
            None => {
                println!("! Target \"{}\" still doesn't exist", target);
                socket.write_u8(0).await?;
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
