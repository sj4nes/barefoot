#!/bin/bash

# Test script for HTTP MCP server
SERVER_URL="http://127.0.0.1:8080/mcp"

echo "Testing HTTP MCP server at $SERVER_URL"
echo "======================================"

# Test 1: Initialize
echo -e "\n1. Testing initialize method..."
curl -s -X POST "$SERVER_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-03-26"
    }
  }' | jq '.'

# Test 2: List tools
echo -e "\n2. Testing list_tools method..."
curl -s -X POST "$SERVER_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "list_tools",
    "params": {}
  }' | jq '.'

# Test 3: List offerings
echo -e "\n3. Testing list_offerings method..."
curl -s -X POST "$SERVER_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "list_offerings",
    "params": {}
  }' | jq '.'

# Test 4: Call a tool
echo -e "\n4. Testing call_tool method..."
curl -s -X POST "$SERVER_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "call_tool",
    "params": {
      "tool": "health_check",
      "args": {
        "detailed": true
      }
    }
  }' | jq '.'

echo -e "\n======================================"
echo "Test completed!" 