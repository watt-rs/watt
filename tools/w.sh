#!/bin/bash

DIR=$(dirname $0)

if [[ $1 == "test" ]]; then
	cd $DIR/tester
	cargo r --release $DIR/../target/debug/Watt
fi
