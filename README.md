# Running
Commands (host-server, client, server)
```bash
nix run github:justryanw/net-phys -- host-server
```

Run a host-server and a client to connect to it (-c is client ID, set to 1 since host-server will be 0)
```bash
nix run github:justryanw/net-phys -- host-server & nix run github:justryanw/net-phys -- client -c 1
```

Connect to remote server
```bash
nix run github:justryanw/net-phys -- client -c 1 -s 127.0.0.1
```

Stress test (locally)
```bash
mangohud cargo run --release -- server & parallel mangohud cargo run --release -- client -c ::: {0..15}
```
