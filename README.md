# Running
Commands (host-server, client, server)
```bash
nix run github:justryanw/net-phys -- host-server
```

Run a host-server and a client to connect to it (-c is client ID)
```bash
nix run github:justryanw/net-phys -- host-server & nix run github:justryanw/net-phys -- client -c 1
```
