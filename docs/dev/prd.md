# FH5 & FH6 语音 / 文本组合配置工具 — 需求文档

## 1. 概述

本地辅助工具，将 Forza Horizon 5 / 6 (Steam PC) 中手动替换语言资源文件的操作封装为**可回滚的一键流程**。

核心价值：**防止用户手动操作时误删、误覆盖、无法恢复。**

### 边界

- 仅读写 `media/Stripped/StringTables/` 下的文本语言包 (.zip)
- 不提供/下载/破解游戏资源，不绕过 DRM/反作弊，不修改可执行文件/存档/联网数据
- 不承诺兼容游戏更新后的文件结构变化
- 用户自行承担修改本地游戏文件的风险

---

## 2. 背景

游戏仅提供整体语言切换，无法分别选择语音和文本语言。用户需要手动将一种语言的文本资源文件覆盖到另一种语言的位置来实现（如日语语音 + 中文文本）。

手动操作的问题：不知道目录在哪、容易覆盖原始文件无法恢复、游戏更新后还原、权限/占用导致失败、不清楚当前状态。

---

## 3. 设计原则

| 原则 | 含义 |
|------|------|
| 本地优先 | 仅访问本机游戏目录，不上传任何数据 |
| 最小修改 | 只动 StringTables 下的 .zip，不碰其他任何文件 |
| 可回滚 | 所有操作前必须备份，一键恢复 |
| 透明可审计 | 操作前展示变更计划，操作后记录日志 |
| 明确风险 | 首次启动、应用前均需确认非官方性质和风险 |

---

## 4. MVP 支持范围

- **游戏**: FH5, FH6
- **平台**: Windows 10 22H2+ / Windows 11
- **渠道**: 仅 Steam 版
- **不支持**: macOS / Linux / SteamOS / Xbox App / MS Store / 主机 / 云游戏 / 盗版

---

## 5. 已验证的游戏文件结构

> 以下数据来自 2026-05-20 对实际 Steam 安装的探测。

### 5.1 Steam 探测链

```
注册表 HKCU\SOFTWARE\Valve\Steam → SteamPath
  → {SteamPath}/steamapps/libraryfolders.vdf       (VDF 格式，列出所有库路径及 app ID)
    → {library}/steamapps/appmanifest_{appId}.acf   (VDF 格式，含 installdir、language)
      → {library}/steamapps/common/{installdir}/
```

### 5.2 游戏标识

| | FH5 | FH6 |
|---|---|---|
| Steam App ID | `1551360` | `2483190` |
| installdir | `ForzaHorizon5` | `ForzaHorizon6` |
| 可执行文件 | `ForzaHorizon5.exe` | `forzahorizon6.exe`（全小写） |
| StringTables 路径 | `media/Stripped/StringTables/` | 同左 |

### 5.3 目录校验规则

有效游戏目录必须同时满足：
1. 存在对应可执行文件
2. 存在 `media/Stripped/StringTables/` 目录
3. StringTables 下至少有 2 个 `.zip` 文件
4. 不能是系统目录、用户根目录、磁盘根目录

### 5.4 文本语言包 (StringTables)

两款游戏均有 24 个语言包：

| 代码 | 显示名 | 代码 | 显示名 | 代码 | 显示名 |
|------|--------|------|--------|------|--------|
| EN | English | GB | English (UK) | FR | Français |
| DE | Deutsch | ES | Español | MX | Español (MX) |
| IT | Italiano | PT | Português | BR | Português (BR) |
| NL | Nederlands | NO | Norsk | SV | Svenska |
| DK | Dansk | FI | Suomi | PL | Polski |
| CZ | Čeština | HU | Magyar | TR | Türkçe |
| RU | Русский | EL | Ελληνικά | JP | 日本語 |
| KO | 한국어 | CHS | 简体中文 | CHT | 繁體中文 |

**关键：FH5 中 `br.zip`、`cz.zip` 为小写，其余大写；FH6 全部大写。工具必须保留原始大小写。**

文件大小：FH5 约 1.8–2.3 MB/个（总 ~49 MB），FH6 约 2.2–3.2 MB/个（总 ~70 MB）。

### 5.5 语音语言 (FMODBanks)

语音 bank 位于 `media/Audio/FMODBanks/`，命名模式 `VO_{name}_{lang}.bank`。

- FH5: BR, CN, DE, EN, ES, FR, IT, JP, KO, MX, TW（11 种）
- FH6: BR, CN, DE, EN, ES, IT, JP, KO, MX, TW（10 种，**无 FR**）

### 5.6 文本 ↔ 语音代码映射

| 文本 (StringTables) | 语音 (FMODBanks) | 说明 |
|---------------------|-----------------|------|
| CHS | CN | 简体中文 |
| CHT | TW | 繁體中文 |
| GB | EN | 英式英语无独立语音 |
| 其余 | 同代码 | BR, DE, EN, ES, FR, IT, JP, KO, MX |

**工具只操作 StringTables，不修改语音 bank。语音由 Steam/游戏语言设置决定。**

### 5.7 Steam 语言配置

`appmanifest_*.acf` 中的 `UserConfig.language` 为全小写英文名（如 `"english"`, `"schinese"`, `"japanese"`）。

---

## 6. 核心流程

### 6.1 原理

以「日语语音 + 简体中文文本」为例：

1. 用户在 Steam/游戏中将语言设为日语 → 游戏加载 JP 语音 bank + JP.zip 文本
2. 工具将 `CHS.zip` 的内容复制覆盖 `JP.zip` 的位置
3. 游戏加载 JP 语音 bank（日语语音）+ 被替换的 JP.zip（实际是中文文本）

### 6.2 应用配置流程

```
1. 检查游戏未运行（进程名检测）
2. 验证源文件和目标文件均存在且可读写
3. 创建备份快照（含原始文件和 manifest）
4. 复制源语言包到临时文件 → 校验哈希 → 替换目标文件
5. 校验替换后文件哈希
6. 写入配置记录
7. 提示用户重启游戏
```

任一步骤失败 → 停止后续 → 尝试自动回滚 → 回滚也失败则提示备份位置和手动恢复方法。

### 6.3 恢复流程

```
1. 检查游戏未运行
2. 读取备份 manifest，验证备份文件完整性
3. 展示恢复计划，用户确认
4. 将备份文件复制回游戏目录
5. 校验恢复后哈希
6. 写入恢复日志
```

---

## 7. 功能需求

### 7.1 首次启动

展示用途说明、非官方性质、风险提示，要求用户勾选确认：
- 已合法拥有游戏
- 理解仅供学习研究
- 理解可能的兼容性/账号风险
- 理解不绕过 DRM/反作弊

未确认前不可进入配置页。

### 7.2 游戏检测

**自动检测**：按 §5.1 探测链查找 FH5/FH6。

**手动选择**：用户选择游戏根目录，按 §5.3 规则校验。

### 7.3 语言资源扫描

扫描 StringTables 目录，每个 .zip 提取：语言代码、文件名、文件大小、修改时间、SHA256、可读/可写状态。

### 7.4 语言组合配置

用户选择游戏 → 语音语言 → 文本语言 → 工具生成文件映射计划并展示：
- 将被覆盖的文件
- 替换来源
- 备份位置
- 预计结果
- 是否需要先在游戏内切换语音语言

### 7.5 备份管理

备份目录：`%LOCALAPPDATA%/FHLanguageComboTool/backups/{game}/{timestamp}_{voice}_voice_{text}_text/`

```
manifest.json        ← 元数据（工具版本、游戏、路径、语言、时间、文件哈希）
original/{file}.zip  ← 被覆盖前的原始文件
```

备份失败 → 禁止继续应用。FH5 和 FH6 备份严格隔离。

### 7.6 状态检测

启动时对比目标语言包哈希与上次应用记录，显示状态：
- ✅ 已应用，文件匹配
- ⚠️ 可能已被游戏更新还原
- ❓ 文件被外部修改，状态未知
- ➖ 未应用任何配置

### 7.7 日志

本地日志记录：启动、检测结果、扫描结果、用户操作、文件变更、错误。不记录账号信息或平台凭据。

---

## 8. UI 需求

### 8.1 主界面

游戏选择（FH5/FH6）、安装目录、当前状态、语音语言选择、文本语言选择、应用按钮、恢复按钮。

### 8.2 确认页

点击应用后展示完整计划，用户确认游戏已关闭、已切换语音语言、理解非官方性质后才执行。

### 8.3 结果页

成功：备份位置 + 当前配置 + 下一步建议。
失败：失败步骤 + 原因 + 是否已回滚 + 备份位置 + 建议。

UI 优先级：安全确认清楚 > 状态可理解 > 操作可回滚 > 错误信息可执行 > 美观。

---

## 9. 异常处理

| 场景 | 处理 |
|------|------|
| 游戏运行中 | 阻止操作，提示关闭 |
| 语言资源不存在 | 提示通过游戏/平台下载 |
| 权限不足 | 提示检查目录权限 |
| 目录结构未识别 | 显示"未识别"，禁止操作 |
| 备份损坏 | 禁止使用，提示 Steam 验证完整性 |
| 重复应用同一配置 | 提示已匹配，强制执行仍需备份 |

---

## 10. 非功能需求

- **原子化文件操作**：先写临时文件 → 校验 → 替换目标，失败时保留原文件
- **性能**：语言包数量少（24 个，每个 ~2–3 MB），扫描和应用应在数秒内完成
- **分层架构**：GameDetector / ResourceScanner / LanguageMapper / BackupManager / ApplyEngine / RestoreEngine / Logger / UI

---

## 11. 数据结构

### GameProfile
```json
{
  "gameId": "fh5",
  "displayName": "Forza Horizon 5",
  "channel": "Steam",
  "steamAppId": "1551360",
  "rootPath": "E:/Program Files (x86)/Steam/steamapps/common/ForzaHorizon5",
  "resourcePath": "E:/Program Files (x86)/Steam/steamapps/common/ForzaHorizon5/media/Stripped/StringTables",
  "executableName": "ForzaHorizon5.exe"
}
```

### LanguagePack
```json
{
  "code": "CHS",
  "displayName": "简体中文",
  "fileName": "CHS.zip",
  "size": 1842797,
  "sha256": "...",
  "modifiedAt": "2026-02-28T23:01:00",
  "readable": true,
  "writable": true
}
```

### ApplyPlan
```json
{
  "gameId": "fh5",
  "voiceLanguage": "JP",
  "textLanguage": "CHS",
  "operations": [
    { "type": "backup", "from": ".../StringTables/JP.zip", "to": ".../backups/.../original/JP.zip" },
    { "type": "copy_replace", "from": ".../StringTables/CHS.zip", "to": ".../StringTables/JP.zip" }
  ]
}
```

---

## 12. 技术栈

**首选**: C# / .NET 8 + WPF + Serilog + JSON 配置 + 单文件 exe / Inno Setup

- Windows 原生生态：文件操作、注册表、进程检测、VDF 解析均成熟
- 不依赖 WebView2，减少用户环境差异
- WPF 配合现代控件库即可满足 UI 需求

**备选**: Rust + Tauri 2 + React（更现代 UI，但工程复杂度更高）

---

## 13. 验收标准

| # | 需求 | 标准 |
|---|------|------|
| A1 | 目录检测 | 自动检测 Steam 库中 FH5/FH6，手动选择可校验有效性 |
| A2 | 资源扫描 | 列出 StringTables 下所有语言包及元数据 |
| A3 | 配置生成 | 选择语音/文本后生成文件操作计划 |
| A4 | 风险确认 | 应用前展示变更计划和风险提示 |
| A5 | 自动备份 | 修改前必须成功创建备份 |
| A6 | 应用配置 | 完成文本资源替换并校验哈希 |
| A7 | 恢复 | 一键恢复最近一次修改 |
| A8 | 进程检测 | 游戏运行中阻止修改 |
| A9 | 错误处理 | 失败时显示具体步骤和原因，尝试自动回滚 |
| A10 | 日志 | 每次操作有本地日志 |
| A11 | 合规边界 | 不修改玩法/联网/存档/反作弊/可执行文件 |

---

## 14. 开发优先级

1. 文件识别准确性
2. 备份与恢复可靠性
3. 操作前确认清晰度
4. 错误处理可理解性
5. UI 美观度
6. 自动检测覆盖率

---

## 15. MVP 明确不做

macOS/Linux/SteamOS、Xbox App/MS Store 版、CLI、多备份可视化管理、自动更新、多语言 UI。

---

## 16. 后续版本

| 版本 | 内容 |
|------|------|
| v1.1 | 多备份管理、dry-run 模式、日志导出 |
| v1.2 | CLI、配置预设、Steam 多库/多磁盘增强 |
| v1.3 | 研究 Xbox App/MS Store 版可行性 |

---

## 17. 免责声明

```
本项目仅供学习和研究本地文件备份、恢复与资源映射流程使用。

本项目不是 Forza、Xbox、Microsoft、Playground Games 或 Turn 10 Studios 的官方工具。
本项目不提供、不下载、不破解任何游戏资源，不绕过 DRM 或反作弊，不修改存档、账号、
联网数据或任何影响游戏公平性的内容。

使用本工具可能导致游戏文件被修改、游戏更新后配置失效、验证完整性后被还原，或带来
其他兼容性与账号风险。请仅在你合法拥有的游戏副本上使用。
```
