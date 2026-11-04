Fixed `tcgetattr` dropping `c_cflag` bits that have no named `ControlFlags`
value. On illumos these bits hold the baud rate, so writing the settings back
hung up the terminal.
