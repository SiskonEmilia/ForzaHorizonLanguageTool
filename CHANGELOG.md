# Changelog

All notable changes to this project are documented here. / 本项目的重要变更记录于此。

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/).

## [1.2.0] - 2026-05-29

### Added / 新增
- **Text pack update detection** — when a game update refreshes the source text pack while your override is still in place, the main page now shows an *Outdated* status with a one-click **Re-apply** banner that refreshes the override from the latest pack. / **文本包更新检测**——当游戏更新刷新了源文本包、而你的替换仍生效时，主页会显示「需更新」状态并给出一键**重新应用**横幅，用最新文本包刷新替换。
- **Re-apply for modified state** — when the voice pack file has been changed outside the tool (e.g. a game re-download), the *Modified* status now also offers a **Re-apply** action to restore your language combo. / **「已修改」状态的重新应用**——当语音包文件被工具以外的方式改动（如游戏重新下载）时，「已修改」状态同样提供**重新应用**操作以恢复语言组合。
- **Switch UI language anytime** — a language switcher on the main page lets you change the tool's interface language after first launch, without reinstalling or clearing settings. / **随时切换界面语言**——主页新增语言切换入口，首次启动后也能更改工具界面语言，无需重装或清除设置。

### Changed / 变更
- Re-apply safely restores the true original voice pack before re-applying, so the backup's recorded original always stays correct. / 重新应用会先安全还原真正的原始语音包再重替换，保证备份记录的原始文件始终正确。
- Backups now record the applied content hash (`appliedSha256`), enabling precise *Outdated* vs *Modified* detection. Older backups remain fully compatible. / 备份现在记录已应用内容的哈希（`appliedSha256`），用于精确区分「需更新」与「已修改」；旧备份完全兼容。

### Fixed / 修复
- Confirmation-page checkboxes no longer show literal `<strong>` tags; emphasis now renders correctly, and the third checkbox is now localized. / 确认页勾选项不再显示出 `<strong>` 标签字面量，加粗正常渲染，第三条勾选项也已本地化。
- Dynamically rendered strings (dropdown placeholders, backup list, status text, etc.) are now fully internationalized and update when the UI language changes. / 动态渲染的文案（下拉占位、备份列表、状态文字等）已完整接入 i18n，并随界面语言切换实时更新。
- Synchronized the version string across the app, installer, and backup metadata. / 统一了应用、安装包与备份元数据中的版本号。

## [1.1.0] - 2026-05-20

### Fixed / 修复
- Localized the effect title and status label; passed the status label into the `status_detail` template. / 本地化了效果标题与状态标签，并将状态标签传入 `status_detail` 模板。

### Changed / 变更
- README: badges, updated preview screenshot, and bilingual (EN/中文) content. / README：徽章、更新预览截图、中英双语内容。

## [1.0.0] - 2026-05-20

### Added / 新增
- Initial public release: mix and match voice and text languages in Forza Horizon 5 / 6 (Steam, PC). / 首个公开版本：在 Steam 版《极限竞速：地平线 5/6》中自由组合语音与文字语言。
- Auto-detect Steam installations, one-click apply with automatic SHA-256-verified backup, and one-click restore. / 自动检测 Steam 安装、一键应用并自动创建经 SHA-256 校验的备份、一键恢复。
- Automatic Steam game-language setting, 24-language UI, multi-language user guides, and CI release workflow. / 自动设置 Steam 游戏语言、24 种语言界面、多语言使用指南、CI 发布流程。

[1.2.0]: https://github.com/SiskonEmilia/ForzaHorizonLanguageTool/releases/tag/v1.2.0
[1.1.0]: https://github.com/SiskonEmilia/ForzaHorizonLanguageTool/releases/tag/v1.1.0
[1.0.0]: https://github.com/SiskonEmilia/ForzaHorizonLanguageTool/releases/tag/v1.0.0
