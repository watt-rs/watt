#!/bin/bash

DIR=$(dirname $(realpath $0))

if [[ $1 == "test" ]]; then
	cd $DIR/tester
	cargo r -q --release test $DIR/..
elif [[ $1 == "bench" ]]; then
	if [[ -z $2 ]]; then
		echo "Select .wt file to bench"
		exit 1
	fi
	cd $DIR/tester
	cargo r -q --release bench $DIR/.. $2
elif [[ $1 == "help" ]]; then
	echo "Usage: $0 mode [FILES...]"
	echo "Available modes: test, bench"
else
	echo "Unknown argument: $1"
fi
