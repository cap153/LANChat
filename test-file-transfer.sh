#!/bin/bash

# 测试文件传输功能

echo "=== LANChat 文件传输测试 ==="
echo ""

# 创建测试文件
TEST_FILE="test_upload.txt"
echo "这是一个测试文件，用于测试 LANChat 的文件传输功能。" > $TEST_FILE
echo "创建测试文件: $TEST_FILE"
echo ""

# 测试上传文件到本地服务器
echo "测试上传文件到 localhost:8888..."
curl -X POST http://localhost:8888/api/upload \
  -F "file=@$TEST_FILE" \
  -F "peer_id=test-peer-123" \
  -v

echo ""
echo ""

# 清理
rm -f $TEST_FILE
echo "测试完成"
