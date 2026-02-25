use std::error::Error;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::time::{Duration, Instant};

use quic_example::certs;
use quic_example::{MAX_DATAGRAM_SIZE, SERVER_ADDR, make_server_config};

fn main() -> Result<(), Box<dyn Error>> {
    let server_addr: SocketAddr = SERVER_ADDR.parse()?;
    let cert_paths = certs::default_local_cert_paths();
    let (cert_paths, created) =
        certs::ensure_localhost_cert_files(&cert_paths.cert_path, &cert_paths.key_path)?;

    if created {
        println!(
            "Generated self-signed certs: {} and {}",
            cert_paths.cert_path.display(),
            cert_paths.key_path.display()
        );
    }

    let socket = UdpSocket::bind(server_addr)?;
    socket.set_read_timeout(Some(Duration::from_millis(250)))?;

    println!("Server listening on {server_addr}");

    let cert_path = cert_paths.cert_path.to_string_lossy().into_owned();
    let key_path = cert_paths.key_path.to_string_lossy().into_owned();
    let mut config = make_server_config(&cert_path, &key_path)?;
    let mut conn: Option<quiche::Connection> = None;

    let mut buf = [0u8; 65535];
    let mut out = [0u8; MAX_DATAGRAM_SIZE];
    let deadline = Instant::now() + Duration::from_secs(30);

    loop {
        if Instant::now() > deadline {
            return Err("server timed out waiting for a client".into());
        }

        let (len, from) = match socket.recv_from(&mut buf) {
            Ok(v) => v,

            Err(err)
                if err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut =>
            {
                continue;
            }

            Err(err) => return Err(err.into()),
        };

        let local_addr = socket.local_addr()?;

        if conn.is_none() {
            let header = quiche::Header::from_slice(&mut buf[..len], quiche::MAX_CONN_ID_LEN)?;
            let new_conn = quiche::accept(&header.dcid, None, local_addr, from, &mut config)?;

            println!("Accepted QUIC connection from {from}");
            conn = Some(new_conn);
        }

        if let Some(connection) = conn.as_mut() {
            let recv_info = quiche::RecvInfo {
                from,
                to: local_addr,
            };

            if let Err(err) = connection.recv(&mut buf[..len], recv_info) {
                eprintln!("recv failed: {err:?}");
                continue;
            }

            let response_sent = if connection.is_established() {
                handle_streams(connection)?
            } else {
                false
            };

            flush_outbound(connection, &socket, &mut out)?;

            if response_sent || connection.is_closed() {
                break;
            }
        }
    }

    Ok(())
}

fn handle_streams(conn: &mut quiche::Connection) -> Result<bool, Box<dyn Error>> {
    let mut stream_buf = [0u8; 1024];

    for stream_id in conn.readable() {
        while let Ok((read, fin)) = conn.stream_recv(stream_id, &mut stream_buf) {
            let request = std::str::from_utf8(&stream_buf[..read])?;
            println!("Server received: {request}");

            if fin {
                conn.stream_send(stream_id, b"hello from quiche server", true)?;
                return Ok(true);
            }
        }
    }

    Ok(false)
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
