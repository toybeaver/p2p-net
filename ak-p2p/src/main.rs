use tokio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;
use ak_lib::{AddressInfo, AddressRequest, StreamObject};

const DEFAULT_SV_ADDR: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() -> std::io::Result<()>
{
    let self_addr: SocketAddr;
    let target_addr: Option<SocketAddr>;
    {
        let mut stream = TcpStream::connect(DEFAULT_SV_ADDR).await?;

        AddressRequest::new("jansen", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaabbc")
            .write_to_stream(&mut stream)
            .await?;

        self_addr = AddressInfo::read_from_stream(&mut stream).await?.to_socket_addr();
        println!("My address: {}", self_addr);

        let target_addr_info = AddressInfo::read_from_stream(&mut stream).await?;

        if target_addr_info.is_empty() {
            println!("! Target doesn't exist yet!");
            target_addr = None;
        } else {
            target_addr = Some(target_addr_info.to_socket_addr());
            println!("! Target found at addr: {}!", target_addr.unwrap());
        }
    }

    match target_addr {
        None => {
            println!("! Starting listening at {}", self_addr);
            let mut listener = TcpListener::bind(self_addr).await?;
            let _ = listener.accept().await?;
        }
        Some(addr) => {
            println!("! try connecting to: {}...", addr);
            let mut stream = TcpStream::connect(addr).await?;
        },
    }


    Ok(())
}
