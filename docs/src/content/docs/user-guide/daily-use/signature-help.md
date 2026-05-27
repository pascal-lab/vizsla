---
title: 签名帮助
description: 在实例参数和端口列表中查看目标模块的参数、端口和当前连接位置。
---

签名帮助会在实例参数赋值列表和端口连接列表中显示目标模块的参数或端口信息。它和补全不同：补全是在当前位置插入候选项，签名帮助用于确认当前正在填写哪个参数或端口。

签名帮助选项属于 VS Code Settings。可以在设置界面搜索 `Vide Signature Help`，也可以在用户或工作区 `settings.json` 里写入 `vide.signature.help.params.only`；完整参考见 [Signature Help](../../../advanced-guide/vscode-settings/#signature-help)。

## 什么时候出现

- 在参数赋值列表 `#(...)` 中，显示目标模块参数。
- 在端口连接列表 `(...)` 中，显示目标模块端口。
- 光标移动到不同参数或端口位置时，当前项会随之更新。

## 解析范围

签名帮助需要目标模块能被解析到。它和跳转、补全一样依赖项目视图里的 `sources`、`include_dirs`、`defines` 和 `libraries`。

如果只想显示参数相关签名帮助，可以开启 `vide.signature.help.params.only`。
