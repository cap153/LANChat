use clap::Parser;
use std::time::Duration;
use tokio;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 8888)]
    port: u16, // 这个端口同时用于 HTTP(TCP) 和 广播(UDP)
    
    #[arg(long)]
    db_path: Option<String>, // 可选的数据库路径
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let port = args.port;
    
    // 初始化数据库
    println!("[Server Main] 正在初始化数据库...");
    let db_path = args.db_path.map(|p| std::path::PathBuf::from(p));
    let pool = lanchat::db::init_db_standalone(db_path)
        .await
        .expect("数据库初始化失败");
    
    // 从数据库读取用户名
    let my_name = lanchat::db::get_username(&pool)
        .await
        .unwrap_or_else(|_| "Web-User".to_string());
    
    println!("[Server Main] 我的用户名: {}", my_name);

    // 1. 启动 Web 服务 (TCP)
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        lanchat::web_server::start_server(port, port, pool_clone).await;
    });

    // 2. 启动 UDP 监听
    let listen_name = my_name.clone();
    tokio::spawn(async move {
        lanchat::network::discovery::start_listening(port, listen_name, None).await;
    });

    // 3. 启动 UDP 广播
    tokio::spawn(async move {
        lanchat::network::discovery::start_announcing(port, my_name).await;
    });

    println!("[Server Main] Web 服务器已启动，端口: {}", port);
    println!("[Server Main] 访问 http://localhost:{}", port);

    // 防止主线程退出
    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}
