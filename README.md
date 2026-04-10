<div align="center">

<img src="https://img.shields.io/badge/-%E2%9A%A1_WINDSURF_ACCOUNT_MANAGER-667eea?style=for-the-badge&labelColor=764ba2" alt="title" />

<br/><br/>

<img src="https://img.shields.io/github/v/release/1837620622/windsurf-account-manager-releases?style=flat-square&color=667eea&label=VERSION" alt="version" />
<img src="https://img.shields.io/badge/Tauri_2.x-Rust_+_Vue3-FFC131?style=flat-square&logo=tauri&logoColor=white" alt="tauri" />
<img src="https://img.shields.io/badge/Windows%20|%20macOS%20|%20Linux-支持-764ba2?style=flat-square" alt="platform" />
<img src="https://img.shields.io/badge/ARM64%20|%20x64-全架构-67c23a?style=flat-square" alt="arch" />

<br/><br/>

**Windsurf 多账号管理 & 无感切号 桌面应用**

*基于 Tauri + Vue3 + Rust 构建 | 高性能 | 全平台 | 开箱即用*

<br/>

[`下载最新版本`](https://github.com/1837620622/windsurf-account-manager-releases/releases)

</div>

<br/>

<div align="center">

### 🔔 最新动态

> **截止 2026 年 4 月 10 日，切号功能一切正常。**
>
> 近期听闻社区反馈：部分新注册账号在其他工具中出现无法切号的情况。
> 经本人实测，在本切号软件中，**新注册账号的切号功能完全正常**，不受该问题影响。
> 如遇类似问题，欢迎使用本工具进行切换。

</div>

<div align="center">

### 🔐 v2.2.0 更新内容

</div>

```
┌─────────────────────────────────────────────────────────────────┐
│  ✨ 新增欢迎门户页                                             │
│     启动软件后需输入访问口令 cknb 进入主界面                    │
│                                                                 │
│  🔧 修复新账号无法登录                                          │
│     解决部分新注册账号无法导入和登录的问题                      │
│                                                                 │
│  🍎 macOS 兼容性优化                                            │
│     修复 macOS 下部分快捷键无响应的问题                         │
└─────────────────────────────────────────────────────────────────┘
```

<img src="https://raw.githubusercontent.com/andreasbm/readme/master/assets/lines/rainbow.png" alt="line" />

## 界面预览

<div align="center">
<img src="images/dashboard.png" width="92%" alt="dashboard" />
<br/>
<sub>主界面 — 多账号卡片式管理，配额 / 套餐 / 状态一目了然</sub>
</div>

<img src="https://raw.githubusercontent.com/andreasbm/readme/master/assets/lines/rainbow.png" alt="line" />

## 核心功能

```
┌─────────────────────────────────────────────────────────────────┐
│  SEAMLESS SWITCH    配额耗尽 → 自动切换下一个账号 → 零操作      │
│  ELASTIC BILLING    实时 Daily / Weekly 配额剩余百分比监控      │
│  BATCH MANAGEMENT   批量导入 / 刷新 / 登录 / 导出              │
│  TEAM CONTROL       团队成员管理 / 席位调整 / 积分重置          │
│  PRIVACY MODE       一键隐藏邮箱地址                            │
│  MULTI-PLATFORM     Windows / macOS / Linux × x64 / ARM64      │
└─────────────────────────────────────────────────────────────────┘
```

<img src="https://raw.githubusercontent.com/andreasbm/readme/master/assets/lines/rainbow.png" alt="line" />

## 前置要求

```
┌─────────────────────────────────────────────────────────────────┐
│  [!] 重要：使用本工具前，请确保以下环境已就绪                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. VPN / 代理环境（必须）                                      │
│     本工具需要访问 Windsurf 海外服务器进行账号登录和信息获取     │
│     请确保已开启 VPN 或全局代理，否则登录和刷新操作将会失败     │
│     推荐使用：Clash / V2Ray / Surge / 机场订阅                  │
│                                                                 │
│  2. Windsurf 编辑器（已安装）                                   │
│     无感切号功能需要检测 Windsurf 安装路径                      │
│     macOS 默认路径: /Applications/Windsurf.app                  │
│     Windows 默认路径: C:\Users\用户名\AppData\Local\Windsurf    │
│                                                                 │
│  3. Windsurf 账号（至少 1 个）                                  │
│     需要有效的 Windsurf 邮箱 + 密码 或 Refresh Token            │
│     建议准备 2 个以上账号以发挥无感切号功能                     │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

<img src="https://raw.githubusercontent.com/andreasbm/readme/master/assets/lines/rainbow.png" alt="line" />

## 快速上手

> **操作流程：开启 VPN → 输入口令 cknb → 开启无感切号 → 导入账号 → 自动切换**

<br/>

### `STEP 00` — 确认 VPN 已开启 & 输入口令进入

```
请先确认 VPN / 代理已开启并处于连接状态

启动软件后，会显示欢迎门户页面：
  ┌─────────────────────────────┐
  │  请输入以下口令进入软件      │
  │                             │
  │     ┌──┬──┬──┬──┐          │
  │     │c │k │n │b │          │
  │     └──┴──┴──┴──┘          │
  │                             │
  │  [ 在此输入口令 cknb ]      │
  └─────────────────────────────┘

在输入框中输入 cknb ，点击「进入」即可进入主界面
```

<br/>

### `STEP 01` — 开启无感切号

安装并打开软件后，点击左下角 **设置** → 切换到 **无感切号** 选项卡

<div align="center">
<img src="images/seamless-switch.png" width="92%" alt="seamless-switch" />
</div>

```
1. 确认 Windsurf 路径已自动检测（或手动浏览选择）
2. 将「启用无感切号」开关切为【开启】
3. 点击【保存】
```

> 开启后，当前账号配额耗尽时自动切换到下一个可用账号，并自动重启 Windsurf

<br/>

### `STEP 02` — 导入账号

点击顶部 **添加账号** 或 **批量导入** 按钮

**方式一：单个添加**

```
1. 点击顶部栏的 [+] 按钮
2. 选择「邮箱密码」模式
3. 输入 Windsurf 邮箱和密码
4. 可选填备注名称、分组、标签
5. 点击【确定】，会自动登录并获取账号信息
```

**方式二：批量导入（推荐，适合多账号）**

```
1. 点击顶部栏的 [批量导入] 按钮
2. 选择「邮箱密码」模式
3. 在文本框中粘贴账号列表，每行一个，格式：

   邮箱 密码 备注(可选)

   示例：
   user1@example.com password123 主力号
   user2@example.com password456 备用号
   user3@example.com password789

4. 勾选「导入后自动登录」
5. 选择分组和标签（可选）
6. 点击【导入】
```

> 也支持 Refresh Token 模式导入，适用于无密码但有 Token 的场景

<br/>

### `STEP 03` — 开始使用

```
账号导入成功后：

  → 每张卡片实时显示：
      套餐类型（Free / Pro / Teams / Enterprise）
      配额用量（已用 / 总量）
      弹性计费剩余百分比（Daily Remaining / Weekly Remaining）
      订阅到期时间
      账号状态（正常 / 已禁用）

  → 无感切号工作流程：
      当前账号配额耗尽
      → 软件自动检测到配额不足
      → 自动切换到下一个配额充足的账号
      → Windsurf 自动重启并加载新账号
      → 无缝继续编码工作

  → 手动操作：
      点击 [刷新] 按钮可手动更新所有账号信息
      点击账号卡片上的 [登录] 可手动切换到指定账号
      支持拖拽排序，调整账号优先级
```

<img src="https://raw.githubusercontent.com/andreasbm/readme/master/assets/lines/rainbow.png" alt="line" />

## 下载安装

前往 [`Releases`](https://github.com/1837620622/windsurf-account-manager-releases/releases) 下载最新版本

### Windows

| 文件类型 | 适用架构 | 说明 |
|:--------:|:--------:|:-----|
| `*_x64-setup.exe` | x64 (Intel/AMD) | NSIS 安装程序，适用于大多数 Windows 电脑 |
| `*_arm64-setup.exe` | ARM64 | 适用于搭载高通骁龙芯片的 Windows 笔记本 |
| `*_x64_zh-CN.msi` | x64 | MSI 安装包（企业部署推荐） |
| `*_arm64_zh-CN.msi` | ARM64 | MSI 安装包 ARM64 版本 |

> 不确定架构？打开 **设置 → 系统 → 关于**，查看「系统类型」

### macOS

| 文件类型 | 适用架构 | 说明 |
|:--------:|:--------:|:-----|
| `*_aarch64.dmg` | Apple Silicon (M1/M2/M3/M4) | 适用于 2020 年后的 Mac（推荐） |
| `*_x64.dmg` | Intel | 适用于 2020 年前的 Intel Mac |

> 不确定架构？点击左上角苹果图标 → **关于本机**，查看芯片信息

#### macOS 启动说明（重要，必读）

macOS 对未签名应用会拦截，提示「已损坏，无法打开」。需要通过终端启动：

```bash
# 第一步：安装后，先移除系统隔离属性（只需执行一次）
sudo xattr -rd com.apple.quarantine /Applications/windsurf-account-manager.app
```

```bash
# 第二步：每次启动都使用此命令（必须用 sudo）
sudo /Applications/windsurf-account-manager.app/Contents/MacOS/windsurf-account-manager
```

```
注意事项：
  - 每次启动都需要通过终端执行上面的 sudo 命令
  - 不能直接双击 .app 或从启动台打开（会提示已损坏）
  - 建议将启动命令保存为终端别名，方便快速启动：

    # 在 ~/.zshrc 或 ~/.bashrc 中添加：
    alias wam='sudo /Applications/windsurf-account-manager.app/Contents/MacOS/windsurf-account-manager'

    # 之后只需在终端输入：
    wam
```

### Linux

| 文件类型 | 适用架构 | 说明 |
|:--------:|:--------:|:-----|
| `*_amd64.deb` | x64 | Debian / Ubuntu |
| `*_arm64.deb` | ARM64 | Debian / Ubuntu ARM 版 |
| `*_x86_64.rpm` | x64 | Fedora / CentOS / RHEL |
| `*_aarch64.rpm` | ARM64 | Fedora ARM 版 |
| `*_amd64.AppImage` | x64 | 免安装通用格式 |

```bash
# deb 安装
sudo dpkg -i windsurf-account-manager_*.deb

# rpm 安装
sudo rpm -i windsurf-account-manager-*.rpm

# AppImage 运行
chmod +x windsurf-account-manager_*.AppImage
./windsurf-account-manager_*.AppImage
```

<img src="https://raw.githubusercontent.com/andreasbm/readme/master/assets/lines/rainbow.png" alt="line" />

## 技术栈

```
Frontend    Vue 3 + TypeScript + Element Plus + Pinia
Backend     Rust (Tauri 2.x)
Build       Vite + Cargo
Platform    Windows / macOS / Linux (x64 + ARM64)
```

<img src="https://raw.githubusercontent.com/andreasbm/readme/master/assets/lines/rainbow.png" alt="line" />

## 联系作者

```
微信        1837620622（传康Kk）
邮箱        2040168455@qq.com
咸鱼/B站    万能程序员
```

<img src="https://raw.githubusercontent.com/andreasbm/readme/master/assets/lines/rainbow.png" alt="line" />

<div align="center">
<sub>本工具仅供学习研究使用，请遵守 Windsurf 服务条款。使用本工具产生的任何后果由使用者自行承担。</sub>
</div>
