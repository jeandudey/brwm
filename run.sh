#!/usr/bin/bash

set -e

XEPHYR=$(whereis -b Xephyr | cut -f2 -d' ')

runXephyrRelease ()
{
    xinit ./xinitrc-release -- \
        "$XEPHYR" \
            :100 \
            -ac \
            -screen 640x480 \
            -host-cursor
}

runXephyrDebug ()
{
    xinit ./xinitrc-debug -- \
        "$XEPHYR" \
            :100 \
            -ac \
            -screen 640x480 \
            -host-cursor
}

while [[ $# > 0 ]]
do
        case "$1" in
                --release)
                        echo "Release mode."
                        cargo build --release
                        runXephyrRelease
                        exit 1
                        ;;

                --help)
                        echo "USAGE: run.sh [OPTIONS]"
                        echo " OPTIONS:"
                        echo "    --release"
                        echo "    --help"
                        echo "EXAMPLE: run.sh --release"
                        exit 1
                        ;;
        esac
        shift
done

echo "Debug mode"
cargo build
runXephyrDebug
