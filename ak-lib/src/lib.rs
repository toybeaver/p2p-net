#![allow(async_fn_in_trait)]

use std::borrow::Cow;
use std::net::{SocketAddr, IpAddr, Ipv4Addr, Ipv6Addr};

use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};


pub fn bake_buffer<T>(src: &T) -> &[u8]
{
    unsafe { std::slice::from_raw_parts(src as *const T as *const u8, size_of::<T>()) }
}

pub fn str_to_byte_arr(target: &mut [u8], s: &str, max_len: usize)
{
    let buf = s.as_bytes();
    let len = std::cmp::min(buf.len(), max_len);
    target[0..len].copy_from_slice(&buf[0..len]);
}


pub trait StreamObject: Sized {
    // TODO think on how to generalize this
    fn from_byte_slice(buf: &[u8]) -> std::io::Result<Self>;

    async fn write_to_stream(&self, stream: &mut TcpStream) -> std::io::Result<()>
    {
        stream.write_all(bake_buffer(self)).await
    }

    async fn read_from_stream(stream: &mut TcpStream) -> std::io::Result<Self>
    {
        // I have no idea if it's possible to keep this in the stack but tbh
        // it doesn't matter that much here
        let mut buf = vec![0; size_of::<Self>()];
        stream.read_exact(&mut buf).await?;
        Ok(Self::from_byte_slice(&buf)?)
    }
}


#[repr(C, packed)]
pub struct AddressRequest {
    name: [u8;50],
    target: [u8;50],
}

impl AddressRequest {
    pub fn new(name_str: &str, target_str: &str) -> Self
    {
        let mut name: [u8;50] = [0;50];
        str_to_byte_arr(&mut name, name_str, 50);

        let mut target: [u8;50] = [0;50];
        str_to_byte_arr(&mut target, target_str, 50);

        Self { name, target }
    }


    pub fn peer_name(&self) -> Cow<'_, str>
    {
        String::from_utf8_lossy(&self.name)
    }

    pub fn peer_target(&self) -> Cow<'_, str>
    {
        String::from_utf8_lossy(&self.target)
    }

}

impl StreamObject for AddressRequest {
    fn from_byte_slice(buf: &[u8]) -> std::io::Result<Self>
    {
        if buf.len() != size_of::<Self>() {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        }

        let mut name = [0;50];
        name.copy_from_slice(&buf[0..50]);

        let mut target = [0;50];
        target.copy_from_slice(&buf[50..100]);

        Ok(Self { name, target })
    }
}


#[repr(C, packed)]
pub struct AddressInfo {
    addr_type: u8, // 0x00 => none ;; 0x01 => ipv4 ;; 0x02 => ipv6
    addr: [u8;16],
    port: [u8;2],
}

impl AddressInfo {
    pub fn empty() -> Self
    {
        Self {
            addr_type: 0x00,
            addr: [0;16],
            port: [0;2],
        }
    }

    pub fn from_socket_addr(s_addr: &SocketAddr) -> Self
    {
        let port = s_addr.port().to_be_bytes();

        let addr_type;
        let mut addr = [0;16];

        match s_addr {
            SocketAddr::V4(v4) => {
                addr_type = 0x01;
                addr[0..4].copy_from_slice(&v4.ip().octets());
            }
            SocketAddr::V6(v6) => {
                addr_type = 0x02;
                addr.copy_from_slice(&v6.ip().octets());
            }
        };
        Self { addr_type, addr, port }
    }

    pub fn is_empty(&self) -> bool
    {
        self.addr_type == 0x00
    }

    pub fn to_socket_addr(&self) -> SocketAddr
    {
        let port = u16::from_be_bytes(self.port);
        match self.addr_type {
            0x01 => {
                let addr = self.addr[0..4].try_into().unwrap();
                SocketAddr::new(IpAddr::V4(Ipv4Addr::from_octets(addr)), port)
            }
            0x02 =>  SocketAddr::new(IpAddr::V6(Ipv6Addr::from_octets(self.addr)), port),

            // TODO: proper error handling
            _ => unreachable!()
        }
    }

    pub async fn read_from_stream(stream: &mut TcpStream) -> std::io::Result<Self>
    {
        let mut buf = [0;size_of::<Self>()];
        stream.read_exact(&mut buf).await?;
        Ok(Self::from_byte_slice(&buf)?)
    }

    pub async fn write_to_stream(&self, stream: &mut TcpStream) -> std::io::Result<()>
    {
        stream.write_all(bake_buffer(self)).await
    }

}

impl StreamObject for AddressInfo {
    fn from_byte_slice(buf: &[u8]) -> std::io::Result<Self>
    {
        if buf.len() != size_of::<Self>() {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        }

        let addr_type = buf[0];

        let mut addr = [0;16];
        addr.copy_from_slice(&buf[1..17]);

        let mut port = [0;2];
        port.copy_from_slice(&buf[17..19]);

        Ok(Self { addr_type, addr, port })
    }
}