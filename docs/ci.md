# CI 与发布

## 日常检查

推送 `main`，或创建、更新、重新打开目标为 `main` 的 PR 时，`Checks` 自动运行。合并 PR 后的 `main` push 会再次检查合并结果。普通功能分支的 push（尚未创建 PR）和推送标签不会触发此工作流。也可以在 Actions 中手动运行。

同一个 PR 或分支连续提交时，旧检查会取消。保留两个现有检查：

- `workspace`：Windows 上检查 Rust 格式、类型、测试、生成的 IPC 契约、国际化、前端测试与构建，以及 Rust workspace 构建。
- `unix-cli`：Linux 上测试除桌面端外的 Rust workspace，保护 CLI 的跨平台兼容性。

本地 Windows 使用 `pnpm install --frozen-lockfile` 后运行 `pnpm run ci`。Unix Rust 检查命令为 `cargo test --workspace --locked --exclude dtt-desktop`，或安装 pnpm 后运行 `pnpm ci:unix`。Rust 版本沿用 `rust-toolchain.toml`，pnpm 版本沿用根目录 `package.json`；CI 使用 Node.js 24。

如需强制 PR 检查通过后才能合并，在 GitHub 仓库 ruleset 中将 `workspace` 与 `unix-cli` 设为 required status checks。提交 YAML 本身不会配置仓库合并规则。单人维护无需要求其他人的审批。

## 手动生成发布草稿

CI 通过不会自动发布。`Release draft` 仅接受手动触发，且只在选择 `main` 时运行。它在 Windows 和 Linux 两个平台分别重新执行全量检查，随后构建桌面端和 CLI 的 release 二进制。两个平台都成功后，才创建同一个 GitHub Release 草稿；任一平台失败都不会创建草稿。

1. 在根 `Cargo.toml` 的 `[workspace.package]`、`apps/dtt-desktop/package.json` 和 `apps/dtt-desktop/src-tauri/tauri.conf.json` 中设置相同版本，例如 `0.1.1`。更新并提交相关锁文件。版本不一致时发布会失败。
2. 将改动提交并推送到 `main`，等待该提交的 `Checks` 通过。
3. 打开 GitHub → Actions → **Release draft** → **Run workflow**，选择 `main`。使用该次运行的固定 commit SHA 打包；运行期间的新提交不会混入产物。首次使用须先将 workflow 文件推送到默认分支。
4. 工作流生成 `v<版本号>` 的草稿，附上 `dtt-v<版本号>-windows-x64.zip`、`dtt-v<版本号>-linux-x64.tar.gz` 和统一的 `SHA256SUMS.txt`。已有同名标签或草稿不会被覆盖。
5. 在 Releases 中打开草稿，下载两个平台的压缩包测试，填写更新说明，最后手动点击 **Publish release**。这一步才对外发布。

Windows ZIP 包含 `dtt-gui.exe`、`dtt.exe` 和简短说明；桌面端需要 WebView2 Runtime。Linux tar.gz 包含保留执行权限的 `dtt-gui`、`dtt`、说明和 `runtime-libraries.txt`，解压后运行 `./dtt-gui` 或 `./dtt --help`。

Linux 二进制使用 Ubuntu 24.04 x64 构建，动态链接系统库。目标系统需要兼容的 glibc；桌面端还需要 GTK 3、WebKitGTK 4.1 等库，具体依赖随包列出。它不是完全静态的通用 Linux 程序，不保证兼容旧发行版或 Alpine。

沿用项目关闭安装包的设置，仅分发未签名的原生二进制压缩包，不生成 RPM、DEB、AppImage、MSI 或 macOS 包，也不配置自动更新。两个平台的构建产物也会在 Actions 中保留 7 天。

工作流使用 GitHub 内置的 `GITHUB_TOKEN` 创建草稿，无需另配 PAT。日常 CI 和两个平台的构建 job 只有读取权限；只有汇总草稿的 `draft` job 获得 `contents: write`。发布不会因新的 main push 被取消。

如果创建草稿时上传中断，先检查 Releases 中是否已出现不完整草稿；清理该草稿后再重跑。若对应标签已经存在，使用新版本号，工作流不会重写标签或覆盖正式版本。
