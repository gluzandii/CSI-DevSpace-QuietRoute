#!/bin/bash

# Test the Quiet Route API

BASE_URL="http://127.0.0.1:3000"

echo "🧪 Testing Quiet Route API"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Test 1: Health Check
echo "1️⃣  Testing health check endpoint..."
curl -s "${BASE_URL}/health"
echo -e "\n"
echo ""

# Test 2: Find Route (Broadway HSR 27th Main to McDonald's HSR Layout)
echo "2️⃣  Testing route finding endpoint..."
echo "   Start: Broadway HSR 27th Main (12.923782, 77.651635)"
echo "   End: McDonald's HSR Layout (12.912297, 77.638196)"
echo ""

curl -s -X POST "${BASE_URL}/route" \
  -H "Content-Type: application/json" \
  -d '{
    "start_lat": 12.923782,
    "start_lon": 77.651635,
    "end_lat": 12.912297,
    "end_lon": 77.638196
  }' | jq '.'

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "✅ Tests complete!"
