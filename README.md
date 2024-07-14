# Running
```bash
nix run github:justryanw/net-phys -- host-server # Client and server
#                                    client
#                                    server

# Run host-server and client to connect to it (-c is client ID)
nix run github:justryanw/net-phys -- host-server & nix run github:justryanw/net-phys -- client -c 1
```
