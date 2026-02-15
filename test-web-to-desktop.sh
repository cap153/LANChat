#!/bin/bash

echo "=== 测试 Web 端到桌面端文件传输 ==="
echo ""

# 创建测试文件
echo "创建测试文件..."
echo "这是一个测试文件" > test_small.txt
dd if=/dev/urandom of=test_1kb.bin bs=1024 count=1 2>/dev/null
dd if=/dev/urandom of=test_1mb.bin bs=1024 count=1024 2>/dev/null

echo ""
echo "1. 测试小文件 (test_small.txt)"
curl -v -X POST http://localhost:8888/api/upload \
  -F "peer_id=test-web-client" \
  -F "file=@test_small.txt" \
  2>&1 | grep -E "(HTTP|Content-Type|error)"

echo ""
echo ""
echo "2. 测试 1KB 文件 (test_1kb.bin)"
curl -v -X POST http://localhost:8888/api/upload \
  -F "peer_id=test-web-client" \
  -F "file=@test_1kb.bin" \
  2>&1 | grep -E "(HTTP|Content-Type|error)"

echo ""
echo ""
echo "3. 测试 1MB 文件 (test_1mb.bin)"
curl -v -X POST http://localhost:8888/api/upload \
  -F "peer_id=test-web-client" \
  -F "file=@test_1mb.bin" \
  2>&1 | grep -E "(HTTP|Content-Type|error)"

echo ""
echo ""

# 清理
rm -f test_small.txt test_1kb.bin test_1mb.bin

echo "测试完成"
echo ""
echo "请检查桌面端的日志和下载目录"
