# Easynote 项目规范

## Git 提交规范

commit message 必须严格遵循以下格式，不允许省略 type 前缀：

```
<type>: <concise description in English>
```

### type 必须是以下之一：
- `feat` — new feature
- `fix` — bug fix
- `refactor` — refactor without changing behavior
- `docs` — documentation changes
- `chore` — build, deps, config, etc.

### 正确示例：
- `fix: correct FriendShipService typo and update references`
- `feat: add WebSocket heartbeat mechanism`
- `refactor: unify error handling with AppError`
- `docs: update development progress`

### 错误示例（禁止）：
- `修复好友系统服务名称拼写错误` （missing type prefix）
- `fix bug` （too vague）
- `Fix: add heartbeat` （type must be lowercase）

### 其他规则：
- 不要用 `git add .` 或 `git add -A`，按文件名逐个添加
- 提交前先 `git diff` 确认变更内容
- 不要提交 `.env`、`target/` 等敏感或构建产物文件
