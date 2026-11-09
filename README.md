# glinet-router

使用 Rust 实现的 GL.iNet 路由器资源工具。

构建和安装到本地（需先安装 Rust/Cargo 和 make）：

```sh
make build    # 编译 release 版本，输出到 target/release/glinet-router
make install  # 编译并安装到当前用户目录，默认 ~/.cargo/bin
```

请确保安装目录在 `PATH` 中；默认目录可通过 `export PATH="$HOME/.cargo/bin:$PATH"` 添加，无需 `sudo`。

查看命令和参数帮助：

```sh
glinet-router --help
glinet-router <subcommand> --help
```

主要顶层子命令：

- `gen-zsh-completion`：生成 Zsh 补全脚本。
- `check-update`：检查、下载并上传 OpenClash 更新。
- `custom-rules`：管理自定义规则，`check` 查看规则，`edit` 暂未实现。
- `openclash`：管理 OpenClash 服务。
- `config`：管理配置文件（订阅）。

自定义规则命令示例（`--host` 和 `--file` 放在二级子命令前）：

```sh
glinet-router custom-rules check
glinet-router custom-rules --host other-router --file openclash_custom_rules.list check
glinet-router custom-rules edit  # 占位命令，执行会触发 todo!()
```

路由器相关子命令（`config`、`openclash`、`custom-rules`）的 `--host` 默认值为 `router-gl-be3600`，它是传给系统 `ssh` 的主机名或别名。使用默认值前，请在 `~/.ssh/config` 配置对应主机，例如：

```sshconfig
Host router-gl-be3600
    HostName 192.168.8.1
    User root
    IdentityFile ~/.ssh/id_ed25519
```

请按实际地址和登录方式修改示例，并确保已安装系统 `ssh`、`scp`，且 `ssh router-gl-be3600` 可以连接。可通过 `--host` 指定其他主机；`check-update` 的默认上传目标 `router-gl-be3600:upload/` 也使用此 SSH 配置。
