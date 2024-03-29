# dcli
数据库连接工具

## 概述

dcli 是一个简单的数据库管理工具。因为个人习惯喜欢用命令行，在平时工作中经常需要通过 mysql-client 连接到多个 mysql 数据库，每次连接都需要敲一长串参数或在历史记录中查找之前输入参数。我希望有一个可以替我保管多个 mysql 连接信息，在需要时指定连接名称就能连上数据库的工具，dcli 由此而来。

**注意: dcli 目前还使用明文保存密码!!!**

## 特性

### 无 mysql-client 和 openssl 依赖

不喜欢在换了一台机器后需要安装额外的 mysql-client 依赖, 特别是 SSL 连接使用的 openssl, 有时候安装 openssl 本身就是一个大麻烦。所以 dcli 使用了纯 rust 实现的 mysql 连接工具 sqlx, 而且最近版本的 sqlx 可以通过 `rustls` 特性使用 rustls 替换 native-tls, 所以无需担心 openssl 的依赖问题🎉。

### 可调整表格样式

### 支持 i18n

通过条件编译和 cargo-i18n fluent 支持国际化，详情见[安装](#安装)。

### 更智能的 shell

mysql-client 提供的 shell 有些简陋，dcli 实现了一个基于 token 的高亮显示和关键字，数据库名称，表名和字段名自动补全，和历史搜索 shell。

### 与 jupyter backend 交互(计划中)

"执行 SQL 获取数据" -> "导出到文件" -> "jupyter notebook 导入"，这个工作流在工作中非常常见，但为什么要导出到文件呢，jupyter notebook 可以通过 jupyter protocol 与 jupyter 交互，将 shell 中保存的表格直接发送到 backend，完成导入。让你不需要再保存那么多甚至于过期的文件.

## 安装

从 crate.io 安装

因为 clap 未能支持 i18n, 所以需要通过条件编译支持 clap 帮助信息 i18n, 而程序运行时的信息输出则是通过
`$LANG` 自动获取。

`export LANG=zh_CN.UTF-8` 可以设置为中文，`export LANG=en_US.UTF-8` 则为英文。


```bash
# 默认为英文版本
cargo install --force dcli
# 安装中文版本
cargo install --no-default-features --features zh-CN --force dcli
```

debian 系可以从 github release 页面下载 dep 包, 接着使用 `dpkg` 命令安装


```bash
sudo dpkg -i dcli_<version>_amd64.deb
```

## 使用

使用 `dcli --help` 查看所有可用命令

```bash
dcli 0.0.1
数据连接工具.

USAGE:
    dcli <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    conn       使用 `mysql` 命令连接到 mysql
    exec       使用一个配置运行命令
    help       Prints this message or the help of the given subcommand(s)
    profile    配置相关命令
    shell      运行连接到 mysql 的 shell
    style      显示样式相关命令
```

### 添加一个连接配置

dcli 将配置文件保存在 `~/.config/dcli.toml` 文件中, 一般情况下你不需要手动修改它。

最开始需要添加一个 MySQL 连接配置，通过 `dcli profile add <配置名>` 添加，可以通过 `--port` 等参数设置端口等信息。

dcli 支持 SSL 连接，默认情况下 dcli 不会尝试进行 SSL 连接，如果需要强制使用 SSL, 通过 `--ssl-mode` 设置 SSL 模式，可选项为 "Disabled", "Preferred", "Required", "VerifyCa", "VerifyIdentity"。

当使用 "Required" 或更高级别的 SSL mode 时需要通过 `--ssl-ca` 指定证书才能连接成功。


```bash
dcli-profile-add 0.0.1
添加一个配置

USAGE:
    dcli profile add [FLAGS] [OPTIONS] <name>

FLAGS:
    -f, --force      是否强制覆盖
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --db <db>                数据库名称
    -h, --host <host>            数据库 hostname, IPv6地址请使用带'[]'包围 [default: localhost]
    -p, --password <password>    密码
    -P, --port <port>            数据库 port 0 ~ 65536 [default: 3306]
        --ssl-ca <ssl-ca>        SSL CA 文件路径
        --ssl-mode <ssl-mode>    SSL 模式
    -u, --user <user>            用户名

ARGS:
    <name>    配置名称
```

### 执行 SQL

添加一个配置后我们就可以通过这个配置连接到 MySQL 执行命令。

如果你只想执行单个 SQL 语句，那么你可以使用 `exec` 命令

```bash
dcli-exec 0.0.1
使用一个配置运行命令

USAGE:
    dcli exec --profile <profile> [command]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -p, --profile <profile>    配置名

ARGS:
    <command>...    命令
```

假设我们添加了名为 "dev" 的配置，想查看该数据中的所有表，可以通过以下命令

```bash
dcli exec -p dev show tables;
┌───────────────────┐
│ Tables_in_default │
╞═══════════════════╡
│ _sqlx_migrations  │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ boxercrab         │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ todos             │
└───────────────────┘
```

输出表格默认为 "utf8full"模式, 可以通过 `dcli style table <样式名>` 配置，可选项为

AsciiFull AsciiMd Utf8Full Utf8HBorderOnly

### 使用默认的 mysql-client 连接到数据库

如果你安装了 mysql-client 且希望使用原生的 mysql shell, 可以通过 `dcli conn -p <配置名>` 使用它。

### 使用无依赖的 shell(开发中...)

如果你不希望使用原生 mysql shell, 且渴望语法高亮等特性，可以尝试使用 `dcli shell <配置名>` 启动一个 dcli
实现的 shell。这个 shell 不依赖 mysql-client 和 openssl，这意味着你不需要安装额外的依赖也能连接到 mysql。

但 dcli 属于早期阶段，所以很多功能仍然不完整，如有问题请开 ISSUE。

#### 使用 `dcli plan` 运行一个 http 服务器

如果你有多个 SQL 语句需要共享，你可以使用 `plan` 子命令启动一个 http 服务，并将所有 SQL 作为一个 http 接口。

首先你需要定义一个 `toml` 文件，可以参考 [plan.toml](./plan.toml)

```toml
# http 接口前置路由，对启用代理或 url 冲突时非常有用
prefix = "api"

[[queries]]
# 设置此 SQL 使用哪个数据库连接配置
profile = "xxx"
# 对应的 SQL 语句
sql = "select * from xxxx"
# 此 SQL 对应的 URL 地址，**不能以 `/` 包围**
url = "some_url"
# 此 SQL 描述
description = "一些有用的描述"
# 非必填，是否分页，如果原 SQL 中最外层含有 `limit` 或 `offset` 则分页不起作用
# 默认开启，可以设置 false 关闭
paging = true
```

接着运行 `dcli plan plan.toml`，dcli 会在 3030 端口启动 http 服务，打开网页会看到 swagger ui, 按照文档浏览使用即可

![index](./docs/assets/swagger_demo.png)


### 设置语言

默认情况下 dcli 会尝试读取本地语言设置,自动设置语言. 如果这不和预期, 可以试用

`dcli style lang` 命令设置语言. 目前支持 `zh-CN` 和 `en-US` 两种语言.

### 其他命令

dcli 使用 structopt 构建命令工具，当你有疑问时可以运行 `dcli help <子命令>` 查看帮助信息。

