# conver

Conver is a chat server that works by maintaining TCP sockets with each client, in their own thread, allowing concurrent and bidirectional communications. If the receiving client is disconnected, the chat is kept an in-memory queue. After the receiving client connects, the pending chat is sent immediately.

This project is written for learning purposes only, as it's missing desirable features for production use, such as authentication, persistence, and encryption.

## Protocol

The server accepts fixed-sized (4096 bytes) messages adhering a binary protocol based on `Chat`, `Join`, and `Leave` structs in [src/message.rs](src/message.rs), de/serialized with [bincode](https://github.com/servo/bincode).

The [demo client](src/bin/client.rs) helps converting human-readable commands below to messages in the right binary format, before sending them to the server.

1. Chat

Send a chat to an individual user or a group comprised of users. With the demo client:

```
CHAT [USER/GROUP] <username/groupname>
> <chat body>
```

2. Join

Asks the server to add you to a group, thus receiving all messages sent to the group. With the demo client:

```
JOIN <groupname>
```

3. Leave

Asks the server to remove you from a group. With the demo client:

```
LEAVE <groupname>
```

## Usage

Server:

```
$ cargo run --bin server -- --help
```

(Demo) client:

```
$ cargo run --bin client -- --help
```

## Examples

Starting the server:

```
$ cargo run --bin server
```

Connecting to the server as Alice, then sending a chat to Bob:

```
$ cargo run --bin client -- -u alice

CHAT USER bob
> Hello, Bob!
```

Connecting as Bob, and receiving the chat from Alice:

```
$ cargo run --bin client -- -u bob

# alice: Hello, Bob!
```

Connecting as Eve, joining a group, and sending a message to it:

```
$ cargo run --bin client -- -u eve

JOIN GROUP bar

CHAT GROUP bar
> Hi y'all.
```

Suppose Alice and Bob have joined the group beforehand, they will receive the chat like so:

```
#[bar] eve: Hi y'all.
```

## Acknowledgments

Conver's architecture is inspired from a random discussion with [@rizkiihza](https://github.com/rizkiihza), along with some inputs from [@nieltg](https://github.com/nieltg).
