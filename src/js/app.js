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
        addUserToList(event.payload.name, event.payload.addr);
    });
}

document.addEventListener('DOMContentLoaded', renderPage);
