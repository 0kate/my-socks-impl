use std::io::prelude::*;
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};

#[derive(Debug)]
struct Request {
    version: u8,
    command: u8,
    dst_port: u16,
    // dst_addr: u32,
    dst_addr: Ipv4Addr,
    id: Vec<u8>,
}

impl Request {
    fn new(version: u8, command: u8, dst_port: u16, dst_addr: u32, id: Vec<u8>) -> Self {
        Self {
            version,
            command,
            dst_port,
            dst_addr: Ipv4Addr::from(dst_addr),
            id,
        }
    }
}

#[derive(Debug)]
struct Response {
    version: u8,
    rep_code: u8,
    dst_port: u16,
    dst_addr: Ipv4Addr,
}

impl Response {
    fn new(version: u8, rep_code: u8, dst_port: u16, dst_addr: u32) -> Self {
        Self {
            version,
            rep_code,
            dst_port,
            dst_addr: Ipv4Addr::from(dst_addr),
        }
    }

    fn to_bytes(&self) -> [u8; 8] {
        let addr_u32: u32 = self.dst_addr.into();
        [
            self.version,
            self.rep_code,
            (self.dst_port >> 8 & 0x00ff) as u8,
            (self.dst_port & 0x00ff) as u8,
            (addr_u32 >> 24 & 0x000000ff) as u8,
            (addr_u32 >> 16 & 0x000000ff) as u8,
            (addr_u32 >> 8 & 0x000000ff) as u8,
            (addr_u32 & 0x000000ff) as u8,
        ]
    }
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buf = [0; 8];
    stream.read(&mut buf)?;

    let mut id = Vec::new();
    loop {
        let mut b = [0; 1];
        stream.read(&mut b)?;
        if b[0] == b'\0' {
            break;
        }
        id.extend_from_slice(&b);
    }

    let req = Request::new(
        buf[0],
        buf[1],
        ((buf[2] as u16) << 8) + buf[3] as u16,
        ((buf[4] as u32) << 24) + ((buf[5] as u32) << 16) + ((buf[6] as u32) << 8) + buf[7] as u32,
        id,
    );
    println!("received: {:02x?}", buf);
    println!("req: {:?}", req);

    let res = Response::new(
        0x00,
        0x5a,
        0x00, // req.dst_port,
        req.dst_addr.into(),
    );
    let res_bytes = res.to_bytes();

    println!("transmit: {:02x?}", res_bytes);
    println!("res: {:?}", res);
    stream.write(&res_bytes)?;

    let mut client = TcpStream::connect(format!("{}:{}", req.dst_addr.to_string(), req.dst_port))?;

    let mut buf = [0; 1024];
    loop {
        let n_read = stream.read(&mut buf)?;
        if n_read < 1 {
            break;
        }
        client.write(&mut buf)?;

        client.read(&mut buf)?;
        stream.write(&mut buf)?;
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:11111")?;

    for stream in listener.incoming() {
        handle_client(stream?)?;
    }

    Ok(())
}
