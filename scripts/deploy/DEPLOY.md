# Delpoy scripts

## Native with systemd
Build takt for the target machine. For example linux/arm for the raspberry pi.
- Install cross: https://github.com/cross-rs/cross
- Build:
cross build --release --target armv7-unknown-linux-gnueabihf

Create a systemd file:
see [example](takt.service)
