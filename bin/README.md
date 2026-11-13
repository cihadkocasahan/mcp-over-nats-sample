# Local demo broker

The Windows x64 `nats-server.exe` in this directory is the unmodified NATS
Server v2.15.3 release from the official project. Its upstream README and LICENSE
are included alongside it. No local configuration or credentials are bundled.

- Release: https://github.com/nats-io/nats-server/releases/tag/v2.15.3
- Archive: https://github.com/nats-io/nats-server/releases/download/v2.15.3/nats-server-v2.15.3-windows-amd64.zip
- Published checksums: https://github.com/nats-io/nats-server/releases/download/v2.15.3/SHA256SUMS
- Archive SHA-256: `b202c46689a56b17e88939b6c53a25294021b000b9c7225209bcb1bbda347bad`
- Executable SHA-256: `9eb7baebf9e77837a1b5da965d5f0930a3e3be07f78a999daa255e0f6e2117fb`

The archive checksum was compared with the official checksum file, and the bundled
executable was compared byte for byte with the archive member on 13 November 2026.
The duplicate ZIP is kept out of Git. For Linux/macOS, download the corresponding
build from the same upstream release.

Run a demo broker on this machine only:

```powershell
.\bin\nats-server-v2.15.3-windows-amd64\nats-server.exe -a 127.0.0.1 -p 4222
```
