# macOS Installation Guide

Antigravity Agent is an unsigned open-source app. macOS may block it from running.

## Quick Fix

Run in Terminal after downloading:

```bash
xattr -cr ~/Downloads/Antigravity*.dmg
```

Or after installing:

```bash
sudo xattr -rd com.apple.quarantine /Applications/Antigravity\ Agent.app
```

## Alternative: System Settings

1. Try to open the app (it will be blocked)
2. Go to **System Settings → Privacy & Security**
3. Click **"Open Anyway"** at the bottom
4. Enter your password

> **Note**: Control-click → Open no longer works on macOS Sequoia 15.1+

## Still not working?

[Open an issue](https://github.com/MonchiLin/antigravity-agent/issues) with your macOS version and error message.
