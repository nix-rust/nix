Fixed `SockaddrStorage::len` returning the size of `sockaddr_storage` for IPv4
and IPv6 addresses on platforms without `sa_len`, which illumos rejects in
`bind`, `connect` and `sendto`.
