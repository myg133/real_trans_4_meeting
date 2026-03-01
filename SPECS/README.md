# 项目规范索引 (Specifications Index)

**项目名称**: 全双工音频处理程序 (trans)  
**版本**: 0.1.0  
**最后更新**: 2026-03-01

---

## 规范文档列表

| 文档 | 说明 | 目标读者 |
|------|------|----------|
| [DESIGN.md](./DESIGN.md) | 系统架构和设计规范 | 架构师、开发者 |
| [DEVELOPMENT.md](./DEVELOPMENT.md) | TDD 开发流程和规范 | 所有开发者 |
| [PROCESSOR-API.md](./PROCESSOR-API.md) | 音频处理器 API 规范 | 处理器开发者 |
| [TESTING-GUIDE.md](./TESTING-GUIDE.md) | 测试指南和示例 | 测试开发者 |

---

## 快速入门

### 新开发者

1. 阅读 [README.md](../README.md) 了解项目概述
2. 阅读 [DESIGN.md](./DESIGN.md) 了解系统架构
3. 遵循 [DEVELOPMENT.md](./DEVELOPMENT.md) 进行 TDD 开发
4. 参考 [TESTING-GUIDE.md](./TESTING-GUIDE.md) 编写测试

### 开发新处理器

1. 阅读 [PROCESSOR-API.md](./PROCESSOR-API.md) 了解 API 规范
2. 按照 [DEVELOPMENT.md](./DEVELOPMENT.md) 的 TDD 流程开发
3. 参考 [TESTING-GUIDE.md](./TESTING-GUIDE.md) 编写测试用例

### 提交代码前检查

```bash
# 1. 格式化
cargo fmt

# 2. 检查编译
cargo check

# 3. 运行测试
cargo test

# 4. 构建发布版本
cargo build --release
```

---

## 核心规范摘要

### TDD 流程（强制）

```
RED → GREEN → REFACTOR

1. RED: 先写失败的测试
2. GREEN: 编写最小实现让测试通过
3. REFACTOR: 重构优化代码
```

### 测试覆盖率要求

| 模块 | 行覆盖率 | 分支覆盖率 |
|------|----------|------------|
| `processor` | 100% | 100% |
| `config` | 90% | 90% |
| `audio_io` | 80% | 75% |
| `main` | 70% | 60% |

### 代码规范

- 遵循 Rust API Guidelines
- 使用 `rustfmt` 格式化
- 函数不超过 50 行
- 参数不超过 5 个
- 必须实现 `Send + Sync`（音频处理器）

### Commit Message 格式

```
<type>(<scope>): <description>

类型：feat, fix, docs, style, refactor, test, chore
```

---

## 相关资源

- [MEMORY.md](../MEMORY.md) - 项目记忆文档
- [README.md](../README.md) - 项目说明
- [Cargo.toml](../Cargo.toml) - 项目依赖配置

---

## 文档维护

- 所有规范文档使用 Markdown 格式
- 文档更新需要随代码一起提交
- 重大变更需要更新版本号
