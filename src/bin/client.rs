use std::error::Error;
use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use quic_example::{
    CLIENT_ADDR, HELLO_STREAM_ID, MAX_DATAGRAM_SIZE, SERVER_ADDR, make_client_config,
};

fn main() -> Result<(), Box<dyn Error>> {
    let client_addr: SocketAddr = CLIENT_ADDR.parse()?;
    let server_addr: SocketAddr = SERVER_ADDR.parse()?;

    let socket = UdpSocket::bind(client_addr)?;
    println!("Client bound on {client_addr}, connecting to {server_addr}");

    let mut config = make_client_config()?;
    let scid = quiche::ConnectionId::from_ref(&[0xba; quiche::MAX_CONN_ID_LEN]);

    let local_addr = socket.local_addr()?;
    let mut conn = quiche::connect(
        Some("localhost"),
        &scid,
        local_addr,
        server_addr,
        &mut config,
    )?;

    let mut out = [0u8; MAX_DATAGRAM_SIZE];
    let mut buf = [0u8; 65535];
    let mut stream_buf = [0u8; 1024];
    let mut response = String::new();
    let mut request_sent = false;
    let deadline = Instant::now() + Duration::from_secs(10);

    loop {
        if Instant::now() > deadline {
            return Err("client timed out waiting for server response".into());
        }

        flush_outbound(&mut conn, &socket, &mut out)?;

        if conn.is_established() && !request_sent {
            conn.stream_send(HELLO_STREAM_ID, b"hello from client", true)?;
            request_sent = true;
            flush_outbound(&mut conn, &socket, &mut out)?;
        }

        let timeout = conn.timeout();
        socket.set_read_timeout(Some(timeout.unwrap_or(Duration::from_millis(200))))?;

        let (len, from) = match socket.recv_from(&mut buf) {
            Ok(v) => v,

            Err(err)
                if err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut =>
            {
                if timeout.is_some() {
                    conn.on_timeout();
                }

                continue;
            }

            Err(err) => return Err(err.into()),
        };

        let recv_info = quiche::RecvInfo {
            from,
            to: socket.local_addr()?,
        };

        if let Err(err) = conn.recv(&mut buf[..len], recv_info) {
            eprintln!("recv failed: {err:?}");
            continue;
        }

        for stream_id in conn.readable() {
            while let Ok((read, fin)) = conn.stream_recv(stream_id, &mut stream_buf) {
                response.push_str(std::str::from_utf8(&stream_buf[..read])?);

                if fin {
                    println!("Client received: {response}");
                    return Ok(());
                }
            }
        }

        if conn.is_closed() {
            return Err("connection closed before full response".into());
        }
    }
}

fn flush_outbound(
    conn: &mut quiche::Connection,
    socket: &UdpSocket,
    out: &mut [u8],
) -> Result<(), Box<dyn Error>> {
    loop {
        let (written, send_info) = match conn.send(out) {
            Ok(v) => v,
            Err(quiche::Error::Done) => break,
            Err(err) => return Err(err.into()),
        };

        socket.send_to(&out[..written], send_info.to)?;
    }

    Ok(())
}
