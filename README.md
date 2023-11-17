# Merchant O' Rust

**Merchant O' Rust** is a simple API gateway built with Rust and Hyper, used for routing requests to different endpoints. It provides a minimalistic approach to remapping and forwarding HTTP requests.

## Usage

To run the program locally, built the project and execute the following command in your terminal:

```bash
APP_CONFIG_PATH=/path/to/config.json merchant_o_rust
```

Ensure your configuration file (`config.json`) follows this structure:

```json
{
  "log_level": "DEBUG",
  "map": {
    "/path": "http://pathto.com/remap"
  }
}
```

- **log_level**: Set the logging level (e.g., "DEBUG", "INFO").
- **map**: Define the mappings from local paths to target URLs.

## Docker Usage

If you prefer running the application within a Docker container, follow these steps:

1. Build the Docker image:

```bash
docker build -t merchant_o_rust .
```

2. Run the Docker container:

```bash
docker run -v /path/to/config_folder:/configs -e APP_CONFIG_PATH=/configs/config.json -p 8081:8188 merchant_o_rust
```

The application will now run on port 8081

## Example Configuration

Here's an example configuration file:

```json
{
  "log_level": "DEBUG",
  "map": {
    "/api/v1": "http://192.168.0.112:8080/api/v1",
    "/api/v2": "http://192.168.0.113:8080/api/v2"
  }
}
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE.txt) file for details.
