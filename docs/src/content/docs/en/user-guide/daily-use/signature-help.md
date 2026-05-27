---
title: Signature Help
description: Inspect target module parameters, ports, and the current connection position inside instance lists.
---

Signature help appears in instance parameter assignment lists and port connection lists. It is different from completion: completion inserts candidates at the current position, while signature help shows which parameter or port you are currently filling in.

Signature help options are VS Code Settings. Search for `Vide Signature Help` in the Settings UI, or write `vide.signature.help.params.only` in user or workspace `settings.json`; see [Signature Help](../../../advanced-guide/vscode-settings/#signature-help) for the full reference.

## When It Appears

- In parameter assignment lists `#(...)`, it shows target module parameters.
- In port connection lists `(...)`, it shows target module ports.
- As the cursor moves across parameter or port positions, the active item updates.

## Resolution Scope

Signature help needs the target module to resolve. Like navigation and completion, it depends on the `sources`, `include_dirs`, `defines`, and `libraries` in the project view.

Enable `vide.signature.help.params.only` if you only want parameter-related signature help.
