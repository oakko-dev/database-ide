#!/usr/bin/env bash
set -euo pipefail

container_name="database-ide-schema-test"
database_url="postgres://postgres:database_ide_test@127.0.0.1:55432/database_ide_test"

cleanup() { docker rm -f "$container_name" >/dev/null 2>&1 || true; }
trap cleanup EXIT

docker rm -f "$container_name" >/dev/null 2>&1 || true
docker run --name "$container_name" -e POSTGRES_PASSWORD=database_ide_test -e POSTGRES_DB=database_ide_test -p 55432:5432 -d postgres:16-alpine >/dev/null

for attempt in $(seq 1 30); do
  if docker exec "$container_name" pg_isready -U postgres -d database_ide_test >/dev/null 2>&1; then break; fi
  sleep 1
done

DATABASE_URL="$database_url" cargo test --test postgres_schema -- --ignored --nocapture
