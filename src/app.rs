use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

use bevy::asset::ron;
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::state::app::StatesPlugin;
use bevy::winit::{WakeUp, WinitPlugin};
use bevy::DefaultPlugins;
use clap::{Parser, ValueEnum};
use lightyear::prelude::client::ClientConfig;
use lightyear::prelude::*;
use lightyear::prelude::{client, server};
use lightyear::server::config::ServerConfig;
use lightyear::shared::log::add_log_layer;
use lightyear::transport::LOCAL_SOCKET;
use serde::{Deserialize, Serialize};

use crate::settings::*;
use crate::shared::{shared_config, SERVER_REPLICATION_INTERVAL};

#[derive(Parser, PartialEq, Debug)]
pub enum Cli {
    HostServer {
        #[arg(short, long, default_value = None)]
        client_id: Option<u64>,
    },
    Server,
    Client {
        #[arg(short, long, default_value = None)]
        client_id: Option<u64>,
        #[arg(short, long, default_value = None)]
        server_ip: Option<Ipv4Addr>,
    },
}

impl Default for Cli {
    fn default() -> Self {
        cli()
    }
}

pub fn cli() -> Cli {
    Cli::parse()
}

pub enum Apps {
    Client {
        app: App,
        config: ClientConfig,
    },
    Server {
        app: App,
        config: ServerConfig,
    },
    HostServer {
        app: App,
        client_config: ClientConfig,
        server_config: ServerConfig,
    },
}

impl Apps {
    pub fn new(settings: Settings, cli: Cli) -> Self {
        match cli {
            Cli::HostServer { client_id } => {
                let client_net_config = client::NetConfig::Local {
                    id: client_id.unwrap_or(settings.client.client_id),
                };
                let (app, client_config, server_config) =
                    combined_app(settings, vec![], client_net_config);
                Apps::HostServer {
                    app,
                    client_config,
                    server_config,
                }
            }
            Cli::Server => {
                let (app, config) = server_app(settings, vec![]);
                Apps::Server { app, config }
            }
            Cli::Client {
                client_id,
                server_ip,
            } => {
                let client_id = client_id.unwrap_or(settings.client.client_id);
                let server_ip = server_ip.unwrap_or(settings.client.server_addr).into();

                let net_config = get_client_net_config(&settings, client_id, server_ip);
                let (app, config) = client_app(settings, net_config);
                Apps::Client { app, config }
            }
        }
    }

    pub fn with_server_replication_send_interval(mut self, replication_interval: Duration) -> Self {
        self.update_lightyear_client_config(|cc: &mut ClientConfig| {
            cc.shared.server_replication_send_interval = replication_interval
        });
        self.update_lightyear_server_config(|sc: &mut ServerConfig| {
            // the server replication currently needs to be overwritten in both places...
            sc.shared.server_replication_send_interval = replication_interval;
            sc.replication.send_interval = replication_interval;
        });
        self
    }

    pub fn add_lightyear_plugins(&mut self) -> &mut Self {
        match self {
            Apps::Client { app, config } => {
                app.add_plugins(client::ClientPlugins {
                    config: config.clone(),
                });
            }
            Apps::Server { app, config } => {
                app.add_plugins(server::ServerPlugins {
                    config: config.clone(),
                });
            }
            Apps::HostServer {
                app,
                client_config,
                server_config,
            } => {
                app.add_plugins(client::ClientPlugins {
                    config: client_config.clone(),
                });
                app.add_plugins(server::ServerPlugins {
                    config: server_config.clone(),
                });
            }
        }
        self
    }

    pub fn add_user_plugins(
        &mut self,
        client_plugin: impl Plugin,
        server_plugin: impl Plugin,
        shared_plugin: impl Plugin + Clone,
    ) -> &mut Self {
        match self {
            Apps::Client { app, .. } => {
                app.add_plugins((client_plugin, shared_plugin));
            }
            Apps::Server { app, .. } => {
                app.add_plugins((server_plugin, shared_plugin));
            }
            Apps::HostServer { app, .. } => {
                app.add_plugins((client_plugin, server_plugin, shared_plugin));
            }
        }
        self
    }

    pub fn update_lightyear_client_config(
        &mut self,
        f: impl FnOnce(&mut ClientConfig),
    ) -> &mut Self {
        match self {
            Apps::Client { config, .. } => {
                f(config);
            }
            Apps::Server { config, .. } => {}
            Apps::HostServer { client_config, .. } => {
                f(client_config);
            }
        }
        self
    }

    pub fn update_lightyear_server_config(
        &mut self,
        f: impl FnOnce(&mut ServerConfig),
    ) -> &mut Self {
        match self {
            Apps::Client { config, .. } => {}
            Apps::Server { config, .. } => {
                f(config);
            }

            Apps::HostServer { server_config, .. } => {
                f(server_config);
            }
        }
        self
    }

    pub fn run(self) {
        match self {
            Apps::Client { mut app, .. } => {
                app.run();
            }
            Apps::Server { mut app, .. } => {
                app.run();
            }
            Apps::HostServer { mut app, .. } => {
                app.run();
            }
        }
    }
}

fn client_app(settings: Settings, net_config: client::NetConfig) -> (App, ClientConfig) {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.build());
    let client_config = ClientConfig {
        shared: shared_config(Mode::Separate),
        net: net_config,
        ..default()
    };
    (app, client_config)
}

fn server_app(
    settings: Settings,
    extra_transport_configs: Vec<server::ServerTransport>,
) -> (App, ServerConfig) {
    let mut app = App::new();
    if !settings.server.headless {
        app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>());
    } else {
        app.add_plugins((
            // TODO: cannot use MinimalPlugins because avian requires render/assets plugin
            // MinimalPlugins,
            // StatesPlugin,
            DefaultPlugins.build().disable::<LogPlugin>(),
        ));
    }

    let mut net_configs = get_server_net_configs(&settings);
    let extra_net_configs = extra_transport_configs.into_iter().map(|c| {
        build_server_netcode_config(settings.server.conditioner.as_ref(), &settings.shared, c)
    });
    net_configs.extend(extra_net_configs);
    let server_config = ServerConfig {
        shared: shared_config(Mode::Separate),
        net: net_configs,
        replication: ReplicationConfig {
            send_interval: SERVER_REPLICATION_INTERVAL,
            ..default()
        },
        ..default()
    };
    (app, server_config)
}

fn combined_app(
    settings: Settings,
    extra_transport_configs: Vec<server::ServerTransport>,
    client_net_config: client::NetConfig,
) -> (App, ClientConfig, ServerConfig) {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build());

    // Server config
    let mut net_configs = get_server_net_configs(&settings);
    let extra_net_configs = extra_transport_configs.into_iter().map(|c| {
        build_server_netcode_config(settings.server.conditioner.as_ref(), &settings.shared, c)
    });
    net_configs.extend(extra_net_configs);
    let server_config = ServerConfig {
        shared: shared_config(Mode::HostServer),
        net: net_configs,
        replication: ReplicationConfig {
            send_interval: SERVER_REPLICATION_INTERVAL,
            ..default()
        },
        ..default()
    };

    // Client config
    let client_config = ClientConfig {
        shared: shared_config(Mode::HostServer),
        net: client_net_config,
        ..default()
    };
    (app, client_config, server_config)
}
