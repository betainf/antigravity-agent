const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// 1. 读取 package.json 获取版本号
const packageJsonPath = path.join(__dirname, '../package.json');
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
const version = packageJson.version;
const vsixName = `antigravity-agent-${version}.vsix`;
const vsixPath = path.join(__dirname, '..', vsixName);

console.log(`📦 正在处理版本: ${version}`);

try {
    // 2. 执行打包
    console.log('🔨 执行 vsce package...');
    execSync('npm run vsix', { stdio: 'inherit', cwd: path.join(__dirname, '..') });

    // 3. 执行安装命令
    // 使用 antigravity 命令替代 code
    const installCmd = `antigravity --install-extension "${vsixName}" --force`;
    console.log(`🚀 安装扩展: ${installCmd}`);

    execSync(installCmd, { stdio: 'inherit', cwd: path.join(__dirname, '..') });

    console.log('✅ 安装完成！请在 VSCode 中重新加载窗口 (Ctrl+Shift+P -> Reload Window)');

} catch (error) {
    console.error('❌ 操作失败:', error.message);
    process.exit(1);
}
