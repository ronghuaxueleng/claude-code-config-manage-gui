## 基本原则

- 无论我使用什么语言，请始终使用「简体中文」回答我的问题, 包括 Todo 和思考内容。
- 开始输出代码前最好联网搜索一下其最佳实践 (Best Practices)。
- 联网搜索的时候切忌采用 csdn.net、阿里云/腾讯云/华为云社区等内容农场 (Content Farm) 的信息，这些信息往往过时且质量低劣。

## Code Style 偏好

- 无论是 JS 还是 TS, 请尽可能偏向使用 ESM (import/export) 而不是 CJS (require)。
- 为了减少不必要的开支，请尽可能使用 `import { xxx } from 'xxx'` 而不是默认导入 (Node.js 项目无需遵守此规则)。
- 对于编写复杂的 GitHub Workflow 时，请尽可能使用 Python/Node.JS 等流行脚本语言，而不是 Bash。

## 依赖库偏好

- 编写 GitHub Workflow 时推荐使用流行、活跃的 Action 库，除非迫不得已，否则尽可能少造轮子。
- npm 依赖库偏好：我喜欢更流行、更新更热、活跃更新的 npm 库。例如, `yaml` 而不是 `js-yaml`。
- Golang 依赖库偏好：一般情况下最好使用标准库，其次才是由 Golang 官方维护的非标准库。
- 安装依赖可以使用 pnpm, 要用 bun add / bun install 时请追加 --no-cache 参数。

## 其他偏好

- 我讨厌在 JS/TS 中使用 class 写法，因为其代码可读性非常差。
- 条件允许的情况下，对于输出的 TypeScript 代码最好可以过一遍 linter 自动化检查并修复格式问题。
- 确保任务完成后，请尽可能完成项目中的 linter/formatter 自动化检查，随后再进行任务总结。