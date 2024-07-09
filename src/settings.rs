use std::net::{Ipv4Addr, SocketAddr};

use bevy::asset::ron;
use bevy::prelude::{default, Resource};
use bevy::utils::Duration;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use bevy::tasks::IoTaskPool;

use lightyear::prelude::client::Authentication;
use lightyear::prelude::client::{SocketConfig, SteamConfig};
use lightyear::prelude::{CompressionConfig, LinkConditionerConfig};

use lightyear::prelude::{client, server};

pub fn read_settings<T: DeserializeOwned>(settings_str: &str) -> T {
    ron::de::from_str::<T>(settings_str).expect("Could not deserialize the settings file")
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ClientTransports {
    Udp,
    Steam {
        app_id: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ServerTransports {
    Udp {
        local_port: u16,
    },
    Steam {
        app_id: u32,
        server_ip: Ipv4Addr,
        game_port: u16,
        query_port: u16,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Conditioner {
    pub(crate) latency_ms: u16,
    pub(crate) jitter_ms: u16,
    pub(crate) packet_loss: f32,
}

impl Conditioner {
    pub fn build(&self) -> LinkConditionerConfig {
        LinkConditionerConfig {
            incoming_latency: Duration::from_millis(self.latency_ms as u64),
            incoming_jitter: Duration::from_millis(self.jitter_ms as u64),
            incoming_loss: self.packet_loss,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerSettings {
    pub(crate) headless: bool,
    pub(crate) inspector: bool,
    pub(crate) conditioner: Option<Conditioner>,
    pub transport: Vec<ServerTransports>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientSettings {
    pub(crate) inspector: bool,
    pub(crate) client_id: u64,
    pub(crate) client_port: u16,
    pub server_addr: Ipv4Addr,
    pub server_port: u16,
    pub(crate) transport: ClientTransports,
    pub(crate) conditioner: Option<Conditioner>,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct SharedSettings {
    pub protocol_id: u64,
    pub private_key: [u8; 32],
    pub(crate) compression: CompressionConfig,
}

#[derive(Resource, Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub client: ClientSettings,
    pub shared: SharedSettings,
}

#[allow(dead_code)]
pub(crate) fn build_server_netcode_config(
    conditioner: Option<&Conditioner>,
    shared: &SharedSettings,
    transport_config: server::ServerTransport,
) -> server::NetConfig {
    let conditioner = conditioner.map_or(None, |c| {
        Some(LinkConditionerConfig {
            incoming_latency: Duration::from_millis(c.latency_ms as u64),
            incoming_jitter: Duration::from_millis(c.jitter_ms as u64),
            incoming_loss: c.packet_loss,
        })
    });
    let netcode_config = server::NetcodeConfig::default()
        .with_protocol_id(shared.protocol_id)
        .with_key(shared.private_key);
    let io_config = server::IoConfig {
        transport: transport_config,
        conditioner,
        compression: shared.compression,
    };
    server::NetConfig::Netcode {
        config: netcode_config,
        io: io_config,
    }
}

pub(crate) fn get_server_net_configs(settings: &Settings) -> Vec<server::NetConfig> {
    settings
        .server
        .transport
        .iter()
        .map(|t| match t {
            ServerTransports::Udp { local_port } => build_server_netcode_config(
                settings.server.conditioner.as_ref(),
                &settings.shared,
                server::ServerTransport::UdpSocket(SocketAddr::new(
                    Ipv4Addr::UNSPECIFIED.into(),
                    *local_port,
                )),
            ),
            ServerTransports::Steam {
                app_id,
                server_ip,
                game_port,
                query_port,
            } => server::NetConfig::Steam {
                steamworks_client: None,
                config: server::SteamConfig {
                    app_id: *app_id,
                    socket_config: server::SocketConfig::Ip {
                        server_ip: *server_ip,
                        game_port: *game_port,
                        query_port: *query_port,
                    },
                    max_clients: 16,
                    ..default()
                },
                conditioner: settings
                    .server
                    .conditioner
                    .as_ref()
                    .map_or(None, |c| Some(c.build())),
            },
        })
        .collect()
}

pub(crate) fn build_client_netcode_config(
    client_id: u64,
    server_addr: SocketAddr,
    conditioner: Option<&Conditioner>,
    shared: &SharedSettings,
    transport_config: client::ClientTransport,
) -> client::NetConfig {
    let conditioner = conditioner.map_or(None, |c| Some(c.build()));
    let auth = Authentication::Manual {
        server_addr,
        client_id,
        private_key: shared.private_key,
        protocol_id: shared.protocol_id,
    };
    let netcode_config = client::NetcodeConfig::default();
    let io_config = client::IoConfig {
        transport: transport_config,
        conditioner,
        compression: shared.compression,
    };
    client::NetConfig::Netcode {
        auth,
        config: netcode_config,
        io: io_config,
    }
}

pub fn get_client_net_config(settings: &Settings, client_id: u64) -> client::NetConfig {
    let server_addr = SocketAddr::new(
        settings.client.server_addr.into(),
        settings.client.server_port,
    );
    let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), settings.client.client_port);
    match &settings.client.transport {
        ClientTransports::Udp => build_client_netcode_config(
            client_id,
            server_addr,
            settings.client.conditioner.as_ref(),
            &settings.shared,
            client::ClientTransport::UdpSocket(client_addr),
        ),
        ClientTransports::Steam { app_id } => client::NetConfig::Steam {
            steamworks_client: None,
            config: SteamConfig {
                socket_config: SocketConfig::Ip { server_addr },
                app_id: *app_id,
            },
            conditioner: settings
                .server
                .conditioner
                .as_ref()
                .map_or(None, |c| Some(c.build())),
        },
    }
}
