#!/bin/bash

echo "cold cache test"

su -c "echo 3 > /proc/sys/vm/drop_caches"
echo "### old"
time projectchooser -b -d "cloud" > /dev/null

su -c "echo 3 > /proc/sys/vm/drop_caches"
echo "### new:"
time ./target/release/project-chooser -b "cloud" > /dev/null

echo
echo "warm cache test"

su -c "echo 3 > /proc/sys/vm/drop_caches"
projectchooser -b -d "cloud" > /dev/null
echo "### old:"
time projectchooser -b -d "cloud" > /dev/null

su -c "echo 3 > /proc/sys/vm/drop_caches"
./target/release/project-chooser -b "cloud" > /dev/null
echo "### new:"
time ./target/release/project-chooser -b "cloud" > /dev/null
