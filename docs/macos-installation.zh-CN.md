# macOS 安装指南

Antigravity Agent 是未签名的开源应用，macOS 可能会阻止运行。

## 快速解决

下载后在终端运行：

```bash
xattr -cr ~/Downloads/Antigravity*.dmg
```

或安装后运行：

```bash
sudo xattr -rd com.apple.quarantine /Applications/Antigravity\ Agent.app
```

## 替代方法：系统设置

1. 尝试打开应用（会被阻止）
2. 打开 **系统设置 → 隐私与安全性**
3. 点击底部的 **"仍要打开"**
4. 输入密码

> **注意**：从 macOS Sequoia 15.1 开始，Control-点击 → 打开 已失效

## 仍无法安装？

[提交 Issue](https://github.com/MonchiLin/antigravity-agent/issues)，附上 macOS 版本和错误信息。
