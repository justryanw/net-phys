### Simulate network delay
```bash
# Start
sudo tc qdisc add dev lo root netem delay 100ms
# Change 
sudo tc qdisc change dev lo root netem delay 50ms
# Reset
sudo tc qdisc del dev lo root
```