# Weather MCP Server

A Model Context Protocol (MCP) server running on the Internet Computer that provides weather information using the Open-Meteo API.

## Features

- Get current weather conditions for any location
- Uses Open-Meteo API (no API key required)
- Supports custom latitude/longitude coordinates
- Defaults to Berlin, Germany when no coordinates provided
- Returns detailed weather information including temperature, wind, and conditions
- Configurable HTTP request replication

## Weather Tool

### `get_current_weather`

Retrieves current weather conditions for a specified location.

**Parameters:**
- `latitude` (optional): Latitude coordinate (defaults to 52.52 for Berlin)
- `longitude` (optional): Longitude coordinate (defaults to 13.41 for Berlin)

**Example Response:**
```
Weather for GMT (GMT) (lat: 52.52, lon: 13.419998)
Time: 2025-10-15T09:45
Temperature: 12.9°C
Wind: 7.7 km/h from 307°
Conditions: Rainy (Day)
Timezone: GMT (GMT)
```

**Note:** All numeric values from the Open-Meteo API are returned as floating-point numbers, which are converted to appropriate formats for display (e.g., wind direction is rounded to the nearest degree).

## Building and Deployment

### Prerequisites
- Rust with wasm32-unknown-unknown target
- dfx (Internet Computer SDK)

### Build
```bash
cargo check --target wasm32-unknown-unknown
```

### Deploy
```bash
dfx start --background
dfx deploy weather --argument '(record { api_key = "your-api-key-here"; replicated = true })'
```

Note: The API key parameter is maintained for compatibility with the authentication system, though the Open-Meteo API doesn't require authentication. The `replicated` field controls whether HTTP requests are replicated across nodes (set to `true` for consistency across calls, `false` for potentially faster but non-deterministic responses).

## API Endpoint Used

The server fetches weather data from:
`https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current_weather=true`

## Weather Codes

The server interprets weather codes as follows:
- 0: Clear sky
- 1-3: Partly cloudy
- 45-48: Foggy
- 51-67: Rainy
- 71-86: Snowy
- 95-99: Thunderstorm

## Authentication

The server uses API key authentication via the `x-api-key` header for MCP requests, following the same pattern as other examples in this project.