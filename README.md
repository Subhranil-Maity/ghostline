# Ghostline

Ghostline is my desktop peer-to-peer chat experiment built with Tauri, React, and Rust. The core idea is simple: keep the UI minimal, keep the networking logic in Rust, and move toward a peer model backed by persistent public-key identity instead of raw socket addresses.

## What I am building

I am building a local-network or direct-peer chat application with these goals:

- start a local TCP server when the app launches
- connect directly to another peer by `host:port`
- exchange a handshake that carries peer identity and capabilities
- derive a stable peer ID from the remote public key
- store peer state and message history on the backend
- push live connection and message updates from Rust into the frontend
- keep the frontend focused on rendering and user interaction

## Current stack

### Frontend

- React 19
- TypeScript 5
- Vite 7
- Tailwind CSS 4
- Tauri JS API

### Backend

- Rust 2021
- Tauri 2
- Tokio
- `uuid`
- `whoami`
- `ed25519-dalek`
- `rand`
- `bs58`
- `dirs`

## Current architecture

At the moment, the app is split into two clear layers:

- `src/`: the React frontend, which handles connection selection, message rendering, message composition, and Tauri event listeners
- `src-tauri/`: the Rust backend, which handles sockets, packet framing, encode/decode, peer identity, and in-memory peer/message state

The backend is the real source of truth. The frontend reads data from Rust commands and reacts to Rust-emitted events.

## Main features that are already in place

- minimal Catppuccin-inspired chat UI
- live message updates through Tauri events
- per-peer chat history fetched from the backend
- sticky-bottom chat scrolling with manual scrollback
- custom themed scrollbars
- length-prefixed packet framing on the TCP transport
- typed packet encode/decode through a bytehandler layer
- handshake packet carrying peer identity
- peer IDs derived from public keys
- persistent local identity loading or generation

## Repository layout

```text
ghostline/
├── src/
│   ├── App.tsx
│   ├── App.css
│   ├── components/
│   └── types/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── lib.rs
│       ├── state.rs
│       ├── peer.rs
│       ├── models/
│       ├── crypto/
│       └── net/
├── package.json
├── vite.config.ts
├── README.md
└── PROJECT_OVERVIEW.md
```

## Important backend concepts

### Peer identity

I now treat identity as a cryptographic concept:

- each local instance has an Ed25519 keypair
- the public key is sent in the handshake
- the peer ID is derived from the public key bytes
- peer state is stored separately from the raw connection object

### Message model

Messages are now structured objects rather than loose strings:

- `uuid`
- `content`
- `timestamp`
- `sender`

That same message object flows from backend state into frontend rendering.

### Packet transport

TCP reads and writes use a length-prefixed frame:

```text
[u32 packet_len][packet bytes]
```

That avoids assuming a single socket read will always contain exactly one packet.

## Frontend behavior

The frontend currently does this:

- shows the local server address
- connects to a remote host
- listens for `ghostline://connection-created`
- listens for `ghostline://message-received`
- fetches chat history for the selected peer
- renders structured messages with sender and timestamp

## Commands I use

### Frontend

```bash
bun run dev
bun run build
```

### Tauri / Rust

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

## Current limitations

A few things are still intentionally unfinished:

- request handling still returns a generic `not implemented` error response
- some crypto helpers are present but not fully used yet
- peer display in the frontend still uses peer IDs rather than richer identity metadata
- message history is in memory only
- connection cleanup and disconnect handling still need more work

## Notes

`PROJECT_OVERVIEW.md` is the detailed architecture document. I use it as the source-of-truth technical overview for the current codebase.
