# nftables-ban

This is the ip-ban component of [network-bans](https://github.com/MatthewCash/network-bans). The application listens for webhook requests to add to, remove from, or check if an address is in the nftables blacklist address set.

## Example Configuration

```toml
[database]
file_path = "/var/nftables-ban/db.sqlite"
table_name = "banned_addrs"

[nftables]
set_table = "filter"
set_name = "blacklist" # set blacklist { type ipv4_addr; }

[webhook]
addr = "0.0.0.0:9378" # either use a reverse proxy or vpn for encryption
auth_token = "3b395ed3b325251570061c786b7fd5b78e0be9569e032f93546920327e631d82" # keep this secret
```
