## Remote Dictionary (KV store)

Endpoints:
- get(key: str)
- set(key: str, val: str)
- get_stats

Components:
- CLI bin
- client lib
- server bin

Implementation:
- Rust, Tokio
- JSON frames over async TCP
- Persy as persistent DB
