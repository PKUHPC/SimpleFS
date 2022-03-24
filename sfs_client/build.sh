#! /bin/bash

cargo build
rm ../wrapper/libsfs_client.a
mv target/debug/libsfs_client.a ../wrapper