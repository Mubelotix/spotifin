#!/bin/bash
set -euo pipefail

container=spotify
profile=debug
target_dir="${CARGO_TARGET_DIR:-target}"

while [ "$#" -gt 0 ]; do
    case "$1" in
        --release)
            profile=release
            ;;
        --debug)
            profile=debug
            ;;
        *)
            printf 'usage: %s [--debug|--release]\n' "$0" >&2
            exit 2
            ;;
    esac
    shift
done

case "$profile" in
    debug)
        cargo build --locked
        binary="$target_dir/debug/spotify-server"
        ;;
    release)
        cargo build --release --locked
        binary="$target_dir/release/spotify-server"
        ;;
esac

if [ "$(podman inspect -f '{{.State.Status}}' "$container" 2>/dev/null || true)" != "running" ]; then
    printf 'container %s is not running\n' "$container" >&2
    exit 1
fi

podman cp "$binary" "$container:/tmp/spotify-server.new"
podman exec "$container" install \
    -o abc -g abc -m 0755 \
    /tmp/spotify-server.new /usr/local/bin/spotify-server
podman exec "$container" rm -f /tmp/spotify-server.new

# Replace only the backend. Spotify, its session, the bridge, and the recorders
# stay alive, so backend changes do not require rebuilding or recreating the image.
podman exec "$container" pkill -x spotify-server 2>/dev/null || true
podman exec -d --user abc "$container" sh -c \
    'exec env ROCKET_ADDRESS=0.0.0.0 ROCKET_PORT=8000 AUDIO_DATA_DIR=/config/audio \
        /usr/local/bin/spotify-server >>/tmp/backend-live.log 2>&1'

for _ in {1..20}; do
    if curl --fail --silent http://127.0.0.1:8032/health >/dev/null; then
        printf 'backend updated (%s build)\n' "$profile"
        exit 0
    fi
    sleep 0.25
done

printf 'backend did not become healthy; inspect with: podman logs %s\n' "$container" >&2
exit 1
