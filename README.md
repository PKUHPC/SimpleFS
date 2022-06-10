# SimpleFS

An implementation for parallel file system with rust

## Introduction

---

This project is based on GekkoFS, an open source burst buffer file system.

Solution to modules in this project:

- System call interception: syscall_intercept
- Async framework: Tokio
- RPC: grpc-rs (grpcio)
- RDMA: custom implementation based on libibverbs and rdmacm (used Rust wrapper provided by rdma-sys)
- Metadata storage: RocksDB
- Data chunk storage: Local filesystem with std file IO

## Dependencies (apt package name)

----

- protobuf-compiler
- libssl-dev
- libcapstone-dev
- (RDMA library installed by your ib adapter driver, including libibverbs, librdmacm, .etc)

## Todo

---

- combine RDMA with Rust async
- optimize metadata read
- apply RDMA to other operation
- ...

## How to use

----

#### Client:

I. Execute command below:

```shell
cd wrapper
./build.sh # if shell script (clean.sh, build_intercept.sh, build.sh) can not get executed, use 'chmod 777 $name_of_script$' to fix that
```

II. Then if build is successfully finished, these file will be added to wrapper folder

- libsfs_client.so
- libsyscall_intercept.so
- libsyscall_intercept.so.0
- libsyscall_intercept.so.1.0

III. You need to copy these file to the folder you want to install the client.

IV. To use the client, a 'hostfile' needs to be place in the working directory, which should contain the address of all server nodes. Content in 'hostfile' should be like:

```
localhost 127.0.0.1
servername 192.168.1.2
$node_hostname$ $node_ipv4_address$
```

V. If you want to execute a command with client enabled, you need to set up 'LD_PRELOAD' and 'LD_LIBRARY_PATH' environment variable.

``` shell
LD_LIBRARY_PATH=$path_to_syscall_intercept$ LD_PRELOAD=$path_to_libsfs_client$ ./your_application
```

#### Server:

I. Execute command below:

```shell
cd sfs_server
cargo build
```

II. Then you have 2 methods to start server

- method 1: use cargo

  ```shell
  cargo run
  ```

- method 2: use executable binary

  After 'cargo build' a executable file named 'sfs_server' will be generated in "sfs_server/target/debug", you can just run the executable file to start server

III. To make sure that server can work properly, a 'hostfile' and a 'config.json' are needed to be placed in the working directory. The 'hostfile' is just like what is needed in client. The 'config.json' should be something like this:

```json
{
    "mountdir": "$mountdir",
    "rootdir": "$rootdir",
    "metadir": "$metadir",
    "hosts_file": "$hostfile_path",
    "listen": "$server_listen_address",
    "output": true // or false
}
```

"rootdir" is the position that server store data chunks and "metadir" points to the folder of metadata database. "hosts_file" describes the location of 'hostfile'. And the server will listen on the address from "listen" field. If "output" is set to "true", debug info will be printed on std output.

"moutdir" is the mount directory of client, this should be set by client. But for the convenience in development, client will fetch this location from server. This may get changed in the future.