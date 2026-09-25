tg-ipc-abi          (zero deps, czysty ABI)
    ↑
tg-cap-core         (depends: tg-ipc-abi)
    ↑
tg-cap-table        (depends: tg-cap-core)
    ↑
tg-ipc-ring         (depends: tg-ipc-abi)
    ↑
tg-ipc-channel      (depends: tg-ipc-abi, tg-ipc-ring, tg-cap-core)
    ↑
tg-shmem            (depends: tg-cap-core)
    ↑
tg-ipc-dispatch     (depends: tg-ipc-abi, tg-cap-core, tg-cap-table)