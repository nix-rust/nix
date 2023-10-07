#!/bin/sh

function red() {
	echo "\033[0;31m$@\033[0m"
}

function green() {
	echo "\033[0;32m$@\033[0m"
}

function byellow() {
	echo "\033[1;33m$@\033[0m"
}

CHANGED_BY_CARGO_FMT=false


echo -e "$(green INFO): Checking code format..."

cargo fmt --all -q -- --check 2>/dev/null
if [ $? -ne 0 ]; then
    echo -e "$(byellow WARN): Unformatted code detected"
    echo -e "$(green INFO): Formatting..."
    cargo fmt --all
    CHANGED_BY_CARGO_FMT=true
fi

if ${CHANGED_BY_CARGO_FMT}; 
then
	echo -e "$(red FAIL): git commit $(red ABORTED), please have a look and run git add/commit again"
	exit 1
else
    echo -e "$(green INFO): Check passed"
fi

exit 0
