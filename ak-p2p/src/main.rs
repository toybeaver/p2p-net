use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};
use std::{net::SocketAddr, str::FromStr};

const DEFAULT_SV_ADDR: &str = "0.0.0.0:3000";

trait P2pExtSocket {
    async fn write_with_padding(&mut self, buf: &[u8], size: usize) -> std::io::Result<()>;
}

impl P2pExtSocket for TcpStream {
    async fn write_with_padding(&mut self, buf: &[u8], size: usize) -> std::io::Result<()>
    {
        let padding = vec![0;size-buf.len()];
        self.write_all(buf).await?;
        self.write_all(&padding).await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()>
{
    let mut self_addr: SocketAddr;
    let mut target_addr: Option<SocketAddr>;
    {
        let mut stream = TcpStream::connect(DEFAULT_SV_ADDR).await?;

        let name = String::from("ana");
        stream.write_with_padding(name.as_bytes(), 50).await?;

        let name = String::from("jansen");
        stream.write_with_padding(name.as_bytes(), 50).await?;

        let size = stream.read_u8().await?;
        let mut buf = vec![0;size as usize];
        stream.read_exact(&mut buf).await?;

        let addr = String::from_utf8_lossy(&buf);
        self_addr = SocketAddr::from_str(&addr).unwrap();
        println!("My address: {}", self_addr);

        let size = stream.read_u8().await?;
        let mut buf = vec![0;size as usize];
        stream.read_exact(&mut buf).await?;

        if size == 0 {
            println!("! Target doesn't exist yet!");
            target_addr = None;
        } else {
            let addr = String::from_utf8_lossy(&buf);
            target_addr = Some(SocketAddr::from_str(&addr).unwrap());
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
