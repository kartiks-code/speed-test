#!/bin/sh
# Optimized-variant entrypoint: warm Laravel's caches with the runtime env
# (available now via --env-file), then run php-fpm in the background and
# nginx in the foreground as PID 1.
set -u

for cmd in config:cache route:cache event:cache; do
    if ! php artisan "$cmd"; then
        echo "warning: php artisan $cmd failed; continuing without it" >&2
    fi
done

# -D daemonizes despite zz-docker.conf's daemonize=no (CLI flag wins).
php-fpm -D

exec nginx -g 'daemon off;'
