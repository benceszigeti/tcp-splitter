<h1 align="center">tcp-clone</h1>
<div align="center">
    <strong>TCP proxy server with ability to send client upstream to observers.</strong>
</div>

<br />

<div align="center">
    <a href="https://travis-ci.com/benceszigeti/tcp-clone/">
        <img src="https://img.shields.io/travis/benceszigeti/tcp-clone?style=flat-square" alt="Build status" />
    </a>
    <a href="https://crates.io/crates/tcp-clone/">
        <img src="https://img.shields.io/crates/v/tcp-clone.svg?style=flat-square" alt="crates.io version" />
    </a>
    <a href="https://github.com/benceszigeti/tcp-clone/releases/">
        <img src="https://img.shields.io/github/release/benceszigeti/tcp-clone.svg?style=flat-square" alt="Latest release" />
    </a>
    <a href="LICENSE">
        <img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License" />
    </a>
</div>

## Architecture
```
                               Simple proxy                                   ...with client TX observers:
#########################################################################                                 
#                                                                       #                                 
#    TCP client                `tcp-clone`                TCP server    #                 Observer 0..N   
#  +------------+            +-------------+            +------------+  #              +--------------+   
#  |            |            |             |            |            |  #              |              |--+
#  |    connect |----------->| accept      |            |            |  #              |              |  |
#  |            |            |     connect |----------->| accept     |  #              |              |  |
#  |            |            |             |\----------------------------------------->| accept       |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |   RX/TX    |             |   client   |            |  #              |              |  |
#  |     RX, TX |<---------->| RX, TX      |   RX/TX    |            |  #              |              |  |
#  |            |            |      RX, TX |<---------->| RX, TX     |  #              |              |  |
#  |            |            |             |\ client TX |            |  #              |              |  |
#  |            |            |             | \---------------------------------------->| RX           |  |
#  |            |            |             |            |            |  #  observer TX |              |  |
#  |            |            |     dropped |<------------------------------------------| TX           |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |             |            |            |  #              |              |  |
#  | disconnect |-----x----->| disconnect  |            |            |  #              |              |  |
#  |         #1 |            |  disconnect |------x---->|            |  #              |              |  |
#  |            |            |             |\           |            |  #              |              |  |
#  |            |            |             | ------------------x---------------------->| disconnect   |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |  disconnect |<-----x-----| disconnect |  #              |              |  |
#  | disconnect |<----x------| disconnect  |            | #2         |  #              |              |  |
#  |            |            |  disconnect |-------------------x---------------------->| disconnect   |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |             |            |            |  #              |              |  |
#  |            |            |  disconnect |<------------------x-----------------------| disconnect   |  |
#  |            |            |             |            |            |  #              | #3           |  |
#  +------------+            +-------------+            +------------+  #              +--------------+  |
#########################################################################               +----------------+
```

## Installation

### From source

With [cargo](https://rustup.rs/) installed run:

```sh
$ cargo install tcp-clone
```

### Pre-builds

Download a [released](https://github.com/benceszigeti/tcp-clone/releases/) version.

## Usage

```sh
$ tcp-clone --help
```

## Configuration file

### Example

```toml
[[tcp_clone]]

  [tcp_clone.server]
  listen_addr = "127.0.0.1:1202"

  [tcp_clone.target]
  addr = "127.0.0.1:5000"

  [[tcp_clone.observer]]
  addr = "127.0.0.1:6000"

  [[tcp_clone.observer]]
  addr = "127.0.0.1:7000"

# Add more servers:
#
#[[tcp_clone]]
#
#  [tcp_clone.server]
#  listen_addr = "127.0.0.1:1111"
#
#  [tcp_clone.target]
#  addr = "127.0.0.1:3333"
#
#  [[tcp_clone.observer]]
#  addr = "127.0.0.1:5555"
```

## Demo

### With `iperf`

```sh
$ tcp-clone tcp-clone.toml                        # `tcp-clone` server
$ iperf -s -p 5000 -b 800Mbits/sec                # Target server
$ iperf -s -p 6000 -b 1Gbytes/sec                 # Observer #1
$ iperf -s -p 7000 -b 500Mbits/sec                # Observer #2
$ iperf -c 127.0.0.1 -p 1202 -n 250Mbytes -P 4
```

### With `netcat`

```sh
$ tcp-clone tcp-clone.toml
$ nc -l -p 5000
$ nc -l -p 6000
$ nc -l -p 7000
$ nc 127.0.0.1 1202
$ # ...and now type into the netcat instances...
```

## License

<div align="center">
<sup>
This project is licensed under the <a href="LICENSE">MIT license</a>.
<br/>
Copyright &copy; 2019 Bence SZIGETI &lt;bence.szigeti@gohyda.com&gt;
</sup>
</div>
