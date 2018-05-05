#!/bin/bash
echo "equality test!"
echo "TODO"

echo "cold cache test"

su -c "echo 3 > /proc/sys/vm/drop_caches"
echo "### find (raw)"
time find ~/projects/ -not \( -path "*/src/*" -prune \) -not \( -path "*/.git/*" -prune \) -iname ".git" -or -iname ".project" -or -iname ".groupproject"> /dev/null

su -c "echo 3 > /proc/sys/vm/drop_caches"
echo "### find_arr:"
time ./find_test.sh > /dev/null

echo "### rpc:"
su -c "echo 3 > /proc/sys/vm/drop_caches"
time ./target/release/project-walker > /dev/null

echo
echo "warm cache test"

su -c "echo 3 > /proc/sys/vm/drop_caches"
find ~/projects/ -not \( -path "*/src/*" -prune \) -not \( -path "*/.git/*" -prune \) -iname ".git" -or -iname ".project" -or -iname ".groupproject"> /dev/null
echo "### find (raw)"
time find ~/projects/ -not \( -path "*/src/*" -prune \) -not \( -path "*/.git/*" -prune \) -iname ".git" -or -iname ".project" -or -iname ".groupproject"> /dev/null

su -c "echo 3 > /proc/sys/vm/drop_caches"
./find_test.sh > /dev/null
echo "### find_arr:"
time ./find_test.sh > /dev/null

su -c "echo 3 > /proc/sys/vm/drop_caches"
./target/release/project-walker > /dev/null
echo "### rpc:"
time ./target/release/project-walker > /dev/null
