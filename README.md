# X86_os

一个以 Rust `no_std` 实现的 32 位 x86 教学内核。MBR、保护模式 loader、中断入口和线程上下文切换保留少量 x86 汇编，其余内核模块均使用 Rust。

## 目录

```text
boot/                 MBR、loader 和分页初始化
kernel/
  arch/x86/           中断入口、上下文切换和底层 CPU 辅助汇编
  src/                Rust 内核源码
  Cargo.toml
  linker.ld
targets/              Rust i686 自定义目标
Makefile              构建、写盘和 QEMU 启动入口
```

构建产物位于 `bin/`，磁盘镜像位于 `disk_img_file/`；二者均不会提交到 Git。

## 构建与运行

依赖：Rust nightly、NASM、`x86_64-elf-ld` 和 `qemu-system-i386`。

```sh
make build
make run
```

首次构建会自动创建 60 MiB 的磁盘镜像。清理所有编译产物：

```sh
make clean
```
