Add `ControlMessageOwned::SoMark(u32)` so that the `(SOL_SOCKET, SO_MARK)`
ancillary message delivered to receivers with `SO_RCVMARK` (Linux 5.19+) is
decoded into a typed variant instead of falling through to the catch-all
`Unknown` arm. This removes the per-recv `Vec<u8>` allocation that the
catch-all path entails.
