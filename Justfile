image := "ghcr.io/matsu0122-png/ustam"
tag := "latest"

# Run formatting, lint, and test checks (mirrors CI).
check:
    cargo fmt --check
    cargo clippy -- -D warnings
    cargo test

# Build the container image defined in Containerfile.
docker-build:
    docker build -t {{ image }}:{{ tag }} -f Containerfile .

# Run the container image built by docker-build.
docker-run *args:
    docker run --rm -v "$(pwd)":/workspace {{ image }}:{{ tag }} {{ args }}

# Push the container image to the registry.
docker-push:
    docker push {{ image }}:{{ tag }}
