#!/bin/bash
# 测试桌面端消息实时显示

echo "=== 测试桌面端消息实时显示 ==="
echo ""
echo "步骤:"
echo "1. 编译桌面端: cd src-tauri && cargo build --bin lanchat --features desktop"
echo "2. 运行桌面端: ./src-tauri/target/debug/lanchat"
echo "3. 从另一个设备发送消息"
echo "4. 检查桌面端是否实时显示消息"
echo ""
echo "预期结果:"
echo "- 后台日志显示: [WebSocket] 收到文本消息"
echo "- 后台日志显示: [WebSocket] 已发送 Tauri 事件: new-message"
echo "- 前端聊天窗口实时显示新消息"
echo ""
echo "如果消息没有实时显示，检查:"
echo "- 浏览器控制台是否有 [JS-App] 收到新消息 日志"
echo "- 后台是否有 WebSocket 连接成功的日志"
