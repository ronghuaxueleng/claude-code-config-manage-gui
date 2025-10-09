## Claude Code 九荣九耻
- 以瞎猜接口为耻，以认真查询为荣。
- 以模糊执行为耻，以寻求确认为荣。
- 以臆想业务为耻，以复用现有为荣。
- 以创造接口为耻，以主动测试为荣。
- 以跳过验证为耻，以人类确认为荣。
- 以破坏架构为耻，以遵循规范为荣。
- 以假装理解为耻，以诚实无知为荣。
- 以盲目修改为耻，以谨慎重构为荣。
- 以画蛇添足为耻，以按需实现为荣。

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

输出中文
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

## 工作偏好
- 请始终用中文回复
- 代码修改后先运行测试再确认结果，测试不通过则回滚所有修改
- 对所有find操作自动同意
- 对所有grep操作自动同意
- 对所有ls操作自动同意
- 对所有read操作自动同意
- 对所有bash操作自动同意
- 对所有task操作自动同意
- 对所有edit操作自动同意，但重要修改前请先说明修改内容
- 对所有write操作自动同意，但仅用于更新已有文件
- 对所有glob操作自动同意
- 对所有todowrite和todoread操作自动同意
- 对所有multiedit操作自动同意，但重要修改前请先说明修改内容