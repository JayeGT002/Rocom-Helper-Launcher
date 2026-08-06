# Rocom-Helper-Launcher
为什么你会需要一个洛克王国世界助手的启动器？  
用于更新[h3110w0r1d-y/rocom-helper](https://github.com/h3110w0r1d-y/rocom-helper/) ，本项目~~暂不开源~~遵循MIT license，虽然也不会有人需要吧，纯属我本地不想塞这玩意的源码罢了。  

## 能干什么
1、启动洛克王国世界助手，然后启动游戏。  
2、检测GitHub的连通情况。  
3、自动更新洛克王国世界助手版本。  
4、没了，这玩意就那么点功能。  
⚠️警告：本项目需要您支持访问Github才能使用。  

## 项目技术栈：  
| 层级 | 技术 | 说明 |
|------|------|------|
| 框架 | **Tauri 2** | 跨平台桌面应用框架，exe 仅 5MB |
| 后端 | **Rust** | 原生编译，处理 GitHub API、文件下载、进程启动 |
| 前端 | **Vue 3 + Vite 5** | 响应式 UI，构建产物 82KB JS + 9KB CSS |
| HTTP 客户端 | **reqwest 0.12** | Rust 异步 HTTP，用于版本检测和文件下载 |
| 对话框/Shell | **tauri-plugin-dialog / tauri-plugin-shell** | 文件选择、确认弹窗、外部进程启动 |  

核心 Rust 依赖（`Cargo.toml`）：`tauri 2`、`serde`、`reqwest`、`tokio`、`futures-util`、`dirs`  
核心前端依赖（`package.json`）：`vue 3.5`、`@tauri-apps/api 2`、`@tauri-apps/plugin-dialog 2`、`@tauri-apps/plugin-shell 2`  

## License
本项目除美术资源外均遵循MIT license协议开源。    
