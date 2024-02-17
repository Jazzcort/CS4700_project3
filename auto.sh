#! /bin/bash

cargo build

cp target/debug/project3-bgp-router* test/

cd test

rm project3-bgp-router.d

mv project3-bgp-router 4700router

python3 test