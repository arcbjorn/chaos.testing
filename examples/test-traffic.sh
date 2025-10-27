#!/bin/bash
# Send test traffic through the interceptor

BASE_URL="http://localhost:8080"

echo "Sending test traffic to $BASE_URL"
echo ""

echo "1. GET /"
curl -s "$BASE_URL/" | jq .
sleep 0.5

echo ""
echo "2. GET /api/users"
curl -s "$BASE_URL/api/users" | jq .
sleep 0.5

echo ""
echo "3. GET /api/products"
curl -s "$BASE_URL/api/products" | jq .
sleep 0.5

echo ""
echo "4. GET /api/products/123"
curl -s "$BASE_URL/api/products/123" | jq .
sleep 0.5

echo ""
echo "5. POST /api/orders"
curl -s -X POST "$BASE_URL/api/orders" \
  -H "Content-Type: application/json" \
  -d '{"item":"widget","quantity":3}' | jq .
sleep 0.5

echo ""
echo "6. GET /api/health"
curl -s "$BASE_URL/api/health" | jq .

echo ""
echo "Test traffic sent successfully"
