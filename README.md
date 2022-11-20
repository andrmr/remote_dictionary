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
- Persy as persistent DB with async cache on top

TODO:
- [] the async cache needs LRU eviction and size limit
- [] the DB interface needs to be more generic to support swapping engines; Sled DB might be a better alternative than Persy
- [] task req: collect stats
- [] task opt: bloom filter
