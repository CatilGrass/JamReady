# 🎮 JamReady - 提速你的 GameJam 🚀🚀🚀🚀🚀

  ![](https://img.shields.io/github/stars/CatilGrass/JamReady?style=flat-square)  ![](https://img.shields.io/badge/CLI-v0.1.0-blue?style=flat-square)  ![](https://img.shields.io/badge/License-MIT-yellow?style=flat-square)  ![](https://img.shields.io/badge/GUI-In_Development-orange?style=flat-square)



## 简述

> ​	该项目诞生于 GameJam 活动之后的一次复盘讨论中，我们团队注意到 Git、SVN 等版本控制流程并不适合小型快速的项目开发。于是，为了追求更简单的版本控制，`JamReady` 项目便应运而生。

​	JamReady 是一套围绕着其设计的 VCS 系统而产生的工具集合，旨在提供更多能在短时间内迭代开发游戏 Demo 的工具。



## 📑版本控制系统 - JamReady VCS📑

> JamReady VCS 的核心部分是以 Cli 程序呈现的，而真正的可视化操作部分在 JamReady GUI。

​	`JamReady VCS` 是该软件的核心部分，其所有的工具都是围绕该 VCS 产生。

​	此 VCS 面向 *迭代周期更短、更追求团队协作*的项目，因为是从 `SVN` 的设计思路上**简化**的模式，所以对于没接触过版本控制的成员，它能更快地上手。

以下是 JamReady VCS 的概念

### 💾工作区 - Workspace💾

1. `工作区`管理着项目中的所有`虚拟文件`、`版本`以及其`历史提交记录`。

2. `成员`可以通过`工作区`**创建**、**编辑**、**审阅**和**调用**其中的`虚拟文件`。

### 📁虚拟文件 - Virtual File📁

1. `工作区`中的*文件地址*、**指向**`工作区`中*可以被访问的文件*。
2. 版本的**更新**会使`工作区`的`本地文件`更新，并会重新将`虚拟文件`**指向**`更新的文件`。
3. `虚拟文件`的**路径更改**不会影响到*正在访问该文件的成员*。因为`虚拟文件`的位置是以`Uuid`的形式表示**的。

### 📝虚拟文件的增查删改📝

```bash
# 创建 Textures/T_Player.png 并获得其修改权
jam new "Textures/T_Player.png" -g

# 尝试获得 Textures/T_Player.png 的修改权并尝试删除该文件
jam rm "Textures/T_Player.png" -g

# 获得文件修改权并移动(重命名)文件
jam mv "Textures/T_Player.png" "Textures/T_Player_BaseColor.png" -g

# 下载并在本地查阅 Textures/T_Player.png
jam v "Textures/T_Player.png"

# 提交所有的本地更改到工作区，此时其他成员将会查看到您的新版本
jam cmt

### 另外，若删除了文件且需要还原该文件时，可以使用 Uuid 还原该文件
jam mv "c6727632-ff49-4fba-85e4-56a4984cb174" "Textures/T_Player.png" -g
```

### 🔒锁定系统🔒

​	不同于 SVN 的`排他锁`或`非排他锁`，`JamReady VCS` 所采用的方案如下：

1. `虚拟文件`仅对`持有锁的成员`视为*可读写*，其他人皆为*只读*
2. `锁`在**单次编辑**后就会被**自动释放**，成员可以使用`长期锁`来*保持该锁的所有权*
3. 对于任何生命周期的`锁`，`Leader` 身份的成员都可以**直接释放**掉该`虚拟文件`的`锁`，但 `Leader` 无法在编辑时**自动释放该锁**，避免了 `Leader` 的**误操作**

```bash
# 尝试拿到某个文件的锁
jam g "Textures/T_Player.png"

# 尝试拿到某个文件的锁 (长期持有)
jam g "Textures/T_Player.png" -l
```



## 本地构建 & 运行

​	若您有将项目在本地构建、打包的需求，请确保您的计算机中安装了以下环境

1. Cargo + Rust 环境 [[安装]](https://www.rust-lang.org/learn/get-started)
2. .NET SDK 9 (客户端部分) [[安装]](https://dotnet.microsoft.com/en-us/download/dotnet/9.0)



### 一、运行客户端界面

```bash
# 在根目录运行如下命令
dotnet run --project app/JamReadyApp/JamReadyWorkspace
```



### 二、发布项目

```bash
# 在根目录运行如下命令
# 构建 JamReady CLI 部分代码
cargo build_release

# 构建 JamReady GUI 部分代码 (可选)
dotnet publish app/JamReadyApp

# 发布项目
cargo release
```







