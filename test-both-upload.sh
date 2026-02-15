#!/bin/bash

# 测试两端的文件上传

echo "=== 测试文件上传（两端） ==="
echo ""

# 创建小测试文件
TEST_FILE="small_test.txt"
echo "这是一个小测试文件" > $TEST_FILE

echo "1. 测试上传小文件到 localhost:8888"
curl -X POST http://localhost:8888/api/upload \
  -F "peer_id=test-123" \
  -F "file=@$TEST_FILE" \
  2>&1 | tail -5

echo ""
echo ""

# 创建稍大的测试文件（1KB）
LARGE_FILE="large_test.txt"
dd if=/dev/urandom of=$LARGE_FILE bs=1024 count=1 2>/dev/null

echo "2. 测试上传 1KB 文件到 localhost:8888"
curl -X POST http://localhost:8888/api/upload \
  -F "peer_id=test-456" \
  -F "file=@$LARGE_FILE" \
  2>&1 | tail -5

echo ""
echo ""

# 清理
rm -f $TEST_FILE $LARGE_FILE
echo "测试完成"
