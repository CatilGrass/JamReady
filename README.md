# 🎮 JamReady - 提速你的 GameJam 🚀🚀🚀🚀🚀

  ![](https://img.shields.io/github/stars/CatilGrass/JamReady?style=flat-square)  ![](https://img.shields.io/badge/CLI-v0.1.1-blue?style=flat-square)  ![](https://img.shields.io/badge/License-MIT-yellow?style=flat-square)  ![](https://img.shields.io/badge/GUI-In_Development-orange?style=flat-square)



## 简述

> ​	该项目诞生于 GameJam 活动之后的一次复盘讨论中，我们团队注意到 Git、SVN 等版本控制流程并不适合小型快速的项目开发。于是，为了追求更简单的版本控制，`JamReady` 项目便应运而生。

​	JamReady 是一套围绕着其设计的 VCS 系统而产生的工具集合，旨在提供更多能在短时间内迭代开发游戏 Demo 的工具。



## JamReady CLI

> ​	JamReady VCS 的核心逻辑构建在 JamReady CLI，若您需要查看图形界面相关内容，请前往 JamReady GUI。

​	`JamReady CLI` 主要包含了基础的 `VCS 版本控制` 部分， 

​	作为一个 `版本控制系统`，它主要面向 *迭代周期更短、更追求团队协作*的项目，我们打算从 `SVN` 的设计思路上再次做 **减法**。它对于没接触过版本控制的成员，能更快速、更轻松地上手。

### 工作区 - Workspace

1. `工作区`管理着项目中的所有`虚拟文件`、`版本`以及其`历史提交记录`。

2. `成员`可以通过`工作区`**创建**、**编辑**、**审阅**和**调用**其中的`虚拟文件`。

### 虚拟文件 - Virtual File

1. `工作区`中的*文件地址*、**指向**`工作区`中*可以被访问的文件*。
2. 版本的**更新**会使`工作区`的`本地文件`更新，并会重新将`虚拟文件`**指向**`更新的文件`。
3. `虚拟文件`的**路径更改**不会影响到*正在访问该文件的成员*。因为`虚拟文件`的位置是以`Uuid`的形式表示的。



CLI 快速入门 : [Quick Start](docs/learn-cli/quick-start_zh_cn.md)



## 本地构建 & 运行

​	若您有将项目在本地构建、打包的需求，请确保您的计算机中安装了以下环境

1. Cargo + Rust 环境 [[安装]](https://www.rust-lang.org/learn/get-started)
2. .NET SDK 9 (客户端部分) [[安装]](https://dotnet.microsoft.com/en-us/download/dotnet/9.0)



### 一、运行客户端界面 (开发中)

```bash
# 在根目录运行如下命令 (暂不支持)
# dotnet run --project app/JamReadyApp/JamReadyWorkspace
```



### 二、发布项目

```bash
# 在根目录运行如下命令
# 构建 JamReady CLI 部分代码
cargo build_release

# 构建 JamReady GUI 部分代码 (暂不支持)
# dotnet publish app/JamReadyApp

# 发布项目
cargo release
```







