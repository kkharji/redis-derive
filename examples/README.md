# Redis Derive Examples

This directory contains examples demonstrating the usage of `redis-derive` with different Redis value types and scenarios.

## Prerequisites

- Docker and Docker Compose
- Rust (latest stable version)

## Quick Start

### 1. Start Redis Server

Start the Redis server using Docker Compose:

```bash
# From the examples directory
docker-compose up -d
```

This will start a Redis 7 server on port 6379 with data persistence.

### 2. Run the Basic Example

```bash
# From the project root
cargo run --example main
```

### 3. Run the Comprehensive Enum Example

```bash
# From the project root  
cargo run --example enum_branches
```

### 4. Clean Up

When you're done testing:

```bash
# From the examples directory
docker-compose down
```

To also remove the data volume:

```bash
docker-compose down -v
```

## Examples Overview

### `main.rs`
Basic example showing struct and enum serialization/deserialization with Redis.

### `enum_branches.rs`
Comprehensive example that tests all branches of enum deserialization:
- `BulkString` - Normal stored enum values
- `SimpleString` - Direct Redis string responses
- `VerbatimString` - Redis 6+ RESP3 verbatim strings
- `Nil` - Error handling for nil values
- Invalid types - Error handling for incompatible types

## Troubleshooting

### Redis Connection Issues

If you get connection errors, make sure Redis is running:

```bash
# Check if Redis container is running
docker-compose ps

# Check Redis logs
docker-compose logs redis

# Test connection manually
redis-cli ping
```

### Port Conflicts

If port 6379 is already in use, you can change it in `docker-compose.yml`:

```yaml
ports:
  - "6380:6379"  # Use port 6380 instead
```

Then update your connection string in the examples to use `redis://127.0.0.1:6380/`.

## Redis Features Used

- **RESP2/RESP3 Protocol**: Examples work with both Redis protocol versions
- **Hash Operations**: Struct serialization uses Redis hashes (HSET/HGET)
- **String Operations**: Enum serialization uses Redis strings (SET/GET)
- **Error Handling**: Demonstrates proper error handling for various scenarios

## Development

To add new examples:

1. Create a new `.rs` file in this directory
2. Add it to the `[[example]]` section in `Cargo.toml`
3. Make sure it connects to `redis://127.0.0.1:6379/`
4. Update this README with a description

## Notes

- The Redis container uses Alpine Linux for a smaller image size
- Data is persisted in a Docker volume named `redis_data`
- Health checks ensure Redis is ready before examples run
- Redis 7 is used to support the latest RESP3 features