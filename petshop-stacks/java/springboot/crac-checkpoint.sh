#!/bin/sh
# crac-checkpoint.sh — Spring Boot CRaC checkpoint entrypoint.
#
# Starts the JVM with -XX:CRaCCheckpointTo=/checkpoint, waits for Tomcat to
# accept HTTP (without hitting the DB — lazy DataSource stays uninitialized),
# then triggers the checkpoint via jcmd on the child JVM PID.
#
# -XX:CRaCCheckpointTo forks a child JVM; $! points at the wrapper, so jps is
# used to find the real process.  spring.context.checkpoint=onRefresh is avoided
# because it runs before HikariCheckpointRestoreLifecycle can close sockets.
set -e

java \
  -XX:InitialRAMPercentage=75.0 \
  -XX:MaxRAMPercentage=75.0 \
  -XX:+UseG1GC \
  -XX:+OptimizeStringConcat \
  -Djava.security.egd=file:/dev/./urandom \
  -XX:CRaCCheckpointTo=/checkpoint \
  -XX:CRaCMinPid=128 \
  -Dspring.main.lazy-initialization=true \
  -Dspring.autoconfigure.exclude=org.springframework.boot.autoconfigure.orm.jpa.HibernateJpaAutoConfiguration,org.springframework.boot.autoconfigure.data.jpa.JpaRepositoriesAutoConfiguration \
  -Dspring.datasource.hikari.allow-pool-suspension=true \
  -Dspring.datasource.hikari.minimum-idle=0 \
  -Dspring.datasource.hikari.initialization-fail-timeout=-1 \
  -Dspringdoc.swagger-ui.enabled=false \
  -Dspringdoc.api-docs.enabled=false \
  -Dlogging.level.root=WARN \
  -jar /app/app.jar &
WRAPPER_PID=$!

echo "[crac] Waiting for Spring Boot Tomcat (wrapper PID=$WRAPPER_PID) ..."
DEADLINE=$(( $(date +%s) + 120 ))
while [ "$(date +%s)" -lt "$DEADLINE" ]; do
    if curl -sf -o /dev/null http://localhost:8080/ 2>/dev/null; then
        JVM_PID=$(jps -l 2>/dev/null | awk '/\/app\/app\.jar/{print $1; exit}')
        if [ -z "$JVM_PID" ]; then
            echo "[crac] Could not resolve Spring Boot JVM PID via jps"
            kill "$WRAPPER_PID" 2>/dev/null || true
            exit 1
        fi
        echo "[crac] Spring Boot ready (JVM PID=$JVM_PID) — triggering CRaC checkpoint ..."
        sleep 1
        jcmd "$JVM_PID" JDK.checkpoint
        wait "$WRAPPER_PID" 2>/dev/null || true
        if [ ! -f /checkpoint/engine ]; then
            echo "[crac] Checkpoint files missing under /checkpoint"
            exit 1
        fi
        echo "[crac] Checkpoint complete"
        exit 0
    fi
    kill -0 "$WRAPPER_PID" 2>/dev/null || { echo "[crac] JVM exited before ready"; exit 1; }
    sleep 0.5
done

echo "[crac] Timeout waiting for Spring Boot — aborting checkpoint"
kill "$WRAPPER_PID" 2>/dev/null || true
exit 1
