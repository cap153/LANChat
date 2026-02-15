// src/js/app.js
async function renderPage() {
    console.log("[JS-App] 页面初始化开始...");
    
    const myName = await apiGetMyName();
    const nameElement = document.getElementById('my-name');
    if (nameElement) {
        nameElement.innerText = "我是：" + myName;
    }

    // 初始化改名功能
    initNameEditor();

    // 使用我们封装好的 apiListen
    await apiListen('new-peer', (event) => {
        console.log("[JS-App] 收到新邻居:", event.payload);
        addUserToList(event.payload.id, event.payload.name, event.payload.addr, false);
    });

    // 启动用户列表轮询（桌面端和 Web 端都需要）
    console.log("[JS-App] 启动用户列表轮询");
    startPeerPolling();
}

// Web 端轮询用户列表
async function startPeerPolling() {
    const pollInterval = 3000; // 3秒轮询一次
    
    const updatePeerList = async () => {
        const peers = await apiGetPeers();
        
        // 获取当前列表中的所有 ID
        const currentIds = new Set();
        const list = document.getElementById('user-list');
        if (list) {
            const items = list.querySelectorAll('li');
            items.forEach(item => currentIds.add(item.dataset.id));
        }
        
        // 更新用户列表
        const receivedIds = new Set();
        for (const peer of peers) {
            addUserToList(peer.id, peer.name, peer.addr, peer.is_offline);
            receivedIds.add(peer.id);
        }
        
        // 移除不在服务器列表中的用户（已经超过60秒）
        for (const id of currentIds) {
            if (!receivedIds.has(id)) {
                removeUserFromList(id);
            }
        }
    };
    
    // 立即执行一次
    await updatePeerList();
    
    // 定时轮询
    setInterval(updatePeerList, pollInterval);
}

document.addEventListener('DOMContentLoaded', renderPage);
