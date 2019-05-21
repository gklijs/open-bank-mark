#!/usr/bin/env bash

git submodule update --init
cd frontend
./build-and-copy-frontend.sh
cd ../test
lein uberjar
cd ..
java -jar test/target/test.jar mapping