#!/bin/bash

echo "=== LANChat 端口诊断 ==="
echo ""

echo "1. 检查 8888 端口"
if lsof -i :8888 > /dev/null 2>&1; then
    echo "   ✓ 端口 8888 正在使用"
    lsof -i :8888 | grep -v COMMAND
else
    echo "   ✗ 端口 8888 未使用"
fi

echo ""
echo "2. 检查 9999 端口"
if lsof -i :9999 > /dev/null 2>&1; then
    echo "   ✓ 端口 9999 正在使用"
    lsof -i :9999 | grep -v COMMAND
else
    echo "   ✗ 端口 9999 未使用"
fi

echo ""
echo "3. 测试 HTTP 连接"
echo "   测试 localhost:8888..."
if curl -s http://localhost:8888/api/get_my_name > /dev/null 2>&1; then
    echo "   ✓ localhost:8888 可访问"
else
    echo "   ✗ localhost:8888 不可访问"
fi

echo ""
echo "   测试 localhost:9999..."
if curl -s http://localhost:9999/api/get_my_name > /dev/null 2>&1; then
    echo "   ✓ localhost:9999 可访问"
else
    echo "   ✗ localhost:9999 不可访问"
fi

echo ""
echo "4. 测试文件上传接口"
echo "test" > /tmp/test_upload.txt

echo "   测试上传到 localhost:8888..."
RESULT=$(curl -s -w "%{http_code}" -o /tmp/upload_result.txt \
    -X POST http://localhost:8888/api/upload \
    -F "peer_id=test" \
    -F "file=@/tmp/test_upload.txt" 2>&1)

if [ "$RESULT" = "200" ]; then
    echo "   ✓ 上传成功 (HTTP 200)"
    cat /tmp/upload_result.txt
else
    echo "   ✗ 上传失败 (HTTP $RESULT)"
    cat /tmp/upload_result.txt
fi

rm -f /tmp/test_upload.txt /tmp/upload_result.txt

echo ""
echo "诊断完成"
