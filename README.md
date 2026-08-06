# Rocom Helper Launcher

[Rocom Helper](https://github.com/h3110w0r1d-y/rocom-helper) 的桌面启动器，自动检测版本、下载更新并启动。

## 功能

- 启动时自动检测 GitHub 连通性与最新版本
- 本地版本与远程版本对比，一键更新
- 支持自定义游戏 / WeGame 路径，一键启动
- 重新下载按钮（带删除确认）
- 版本刷新按钮，无需重启即可重新检测

## 技术栈

- **Tauri 2** — 跨平台桌面框架
- **Rust** — 后端逻辑（GitHub API、文件下载、进程启动）
- **Vue 3 + Vite** — 前端界面

## 构建

```bash
npm install
npx tauri build --no-bundle
```

生成的 exe 位于 `src-tauri/target/release/rocom-helper-launcher.exe`。

## 许可证

[MIT](LICENSE)
