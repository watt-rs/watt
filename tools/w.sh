#!/bin/bash

DIR=$(dirname $(realpath $0))

if [[ $1 == "test" ]]; then
	cd $DIR/tester
	cargo r --release $DIR/../target/release/Watt
fi
