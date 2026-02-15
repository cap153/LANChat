#!/bin/bash

# LANChat Web API 测试脚本

PORT=${1:-8888}
BASE_URL="http://localhost:$PORT"

echo "========================================="
echo "LANChat Web API 测试"
echo "========================================="
echo "服务器地址: $BASE_URL"
echo ""

# 测试 1: 获取用户名
echo "测试 1: 获取当前用户名"
echo "----------------------------------------"
curl -s "$BASE_URL/api/get_my_name" | jq '.'
echo ""
echo ""

# 测试 2: 更新用户名
echo "测试 2: 更新用户名为 'TestUser-$(date +%s)'"
echo "----------------------------------------"
NEW_NAME="TestUser-$(date +%s)"
curl -s -X POST "$BASE_URL/api/update_my_name" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"$NEW_NAME\"}" | jq '.'
echo ""
echo ""

# 测试 3: 再次获取用户名（验证更新）
echo "测试 3: 验证用户名已更新"
echo "----------------------------------------"
curl -s "$BASE_URL/api/get_my_name" | jq '.'
echo ""
echo ""

# 测试 4: 测试空用户名（应该失败）
echo "测试 4: 测试空用户名（预期失败）"
echo "----------------------------------------"
curl -s -X POST "$BASE_URL/api/update_my_name" \
  -H "Content-Type: application/json" \
  -d '{"name":""}' | jq '.'
echo ""
echo ""

# 测试 5: 测试过长用户名（应该失败）
echo "测试 5: 测试过长用户名（预期失败）"
echo "----------------------------------------"
LONG_NAME=$(python3 -c "print('A' * 60)")
curl -s -X POST "$BASE_URL/api/update_my_name" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"$LONG_NAME\"}" | jq '.'
echo ""
echo ""

echo "========================================="
echo "测试完成！"
echo "========================================="
