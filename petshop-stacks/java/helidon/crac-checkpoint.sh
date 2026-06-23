#!/bin/sh
# crac-checkpoint.sh — Helidon CRaC checkpoint entrypoint.
#
# Starts the Helidon JVM with -XX:CRaCCheckpointTo=/checkpoint, waits for the
# health endpoint to report ready, then triggers the checkpoint via jcmd.
# After jcmd JDK.checkpoint the JVM writes CRIU files to /checkpoint and exits.
# The benchmark harness (run.sh) then docker commit the stopped container as the
# final restore image with ENTRYPOINT ["java", "-XX:CRaCRestoreFrom=/checkpoint"].
set -e

java \
  -XX:InitialRAMPercentage=75.0 \
  -XX:MaxRAMPercentage=75.0 \
  -XX:+UseG1GC \
  -XX:+OptimizeStringConcat \
  -Djava.security.egd=file:/dev/./urandom \
  -XX:CRaCCheckpointTo=/checkpoint \
  -XX:CRaCMinPid=128 \
  -Dmetrics.rest-request.enabled=false \
  -jar /app/app.jar &
JVM_PID=$!

echo "[crac] Waiting for Helidon to be ready (PID=$JVM_PID) ..."
DEADLINE=$(( $(date +%s) + 60 ))
while [ "$(date +%s)" -lt "$DEADLINE" ]; do
    if curl -sf "http://localhost:8080/api/v3/pet/findByStatus?status=available" > /dev/null 2>&1; then
        echo "[crac] Helidon ready — triggering CRaC checkpoint ..."
        jcmd "$JVM_PID" JDK.checkpoint
        wait "$JVM_PID" 2>/dev/null || true
        echo "[crac] Checkpoint complete"
        exit 0
    fi
    kill -0 "$JVM_PID" 2>/dev/null || { echo "[crac] JVM exited before ready"; exit 1; }
    sleep 0.5
done

echo "[crac] Timeout waiting for Helidon — aborting checkpoint"
kill "$JVM_PID" 2>/dev/null || true
exit 1
