---
title: 快速开始
description: 安装 Vizsla 扩展并确认核心 IDE 功能可用。
---

请根据以下步骤，快速开始体验 Vizsla。

## 1. 安装扩展

在 VS Code 的扩展面板中搜索显示名 `Vizsla` 并安装即可。

## 2. 打开工程目录

用 VS Code 打开包含 RTL 源码的目录。没有 `vizsla.toml` 或旧版 `vizsla_config.toml` 时, 扩展会创建默认 `vizsla.toml` 并弹出提示:

```toml
#:schema https://pascal-lab.github.io/vizsla/schemas/v1/vizsla.schema.json
sources = []

# include_dirs = ["include"]
# defines = ["SYNTHESIS"]
# top_modules = ["top"]
# libraries = ["../common_cells"]
# exclude = ["build/**"]
```

这个默认项目配置文件显式写入 `sources = []`, 因此不会扫描 workspace 下的源文件, 也不会建立编译 profile 或运行跨文件 semantic diagnostics。需要跨文件跳转、引用和语义诊断时, 请把 `sources` 改成符合工程结构的 shell glob, 例如 `sources = ["rtl/**"]`, 并按需补充 `include_dirs`, `defines`, `libraries` 或 `top_modules`。

## 3. 确认状态栏

扩展激活后, VS Code 右侧状态栏会出现名为 `Vizsla` 的状态项。状态项通常显示 `Vizsla`; 启动或停止时会带旋转图标, 项目配置缺失时会带 warning 图标, 服务器或项目配置失败时会带 error 图标。

将鼠标悬停在状态项上可以查看当前详情。点击状态项, 或执行 `Vizsla: Show Status`, 会打开状态菜单; 这里可以打开或创建项目配置文件、重新加载项目配置、重启语言服务器、运行 diagnostics profiling, 或打开 `Vizsla Language Server` 输出通道。

## 4. 打开 Verilog/SystemVerilog 文件

打开 `.v`, `.vh`, `.sv`, `.svh` 或 `.svi` 文件。VS Code 应该把它识别为 Verilog 或 SystemVerilog, 并启用语法高亮和语言服务。

## 5. 试用核心功能

你可以按这个顺序验证:

1. 写一处明显语法错误, 查看 `Problems` 面板中的诊断。
2. 在模块名、信号名或实例名上执行 `Go to Definition` 或 `Go to Declaration`。
3. 在实例端口连接、参数赋值、表达式或预处理位置触发补全。
4. 把光标放到符号上查看悬停说明。
5. 执行 `Format Document`。默认格式化会调用 `verible-verilog-format`, 如果本机没有这个工具, 请配置 `vizsla.formatter.path` 或先跳过格式化验证。

配置变更后, 如果 VS Code 提示重启语言服务器, 选择 `Restart`。

需要跨文件 semantic diagnostics 和更准确的端口/参数相关能力时, 请在 `vizsla.toml` 中写入实际的 `sources` 或 `include_dirs`, 并按需补充 `defines`, `libraries` 或 `top_modules`。
