pub const MAX_DATAGRAM_SIZE: usize = 1350;
pub const HELLO_STREAM_ID: u64 = 0;
pub const CLIENT_ADDR: &str = "127.0.0.1:4444";
pub const SERVER_ADDR: &str = "127.0.0.1:5555";

const ALPN: &[u8] = b"hello-quic";

pub fn make_client_config() -> Result<quiche::Config, quiche::Error> {
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION)?;
    config.verify_peer(false);
    config.set_application_protos(&[ALPN])?;
    configure_transport(&mut config);

    Ok(config)
}

pub fn make_server_config(
    cert_path: &str,
    key_path: &str,
) -> Result<quiche::Config, quiche::Error> {
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION)?;
    config.load_cert_chain_from_pem_file(cert_path)?;
    config.load_priv_key_from_pem_file(key_path)?;
    config.set_application_protos(&[ALPN])?;
    configure_transport(&mut config);

    Ok(config)
}

fn configure_transport(config: &mut quiche::Config) {
    config.set_max_idle_timeout(5_000);
    config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_initial_max_data(1_000_000);
    config.set_initial_max_stream_data_bidi_local(100_000);
    config.set_initial_max_stream_data_bidi_remote(100_000);
    config.set_initial_max_streams_bidi(16);
    config.set_initial_max_streams_uni(16);
    config.set_disable_active_migration(true);
}
