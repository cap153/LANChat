#!/bin/bash

# 测试文件上传 API

echo "=== 测试文件上传 API ==="
echo ""

# 创建测试文件
TEST_FILE="test_file.txt"
echo "这是一个测试文件" > $TEST_FILE

echo "1. 测试文件: $TEST_FILE"
echo "2. 文件大小: $(wc -c < $TEST_FILE) 字节"
echo ""

# 测试上传
echo "3. 上传到 localhost:8888..."
curl -v -X POST http://localhost:8888/api/upload \
  -F "peer_id=test-peer-123" \
  -F "file=@$TEST_FILE" \
  2>&1 | grep -E "(HTTP|< |> )"

echo ""
echo ""

# 清理
rm -f $TEST_FILE
echo "测试完成"
