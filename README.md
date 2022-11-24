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
- Bincode or JSON frames over async TCP
- Persy as persistent DB with async cache on top
- Clap as CLI args parser
- Criterion for benchmarking

TODO:
* [-] select on cancel for clean shutdown (i.e. propagate ctrlc hook)
* [-] spawn_blocking the DB calls
* [-] the async cache needs LRU eviction and size limit
* [-] convert stats counting to atomic counters and write the stats to DB every N-requests
* [-] the DB interface needs to be more generic to support swapping engines; Sled DB might be a better alternative than Persy  
* fix incoherency in the async DB/cache i.e. need transactions and locking   
* [-] task opt: bloom filter
* [-] reduce string copies; clean-up generics where needed
* [-] connection code needs to be more generic, right now the test code is duplicate for bincode and json

DONE:
* [+] task req: collect stats 
