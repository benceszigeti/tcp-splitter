[![Build status][build-badge]][build-url]
[![Version][version-badge]][version-url]
[![MIT licensed][mit-badge]][mit-url]

[build-badge]: https://travis-ci.com/benceszigeti/tcp-clone.svg?branch=master
[build-url]: https://travis-ci.com/benceszigeti/tcp-clone
[version-badge]: https://img.shields.io/github/release/benceszigeti/tcp-clone.svg
[version-url]: https://github.com/benceszigeti/tcp-clone/releases/latest
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE
```
             .                                          oooo                                  
           .o8                                          `888                                  
         .o888oo  .ooooo.  oo.ooooo.           .ooooo.   888   .ooooo.  ooo. .oo.    .ooooo.  
           888   d88' `"Y8  888' `88b         d88' `"Y8  888  d88' `88b `888P"Y88b  d88' `88b 
           888   888        888   888 8888888 888        888  888   888  888   888  888ooo888 
           888 . 888   .o8  888   888         888   .o8  888  888   888  888   888  888    .o 
           "888" `Y8bod8P'  888bod8P'         `Y8bod8P' o888o `Y8bod8P' o888o o888o `Y8bod8P' 
                            888                                                               
                           o888o                                                              
=========================================================================================================
Project name: tcp-clone
Description:  TCP proxy server with ability to copy the client upstream to observers.

WARNING:      The application is highly parallelized. Slow receivers can increase memory usage.
=========================================================================================================


ARCHITECTURE:
=============
                              Simple proxy                                   ...with client TX observers:
########################################################################                                 
#                                                                      #                                 
#   TCP client                `tcp-clone`                TCP server    #                 Observer 0..N   
# +------------+            +-------------+            +------------+  #              +--------------+   
# |            |            |             |            |            |  #              |              |--+
# |    connect |----------->| accept      |            |            |  #              |              |  |
# |            |            |     connect |----------->| accept     |  #              |              |  |
# |            |            |             |\----------------------------------------->| accept       |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |             |            |            |  #              |              |  |
# |            |   RX/TX    |             |   client   |            |  #              |              |  |
# |     RX, TX |<---------->| RX, TX      |   RX/TX    |            |  #              |              |  |
# |            |            |      RX, TX |<---------->| RX, TX     |  #              |              |  |
# |            |            |             |\ client TX |            |  #              |              |  |
# |            |            |             | \---------------------------------------->| RX           |  |
# |            |            |             |            |            |  #  observer TX |              |  |
# |            |            |     dropped |<------------------------------------------| TX           |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |             |            |            |  #              |              |  |
# | disconnect |-----x----->| disconnect  |            |            |  #              |              |  |
# |         #1 |            |  disconnect |------x---->|            |  #              |              |  |
# |            |            |             |\           |            |  #              |              |  |
# |            |            |             | ------------------x---------------------->| disconnect   |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |  disconnect |<-----x-----| disconnect |  #              |              |  |
# | disconnect |<----x------| disconnect  |            | #2         |  #              |              |  |
# |            |            |  disconnect |-------------------x---------------------->| disconnect   |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |             |            |            |  #              |              |  |
# |            |            |  disconnect |<------------------x-----------------------| disconnect   |  |
# |            |            |             |            |            |  #              | #3           |  |
# +------------+            +-------------+            +------------+  #              +--------------+  |
########################################################################               +----------------+

                             This project is licensed under the MIT license.
                       Copyright (c) 2019 Bence SZIGETI <bence.szigeti@gohyda.com>

=========================================================================================================
```
```
CONFIG:
=======
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

Try it with iperf:
==================

$ ./tcp-clone abovecfg.toml                       # `tcp-clone` server
$ iperf -s -p 5000 -b 800Mbits/sec                # Target server
$ iperf -s -p 6000 -b 1Gbytes/sec                 # Observer #1
$ iperf -s -p 7000 -b 500Mbits/sec                # Observer #2
$ iperf -c 127.0.0.1 -p 1202 -n 250Mbytes -P 4    # Run

Try it with netcat:
==================

$ ./tcp-clone abovecfg.toml
$ nc -l -p 5000
$ nc -l -p 6000
$ nc -l -p 7000
$ nc 127.0.0.1 1202

...and now type into the netcat instances...
```
