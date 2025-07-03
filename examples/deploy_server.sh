#!/bin/bash

# Configuration
API_KEY="123"  # Default API key, can be overridden with environment variable MCP_API_KEY

# Check if folder name is provided as argument
if [ $# -eq 0 ]; then
    echo "❌ Usage: $0 <folder_name> [api_key]"
    echo "Example: $0 adder"
    echo "Example: $0 adder myapikey123"
    echo "Example: MCP_API_KEY=mykey $0 adder"
    exit 1
fi

FOLDER_NAME=$1

# Override API key if provided as second argument or environment variable
if [ $# -ge 2 ]; then
    API_KEY=$2
elif [ ! -z "$MCP_API_KEY" ]; then
    API_KEY=$MCP_API_KEY
fi

echo "🔑 Using API Key: $API_KEY"

# Navigate to the project directory
echo "Navigating to $FOLDER_NAME project directory..."
cd "$FOLDER_NAME"

# Check if we're in the right directory (has dfx.json)
if [ ! -f dfx.json ]; then
    echo "❌ dfx.json not found in $FOLDER_NAME directory! Make sure you're running this script from the examples directory and the folder exists."
    exit 1
fi

# Deploy the canister to the playground
echo "Deploying $FOLDER_NAME canister to IC playground..."
cargo generate-lockfile
dfx deploy "$FOLDER_NAME" --argument "\"$API_KEY\"" --mode reinstall --output-env-file .env --playground

# Check if deployment was successful
if [ $? -ne 0 ]; then
    echo "❌ Deployment failed!"
    exit 1
fi

echo "✅ Deployment successful!"

# Check if .env file exists
if [ ! -f .env ]; then
    echo "❌ .env file not found!"
    exit 1
fi

# Extract the CANISTER_ID from the .env file
CANISTER_ID_VAR="CANISTER_ID_$(echo $FOLDER_NAME | tr '[:lower:]' '[:upper:]')"
CANISTER_ID=$(grep "$CANISTER_ID_VAR" .env | cut -d'=' -f2 | tr -d '"' | tr -d "'")

# Check if CANISTER_ID was found
if [ -z "$CANISTER_ID" ]; then
    echo "❌ Could not extract $CANISTER_ID_VAR from .env file"
    echo "Available variables in .env:"
    cat .env
    exit 1
fi

echo ""
echo "🚀 MCP Server ($FOLDER_NAME) deployed successfully!"
echo "📍 Server URL: https://${CANISTER_ID}.icp0.io/mcp"
echo "🔑 API Key: $API_KEY"
echo ""
echo "📋 Copy this JSON configuration for your MCP client:"
echo "┌─────────────────────────────────────────────────────────────┐"

# Generate server name with timestamp to avoid conflicts
SERVER_NAME="mcp-server-${FOLDER_NAME}-$(date +%s)"

cat << EOF
{
  "servers": {
    "$SERVER_NAME": {
      "url": "https://${CANISTER_ID}.icp0.io/mcp",
      "headers": {
        "x-api-key": "$API_KEY"
      }
    }
  }
}
EOF

echo "└─────────────────────────────────────────────────────────────┘"
