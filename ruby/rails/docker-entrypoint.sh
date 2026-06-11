#!/bin/sh
set -e

if [ "$1" = 'bin/rails' ] && { [ "$2" = 'server' ] || [ "$2" = 's' ]; }; then
    rm -f tmp/pids/server.pid
    bin/rails db:migrate
fi

exec "$@"
