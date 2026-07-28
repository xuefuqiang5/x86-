# X86_os 在 macOS QEMU 上的启动问题复盘

## 1. 背景

X86_os 最初在 Linux 环境中开发，构建脚本、工具链和启动流程都依赖原开发机配置。将项目迁移到 Apple Silicon macOS 后，需要使用交叉编译器生成 32 位 x86 ELF 文件，并通过 `qemu-system-i386` 运行。

迁移过程中，系统最初只能执行 MBR。经过逐步修复后，串口标记依次推进到：

```text
L -> P -> D -> E -> X -> Y
```

这些字符分别表示：

| 标记 | 含义 |
|---|---|
| `L` | 已进入 loader 实模式代码 |
| `P` | 已进入 32 位保护模式 |
| `D` | `print_string` 已返回，准备读取内核 |
| `E` | 内核磁盘读取完成 |
| `X` | ELF 内核段复制完成 |
| `Y` | 即将跳转到内核入口 |

最终系统成功进入内核主循环，时钟中断 `IRQ 0x20` 持续触发。

---

## 2. 构建环境问题

### 2.1 构建脚本包含 Linux 绝对路径

原始 `Makefile` 中硬编码了类似下面的路径：

```text
/home/xuefuqiang5/X86_os/...
```

这些路径在 macOS 上不存在，因此磁盘镜像写入和 QEMU 启动命令无法直接使用。

### 2.2 Apple Silicon 不能直接生成目标格式

项目目标是 32 位 x86 裸机程序，而 Apple Silicon 使用 ARM64。macOS 自带编译器和链接器也不适合直接生成项目所需的 ELF32/i386 裸机内核。

因此需要使用：

```text
x86_64-elf-gcc -m32
x86_64-elf-ld -m elf_i386
x86_64-elf-ar
nasm -f elf32
qemu-system-i386
```

### 2.3 macOS 系统 `ar` 无法正确处理 ELF 对象

macOS 自带的 `ar` 面向 Mach-O 工具链。项目中的 `.o` 文件是 ELF32 格式，混用系统 `ar` 会导致静态库格式或索引错误。

修复方法是统一使用交叉工具链中的：

```text
x86_64-elf-ar
```

### 2.4 构建依赖不完整

早期顶层 `Makefile` 只判断 `bin/mbr.bin` 和 `bin/loader.bin` 是否存在，没有正确追踪：

```text
loader/mbr.asm
loader/loader.asm
loader/boot.inc
```

修改源文件后直接执行 `make`，可能不会重新生成二进制文件，导致 QEMU 继续运行旧代码。这使调试结果看起来不稳定。

修复后，源文件和 `boot.inc` 的变化都会触发重新汇编。

---

## 3. GCC 15 暴露出的 C 代码问题

新版本 GCC 的类型检查更加严格，原项目中一些旧式写法会产生错误或警告。

### 3.1 等待线程参数的指针层级错误

原代码：

```c
ioq_wakeup(i->consumer);
```

`ioq_wakeup` 需要能够修改保存在线程槽位中的指针，因此参数应为二级指针：

```c
ioq_wakeup(&i->consumer);
```

### 3.2 函数声明缺少参数

原声明：

```c
void set_intr_status();
```

这不是一个明确的无参数原型，而是旧式的“参数未指定”声明。实际函数需要接收中断状态：

```c
void set_intr_status(uint32_t status);
```

### 3.3 字符串使用指针比较

原代码：

```c
if (name != "main")
```

这比较的是两个字符串指针的地址，而不是字符串内容。不同编译和链接条件下，即使内容均为 `"main"`，地址也不一定相同。

修复后改为逐字符比较。

### 3.4 整数和指针之间的隐式转换

内核大量使用 32 位物理地址和虚拟地址。较新的 GCC 不再接受某些隐式整数/指针转换。

修复时通过 `uintptr_t` 明确表示“足以保存指针值的无符号整数”，例如：

```c
(uint8_t *)(uintptr_t)physical_address
```

这既消除了编译错误，也明确表达了裸机地址转换的意图。

---

## 4. MBR 的基础环境问题

### 4.1 栈段设置错误

原 MBR 使用：

```asm
mov ax, 0x7c00
mov ss, ax
```

这会把栈段基址设置为 `0x7c000`，并不是通常期望的物理地址 `0x7c00`。

正确方式是：

```asm
cli
xor ax, ax
mov ss, ax
mov sp, 0x7c00
```

即栈段基址为零，栈顶偏移为 `0x7c00`。

`cli` 也非常重要：在保护模式 IDT 尚未建立前，不应允许硬件中断进入旧的实模式中断表。

### 4.2 VGA 字符串使用了错误的段寄存器

`lodsb` 默认从 `DS:SI` 读取，而 VGA 文本内存位于 `0xB8000`。原代码混用了 `DS`、`ES` 和段覆盖前缀，可能从错误地址读取字符串或向错误地址写屏幕。

修复后的约定是：

```text
DS:SI -> 启动代码中的字符串
ES:DI -> VGA 文本内存
```

---

## 5. ATA PIO 磁盘读取问题

这是迁移过程中最容易误判的部分。

### 5.1 发出命令后立即读取状态

向 ATA 命令端口写入 `READ SECTORS` 后，QEMU 模拟设备需要时间更新状态。如果立即读取数据端口，可能在 `BSY` 尚未清除或 `DRQ` 尚未置位时读到无效数据。

正确等待条件是：

```text
BSY == 0
ERR == 0
DRQ == 1
```

驱动器选择后还需要约 400ns 延时。实现中通过读取备用状态端口 `0x3F6` 四次完成。

### 5.2 使用 `CX` 延时破坏扇区计数

初期修复使用：

```asm
mov cx, 4
loop delay
```

但 `CX` 同时保存待读取的扇区数。延时循环结束后，原扇区计数已经丢失。

虽然可以通过 `push cx`/`pop cx` 保护，但最终实现直接使用独立寄存器保存扇区数，避免一个寄存器承担多种职责。

### 5.3 多扇区命令的 DRQ 竞态

ATA PIO 多扇区读取不是一次等待后连续读取所有数据。每个扇区都需要等待一次新的 DRQ。

如果只等待一次，然后连续读取：

```text
扇区数 × 256 个 word
```

QEMU 可能在扇区边界返回旧数据或无效数据。

最终采用更保守、也更容易验证的方式：

1. 每次只向 ATA 控制器请求一个扇区。
2. 等待 `BSY=0`、`DRQ=1`。
3. 读取 256 个 word。
4. 等待当前扇区完成，即 `BSY=0`、`DRQ=0`。
5. LBA 和目标地址递增。
6. 再发出下一次单扇区读取命令。

MBR 读取 loader 和 loader 读取内核都使用同样的流程。

---

## 6. `print_string` 返回异常的表象

最初串口只能看到：

```text
LP
```

因此看起来像是：

```asm
call print_string
```

没有正常返回。

函数本身确实存在寄存器恢复顺序错误：

```asm
push eax
push edi
...
pop eax
pop edi
```

栈是后进先出，因此正确顺序应为：

```asm
pop edi
pop eax
```

不过，这并不是启动停在 `LP` 的唯一原因。更关键的问题是 loader 的跨扇区代码已被破坏，导致 `print_string` 使用的代码或字符串内容不可靠。

换言之，`call/ret` 只是故障表现，根因仍在 loader 镜像的内存内容。

---

## 7. 最关键的根因：`total_mem_addr` 覆盖 loader 代码

### 7.1 地址布局发生冲突

loader 从物理地址 `0x900` 开始加载。其第二个 512 字节扇区从：

```text
0x900 + 0x200 = 0xB00
```

开始。

原项目又将 BIOS E820 检测到的总内存值保存在：

```asm
total_mem_addr equ 0xb00
```

因此下面这条指令：

```asm
mov [es:total_mem_addr], eax
```

会直接覆盖 loader 第二扇区开头的四个字节。

### 7.2 为什么这个问题很难发现

被写入的数值是 E820 探测结果。在当时的错误 E820 实现中，该值为：

```text
0x0009FC00
```

QEMU 指令跟踪显示，原本正确的指令：

```asm
mov ebx, 0x00070000
```

在内存中变成了：

```asm
mov ebx, 0xFC000000
```

这正好对应写入 `0xB00` 的内存容量字节。

结果是内核读取目标地址从 `0x70000` 变成了错误的高地址。后续表现包括：

- 内核缓冲区为空或内容随机。
- ELF 头的 `e_phoff`、`e_phnum` 看起来损坏。
- ELF 段目标地址变成随机值。
- 页表似乎偶发失效。
- QEMU 最终发生页故障和 triple fault。

这些现象都不是独立问题，而是同一次 loader 自覆盖产生的连锁反应。

### 7.3 修复方法

将总内存参数迁移到不会与下列区域重叠的地址：

- MBR：`0x7C00`
- loader：从 `0x900` 开始
- 页目录：`0x10000`
- 内核临时镜像：`0x70000`

最终使用：

```asm
total_mem_addr equ 0x8000
```

内核中的读取地址也同步修改为 `0x8000`。

---

## 8. E820 内存探测逻辑问题

原代码在 BIOS 返回后立即检查：

```asm
cmp ebx, 0
je probe_end
```

E820 使用 `EBX=0` 表示“当前返回的是最后一项”。最后一项本身仍然有效，必须先记录，再结束循环。

原逻辑会丢弃最后一个内存区域，因此只得到：

```text
0x0009FC00
```

即传统内存区域，而不是完整的 64MB。

另外，`ECX` 同时被当成：

- E820 结构长度；
- `loop` 指令计数器。

BIOS 每次调用都可能修改 `ECX`，因此不能用同一个值控制探测循环。

修复后的流程是：

1. 每次调用前重新设置 `ECX=20`。
2. 验证 BIOS 返回签名 `SMAP`。
3. 先记录当前 ARDS。
4. 再检查 `EBX` 是否为零。
5. 使用独立的 ARDS 数量计算最大可用物理地址。

修复后，64MB QEMU 配置显示：

```text
0x04000000
```

---

## 9. 分页和 ELF 装载问题

### 9.1 页表初始化

启动页目录迁移到物理地址：

```text
0x10000
```

页表初始化完成以下映射：

- 低端 1MB 的恒等映射；
- 低端 1MB 到 `0xC0000000` 的内核高地址映射；
- 页目录自映射；
- 内核后续页目录项的基础结构。

页目录位置与 loader、MBR、内核临时镜像均不重叠。

### 9.2 ELF 头“损坏”并非链接器问题

故障期间，ELF 程序头中的偏移、目标地址和大小看起来都是随机值。最初可能怀疑：

- 交叉链接器输出错误；
- ELF32 程序头布局不同；
- `kernel_init` 偏移常量错误。

实际检查表明磁盘镜像中的 ELF 文件是正确的：

```text
Class:   ELF32
Machine: Intel 80386
```

ELF 解析异常是 loader 自覆盖导致错误目标地址后产生的次生问题。

---

## 10. 内核中的 SSE 非法指令

系统成功跳转到内核后，出现了 `#UD`，故障指令为：

```asm
movdqa ... , %xmm0
```

GCC 为普通 C 数组初始化自动生成了 SSE 指令。但裸机内核没有完成以下初始化：

- CR0 中的浮点/SSE相关状态；
- CR4.OSFXSR；
- CR4.OSXMMEXCPT；
- FPU/SSE 上下文管理。

因此执行 `movdqa` 会产生非法指令异常。

目前内核并不需要 SSE，最简单且可靠的修复是在编译内核和基础库时加入：

```text
-mno-sse
-mno-sse2
-mno-mmx
-msoft-float
```

这样 GCC 只生成适合当前启动环境的整数指令。

---

## 11. 故障链总结

完整故障链如下：

```text
Linux 专用构建配置
        ↓
macOS 无法正确构建 ELF32 内核
        ↓
修复交叉工具链和 C 类型问题
        ↓
MBR/ATA 时序导致 loader 跨扇区内容不稳定
        ↓
loader 扩大后，第二扇区起始地址变成 0xB00
        ↓
E820 结果写入 0xB00，覆盖 loader 指令
        ↓
内核读取目标地址被改成 0xFC000000
        ↓
ELF 头和页表表现为随机损坏
        ↓
修复地址冲突后成功进入内核
        ↓
GCC 自动生成 SSE 指令，引发 #UD
        ↓
禁用 SSE/MMX 后稳定进入内核主循环
```

---

## 12. 最终验证

最终验证环境：

```text
Host:    Apple Silicon macOS
Target:  ELF32 / Intel 80386
Memory:  64MB
Disk:    raw IDE image
QEMU:    qemu-system-i386
```

构建命令：

```bash
make clean
make
```

图形模式运行：

```bash
make run
```

无窗口串口模式：

```bash
make run-headless
```

验证结果：

- MBR 正常执行。
- loader 正常进入保护模式。
- 磁盘逐扇区读取稳定。
- E820 正确识别 64MB。
- 分页正常启用。
- ELF 内核段正常复制。
- 成功跳转到内核入口。
- IDT、PIC、定时器、内存池、线程和键盘初始化完成。
- 时钟中断 `IRQ 0x20` 持续触发。
- 未出现页故障、非法指令或 triple fault。

屏幕最终输出包括：

```text
timer_init start
timer_init done
the total memory is:
4000000
mem_init start
mem_init done
list done
init main done
main done
keyboard has done
init print_keybuf done
start done
```

这表明系统已经完成启动并进入正常运行状态。
