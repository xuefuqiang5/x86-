# X86_os 后续实现与 Rust 迁移路线

## 1. 目标

当前系统已经能够在 Apple Silicon macOS 上完成以下流程：

```text
MBR
  → Loader
  → 保护模式
  → 分页
  → ELF 内核装载
  → IDT/PIC
  → 定时器
  → 内存池
  → 内核线程
  → 键盘中断
```

下一阶段的目标不是立即重写全部代码，而是在保持系统持续可启动、持续可调试的前提下，将内核逐步迁移到 Rust，并继续实现更完整的内存管理、线程调度、用户态和系统调用。

核心原则：

1. 每个阶段结束时，系统都必须能够启动。
2. 每次只替换一个边界清晰的模块。
3. 汇编只负责 CPU 必须的入口和上下文保存。
4. Rust 负责状态管理、资源所有权和大部分业务逻辑。
5. 所有关键阶段必须能通过串口和自动化测试判断成功或失败。
6. 不在一次修改中同时更换启动协议、语言、链接布局和内存模型。

---

## 2. 推荐的总体方案

### 2.1 不立即重写 MBR 和 loader

当前 MBR 和 loader 已经完成：

- ATA PIO 磁盘读取；
- E820 内存探测；
- A20 开启；
- GDT 安装；
- 保护模式切换；
- 页表建立；
- ELF32 内核装载；
- 跳转高地址内核。

这些代码已经过 QEMU 验证。立即用 Rust 重写这部分的收益很低，却会同时引入：

- 16 位实模式支持问题；
- Rust 自定义目标问题；
- 链接地址和代码模型问题；
- BIOS 调用边界问题；
- 多阶段二进制尺寸限制。

建议长期保留：

```text
loader/mbr.asm
loader/loader.asm
kernel/isr_entry.S
kernel/switch_to.S
```

待 Rust 内核稳定后，再决定是否将 loader 更换为 Multiboot2、Limine 或自定义 Rust loader。

### 2.2 采用渐进式混合内核

推荐的过渡结构：

```text
MBR / Loader                 NASM
        │
        ▼
内核入口、ISR、switch_to      NASM
        │
        ▼
已有低层驱动和兼容层          C
        │
        ▼
新内核模块                    Rust no_std
```

Rust 首先编译成静态库，由现有 ELF32 链接流程链接进内核：

```text
libkernel_rs.a
```

C 调用 Rust：

```c
extern void rust_kernel_probe(void);
```

Rust 导出 C ABI：

```rust
#[unsafe(no_mangle)]
pub extern "C" fn rust_kernel_probe() {
    // 第一阶段只输出串口标记。
}
```

所有跨语言结构必须使用稳定 ABI：

```rust
#[repr(C)]
pub struct InterruptFrame {
    // 字段顺序必须与汇编压栈顺序完全一致。
}
```

---

## 3. 推荐的代码结构

第一阶段不需要移动现有 C 文件，只新增 Rust 工作区：

```text
X86_os/
├── Makefile
├── loader/
├── kernel/
├── lib/kernel/
├── rust/
│   ├── Cargo.toml
│   ├── rust-toolchain.toml
│   ├── targets/
│   │   └── i686-x86-os.json
│   ├── kernel-rs/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── panic.rs
│   │       ├── arch/
│   │       │   └── x86/
│   │       │       ├── mod.rs
│   │       │       ├── io.rs
│   │       │       ├── interrupts.rs
│   │       │       └── paging.rs
│   │       ├── console.rs
│   │       ├── memory/
│   │       │   ├── mod.rs
│   │       │   ├── frame.rs
│   │       │   └── bitmap.rs
│   │       ├── task/
│   │       │   ├── mod.rs
│   │       │   ├── scheduler.rs
│   │       │   └── sync.rs
│   │       └── drivers/
│   │           ├── mod.rs
│   │           ├── serial.rs
│   │           ├── vga.rs
│   │           ├── timer.rs
│   │           └── keyboard.rs
│   └── .cargo/
│       └── config.toml
├── scripts/
│   ├── run-qemu.sh
│   ├── test-qemu.sh
│   └── debug-qemu.sh
├── TROUBLESHOOTING.md
└── RUST_ROADMAP.md
```

不要在迁移初期创建大量 crate。一个 `no_std` 内核静态库已经足够。等模块边界稳定后，再拆分 `arch_x86`、`kernel_api` 等 crate。

---

## 4. Rust 构建策略

### 4.1 Rust 版本

当前本机已经安装：

```text
rustc 1.93.0
cargo 1.93.0
nightly-aarch64-apple-darwin
```

宿主机是 ARM64，但 Rust 的输出目标必须是无操作系统的 32 位 x86。

建议在仓库中固定 nightly 版本，而不是跟随本机全局默认值：

```toml
[toolchain]
channel = "nightly-2026-01-19"
components = ["rust-src", "llvm-tools-preview"]
profile = "minimal"
```

固定版本能够避免编译器升级导致自定义 target JSON 或 `build-std` 行为变化。

### 4.2 自定义目标

需要自定义 `i686-x86-os.json`，至少明确：

- `arch = "x86"`；
- 32 位指针宽度；
- little endian；
- 静态重定位；
- 无操作系统；
- panic abort；
- 禁用 red zone；
- 禁用 SSE/MMX；
- 使用 ELF32 链接器。

初期应与当前 C 内核保持相同 CPU 能力，不允许 LLVM 自动生成 XMM 指令。Rust 目标也必须明确禁用：

```text
SSE
SSE2
MMX
```

### 4.3 `no_std`

内核 crate 必须使用：

```rust
#![no_std]
```

第一阶段不要依赖堆分配，因此不需要 `alloc`。只编译：

```text
core
compiler_builtins
```

建议使用：

```bash
cargo +nightly build \
  -Z build-std=core,compiler_builtins \
  --target rust/targets/i686-x86-os.json \
  --release
```

### 4.4 Panic 策略

禁止 unwind：

```toml
[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

提供自己的 panic handler：

```rust
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    serial::write_str("RUST PANIC: ");
    // 尽可能输出位置和消息。
    qemu::exit_failure();
}
```

### 4.5 ABI 规则

跨 C/Rust 边界只使用：

- 固定宽度整数；
- 裸指针；
- `#[repr(C)]` 结构；
- `extern "C"` 函数；
- 明确的所有权约定。

不要直接跨边界传递：

- Rust 引用；
- `String`；
- `Vec`；
- trait object；
- Rust enum；
- 未标记 `repr(C)` 的结构。

---

## 5. 分阶段实现流程

## Phase 0：冻结当前可启动基线

目标：任何后续修改都能与一个已知可工作的版本比较。

任务：

1. 为当前提交打标签，例如：

   ```bash
   git tag c-kernel-qemu-baseline
   ```

2. 保存当前成功启动的串口序列：

   ```text
   LPDEXY
   ```

3. 保存当前屏幕初始化输出。
4. 建立自动启动超时，避免 QEMU 卡死拖住测试。
5. 给每个阶段定义明确的通过条件。

通过条件：

- `make clean && make` 成功；
- QEMU 不产生 triple fault；
- 内核持续接收时钟中断；
- 键盘输入能够到达消费者线程。

---

## Phase 1：先建立调试基础设施

在引入 Rust 前，先提高可观察性。

需要实现：

### 1. 串口输出函数

当前 loader 直接向 `0x3F8` 写字符。后续应实现可靠串口驱动：

1. 初始化 COM1；
2. 写字符前等待 Line Status Register 的 THR empty 位；
3. 支持十六进制整数；
4. 支持换行；
5. panic 和异常处理器都能调用。

### 2. QEMU 退出端口

测试模式下向 `isa-debug-exit` 端口写值，让自动测试能区分成功和失败：

```text
端口：0xF4
成功码：0x10
失败码：0x11
```

QEMU 参数：

```bash
-device isa-debug-exit,iobase=0xf4,iosize=0x04
```

### 3. 统一异常报告

异常处理器至少输出：

- vector；
- error code；
- EIP；
- CS；
- EFLAGS；
- ESP；
- SS；
- CR2（页故障时）；
- 当前线程名称；
- 当前线程栈魔数。

### 4. 构建产物

每次构建保留：

```text
kernel
kernel.map
kernel.sym
反汇编文本
```

通过条件：

- 人为触发 `int3` 能看到完整寄存器；
- 人为访问未映射地址能看到 CR2；
- 自动测试能通过 QEMU 退出码判断成功与失败。

---

## Phase 2：Rust 最小接入

目标：不替换任何现有模块，只证明 Rust 代码能够进入最终 ELF 并被执行。

任务：

1. 创建 `kernel-rs` 静态库。
2. 实现：

   ```rust
   #[unsafe(no_mangle)]
   pub extern "C" fn rust_kernel_probe() -> u32 {
       0x52555354
   }
   ```

3. 从 C 内核调用该函数。
4. 检查返回值并打印：

   ```text
   RUST
   ```

5. 反汇编确认没有 XMM/MMX 指令。
6. 检查最终 ELF 中存在 `rust_kernel_probe` 符号。

通过条件：

- C → Rust 调用成功；
- Rust → C 串口调用成功；
- panic handler 可以工作；
- 最终内核仍持续接收时钟中断。

这一阶段不要引入：

- allocator；
- trait object；
-复杂泛型；
-异步运行时；
-第三方驱动框架。

---

## Phase 3：用 Rust 建立硬件抽象层

优先迁移无共享状态或状态很少的模块。

推荐顺序：

1. 端口 I/O；
2. COM1 串口；
3. VGA 文本输出；
4. 中断开关；
5. CPU halt；
6. PIC；
7. PIT。

端口类型示例：

```rust
pub struct Port8 {
    port: u16,
}

impl Port8 {
    pub const unsafe fn new(port: u16) -> Self {
        Self { port }
    }

    pub unsafe fn read(&self) -> u8 {
        // asm!("in al, dx", ...)
        todo!()
    }

    pub unsafe fn write(&self, value: u8) {
        // asm!("out dx, al", ...)
        todo!()
    }
}
```

所有 `unsafe` 应限制在 `arch::x86` 层。上层驱动使用安全包装，不直接编写内联汇编。

通过条件：

- Rust 串口输出替代 C 串口输出；
- Rust VGA 输出与现有显示一致；
- PIT 频率保持不变；
- 每个端口操作都集中在少数可审计函数中。

---

## Phase 4：中断系统迁移

中断入口继续保留汇编，Rust 负责分发和处理。

推荐结构：

```text
CPU
 ↓
isr_entry.S
 ↓ 保存寄存器
rust_interrupt_dispatch(frame)
 ↓
具体 handler
 ↓
汇编恢复寄存器
 ↓
iretd
```

关键要求：

1. 中断帧必须使用 `#[repr(C)]`。
2. 明确哪些异常由 CPU 自动压入 error code。
3. 异常和硬件 IRQ 必须区分。
4. 只对硬件 IRQ 发送 EOI。
5. 从属 PIC 的 IRQ 才向从 PIC 发送 EOI。
6. 普通内核异常门使用 DPL0。
7. 只有系统调用门或显式用户入口使用 DPL3。
8. IDTR limit 必须是：

   ```text
   sizeof(IDT) - 1
   ```

通过条件：

- divide error、invalid opcode、page fault 均能正确报告；
- IRQ0 和 IRQ1 正常；
- 异常不会错误发送 EOI；
- 无中断嵌套时栈帧完全恢复。

---

## Phase 5：物理内存和分页迁移

建议先写新的 Rust 内存管理器，不要直接翻译现有 C 实现。

模块划分：

```text
BootMemoryInfo
FrameAllocator
PageTable
Mapper
KernelHeap
```

### 1. BootMemoryInfo

不要只传递一个 `total_mem`。loader 应最终传递完整 E820 map：

```rust
#[repr(C)]
pub struct MemoryRegion {
    pub base: u64,
    pub length: u64,
    pub region_type: u32,
    pub attributes: u32,
}
```

32 位内核也应以 64 位字段保存 BIOS 地址，避免丢失高位。

### 2. FrameAllocator

职责：

- 管理物理页帧；
- 跳过保留区域；
- 跳过内核、loader、页表和启动信息；
- 支持 allocate/free；
- 检测 double free；
- 检测地址越界。

### 3. Mapper

职责：

- map；
- unmap；
- translate；
- 修改权限；
- `invlpg`；
- 页表创建。

不要通过拼接指针推测 PDE/PTE 地址。将页表结构定义为明确的 1024 项数组：

```rust
#[repr(align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 1024],
}
```

### 4. Heap

FrameAllocator 和 Mapper 稳定后，再接入 `alloc`：

```rust
extern crate alloc;
```

第一版可以使用简单 bump allocator；确认生命周期和映射正确后，再实现 free-list 或 slab。

通过条件：

- 连续分配和释放页帧通过压力测试；
- 虚拟地址映射后可读写；
- unmap 后访问必然触发页故障；
- 页表本身保持 4KB 对齐；
- 内核堆能运行 `Box` 和 `Vec` 的最小测试。

---

## Phase 6：线程和调度器迁移

`switch_to.S` 暂时保留，Rust 管理线程状态和队列。

迁移前必须先修正当前上下文约定：

当前 `thread_stack` 保存：

```text
EBP, EBX, EDI, ESI
```

但 `switch_to.S` 当前压入：

```text
ESI, EDI, EDX, EBP
```

这里的 `EDX` 应与结构约定核对，通常应保存 callee-saved 的 `EBX`。在迁移调度器前必须建立一个寄存器保持测试，否则线程切换后会出现随机状态损坏。

Rust 数据结构建议：

```rust
pub enum TaskState {
    Running,
    Ready,
    Blocked,
    Sleeping,
    Exited,
}

#[repr(C)]
pub struct Task {
    pub saved_stack: *mut u32,
    // 其余字段不直接由汇编访问。
}
```

调度器第一版保持简单：

- 单 CPU；
- round-robin；
- 固定优先级时间片；
- 中断关闭时修改就绪队列；
- 不支持抢占临界区；
- 不支持用户态。

通过条件：

- 两个线程分别维护不同的寄存器哨兵值；
- 运行数十万次时钟切换后哨兵值不变；
- 阻塞线程不再获得 CPU；
- 唤醒后线程从原调用点继续；
- 空闲时使用 `hlt`，而不是忙循环。

---

## Phase 7：同步原语和设备输入

在调度器稳定后迁移：

1. SpinLock；
2. Semaphore；
3. Mutex；
4. WaitQueue；
5. RingBuffer；
6. Keyboard driver。

中断上下文不能睡眠。需要明确区分：

```text
lock_irqsave
spin_lock
sleeping_mutex
```

Rust 所有权可以避免部分悬空指针，但不能自动解决：

- 中断重入；
- 锁顺序；
- 死锁；
- 丢失唤醒；
- 内存映射失效；
- 错误的裸指针别名。

通过条件：

- 键盘 IRQ 只负责读取扫描码并放入 ring buffer；
- 解码和输出由线程完成；
- buffer 满时行为明确；
- producer/consumer 压力测试无丢失唤醒。

---

## Phase 8：用户态和系统调用

不要在内核内存管理和调度器稳定前进入用户态。

实现顺序：

1. TSS；
2. 用户代码段和数据段；
3. 每线程内核栈；
4. 用户页目录；
5. ring3 测试程序；
6. `int 0x80` 或其他系统调用入口；
7. 用户指针检查；
8. 进程退出；
9. 基础文件描述符抽象。

首批系统调用只实现：

```text
write
exit
yield
```

通过条件：

- ring3 不能写内核页；
- 用户页故障只终止当前进程；
- 系统调用返回后寄存器保持；
- 无效用户指针不会导致内核崩溃。

---

## Phase 9：文件系统和更高级功能

推荐顺序：

1. 只读 initramfs；
2. VFS 接口；
3. IDE 块设备驱动；
4. 简单文件系统；
5. shell；
6. 用户程序加载；
7. fork/exec 或更简单的 spawn 模型；
8. 多核支持。

不要优先实现网络、GUI 或 SMP。这些功能会放大内存管理、同步和调度器中的基础错误。

---

## 6. 当前代码迁移前必须处理的风险

这些问题不一定影响当前演示启动，但会阻碍后续功能。

### 6.1 IDTR limit

当前：

```c
idtr.limit = IDTSIZE * sizeof(struct gatedesc);
```

正确值应减一：

```c
idtr.limit = IDTSIZE * sizeof(struct gatedesc) - 1;
```

### 6.2 中断门权限

当前所有中断门都叠加了 `IDT_DPL3`。普通异常和硬件 IRQ 应使用 DPL0，否则用户态可以主动调用不应暴露的中断门。

### 6.3 EOI 发送时机

当前 ISR 汇编在调用 handler 前，同时向主从 PIC 发送 EOI。应修改为：

- CPU 异常不发送 EOI；
- IRQ0–IRQ7 只向主 PIC 发送；
- IRQ8–IRQ15 先从 PIC、再主 PIC；
- 通常在 handler 完成后发送。

### 6.4 `switch_to` 保存寄存器不一致

结构定义期望保存 `EBX`，汇编实际保存 `EDX`。这是调度器继续扩展前的高优先级问题。

### 6.5 页表注册逻辑

当前 `page_register` 中 PDE/PTE 地址计算和新页表分配逻辑需要重新审核。不存在的 PDE 不能只设置标志位，必须先分配并清零一个物理页作为页表。

### 6.6 分配失败处理

位图分配失败、物理页耗尽、就绪队列为空等路径目前缺少统一错误处理。Rust 版本应使用：

```rust
Result<T, Error>
Option<T>
```

而不是继续使用无效地址。

### 6.7 中断状态类型

当前中断 API 混用 `int` 和 `uint32_t`。Rust 版本应使用明确类型：

```rust
pub struct InterruptState {
    enabled: bool,
}
```

恢复中断时只恢复先前状态，不要无条件 `sti`。

---

## 7. Debug 方法设计

## 7.1 第一层：编译时检查

每次构建执行：

```bash
git diff --check
make clean
make
```

Rust 代码执行：

```bash
cargo fmt --check
cargo clippy --target <target-json>
cargo build --target <target-json>
```

额外检查最终内核中是否出现不允许的指令：

```bash
x86_64-elf-objdump -d bin/kernel |
  rg 'xmm|movdqa|movaps|fxsave|fxrstor'
```

检查 ELF：

```bash
x86_64-elf-readelf -h -l -S bin/kernel
x86_64-elf-nm -n bin/kernel
```

重点确认：

- ELF32；
- i386；
- entry 地址正确；
- LOAD 段地址正确；
- Rust 符号已链接；
- `.bss` 位于预期区域。

---

## 7.2 第二层：串口里程碑

为启动过程保留稳定、短小的标记：

```text
B  MBR
L  loader
P  protected mode
G  paging enabled
E  ELF loaded
K  kernel entry
I  IDT ready
M  memory ready
T  scheduler ready
U  user mode
```

规则：

- 每个标记只代表一个已经完成的阶段；
- 不要在完成前打印成功标记；
- panic 使用不同前缀；
- 串口写入必须等待发送寄存器空闲。

示例：

```text
PANIC vector=0E err=00000002
eip=C0002345 cr2=DEADBEEF
task=keyboard esp=C009EFA0
```

---

## 7.3 第三层：QEMU 异常日志

诊断异常：

```bash
qemu-system-i386 \
  -drive file=disk_img_file/c.img,format=raw,if=ide \
  -display none \
  -serial stdio \
  -monitor none \
  -no-reboot \
  -no-shutdown \
  -d int,guest_errors,cpu_reset \
  -D qemu.log
```

用途：

- `check_exception`：查看异常链；
- `CR2`：页故障地址；
- `Triple fault`：IDT、栈或异常入口再次故障；
- `cpu_reset`：判断是否发生了复位。

如果只看到 triple fault，应向前寻找第一个异常，而不是分析最后一个 double fault。

---

## 7.4 第四层：QEMU 指令跟踪

怀疑代码在内存中被覆盖时：

```bash
-d in_asm
```

这次修复中，正是通过指令跟踪发现：

```text
预期：mov ebx, 0x00070000
实际：mov ebx, 0xFC000000
```

然后将 `0xFC000000` 与写入 `0xB00` 的 E820 数据关联起来，最终定位 loader 自覆盖。

指令跟踪日志很大，只在以下场景启用：

- 跳转到了错误地址；
- 指令字节与二进制不一致；
- 怀疑 self-modifying code；
- 怀疑磁盘读取或内存覆盖。

---

## 7.5 第五层：GDB/LLDB

调试启动：

```bash
qemu-system-i386 \
  -S \
  -s \
  -drive file=disk_img_file/c.img,format=raw,if=ide
```

参数含义：

- `-S`：CPU 启动后暂停；
- `-s`：在 TCP 1234 开启 GDB server。

连接后常用断点：

```text
loader_start
p_mode_start
enter_kernel
kernel_init
main
rust_kernel_probe
rust_interrupt_dispatch
schedule
```

常用检查：

```text
寄存器
CR0/CR2/CR3/CR4
GDT/IDT
页目录物理内存
ELF header
当前栈
中断栈帧
线程 saved_stack
```

Rust 调试建议保留 DWARF：

```toml
[profile.dev]
debug = 2
opt-level = 0
```

如果优化导致单步困难，用 dev profile 复现；不要直接在高优化 release 上猜测源码执行顺序。

---

## 7.6 第六层：自动化 QEMU 测试

建议每个测试构建一个独立内核入口，或者使用 feature：

```bash
cargo build --features test-page-map
```

测试成功：

```rust
qemu::exit_success();
```

测试失败：

```rust
qemu::exit_failure();
```

第一批自动测试：

1. `serial_smoke`；
2. `interrupt_breakpoint`；
3. `page_fault_report`；
4. `frame_alloc_unique`；
5. `map_translate_unmap`；
6. `heap_box`；
7. `context_switch_registers`；
8. `wait_queue_wakeup`；
9. `keyboard_ring_buffer`。

所有测试必须有超时。超时应被视为失败，并保存：

- 串口日志；
- QEMU 异常日志；
- 最终寄存器；
- 屏幕截图。

---

## 7.7 第七层：哨兵值和结构布局

低层系统中最有效的调试方法之一是哨兵值。

线程栈：

```text
0x19870916
```

页帧：

```text
已分配：0xAA
已释放：0xDD
新栈：   0xCC
```

跨 C/Rust 结构应在两边验证：

```text
sizeof
alignof
offsetof
```

对于汇编访问的结构，建议生成或静态断言字段偏移。任何结构字段调整都必须同步汇编。

---

## 8. 每次修改的推荐工作流

使用以下固定循环：

```text
1. 定义唯一目标
2. 写通过条件
3. 添加失败时可见的日志
4. 只修改一个模块
5. 静态检查
6. 完整构建
7. 运行 QEMU smoke test
8. 运行本阶段专项测试
9. 检查异常日志
10. 提交小 commit
```

示例：

```text
目标：Rust 读取 COM1 line status

通过条件：
- C 调用 rust_serial_probe
- 串口打印 RS
- 无异常
- objdump 无 SSE

只修改：
- rust/kernel-rs/src/drivers/serial.rs
- Rust 静态库链接规则
```

不要把以下内容放在同一个提交中：

- 页表重写；
- 调度器重写；
- Rust target 修改；
- 链接脚本修改；
- loader 修改。

这些变化一旦组合，出现故障时很难判断错误属于哪个边界。

---

## 9. 推荐的近期实施顺序

接下来建议按以下顺序推进：

### Milestone 1：可自动判定的内核测试

- 可靠串口；
- QEMU debug-exit；
- 异常寄存器输出；
- `make test`。

### Milestone 2：Rust 静态库接入

- 自定义 target；
- `no_std`；
- panic handler；
- C/Rust 双向调用；
- 无 SSE 验证。

### Milestone 3：Rust HAL

- port I/O；
- serial；
- VGA；
- PIC/PIT。

### Milestone 4：中断重构

- 修正 IDT limit；
- 修正 gate DPL；
- 修正 EOI；
- Rust interrupt dispatch。

### Milestone 5：内存管理重写

- 完整 E820 map；
- frame allocator；
- mapper；
- heap；
- `alloc`。

### Milestone 6：调度器重写

- 修正 `switch_to`；
- Rust task；
- ready/wait queue；
- idle task；
- 同步原语。

### Milestone 7：用户态

- TSS；
- ring3；
- syscall；
- 用户地址空间。

完成 Milestone 1 和 2 后，再决定后续模块的 Rust API。现在过早设计全部 trait 和类型，容易围绕尚未验证的架构产生无效抽象。

---

## 10. 第一项建议任务

下一步最适合执行：

```text
建立 Rust 最小静态库 + QEMU 自动退出测试
```

原因：

- 不改变启动链；
- 不改变分页；
- 不改变调度器；
- 能快速验证 Rust 工具链、ABI 和链接流程；
- 为后续每个 Rust 模块提供自动测试基础。

具体交付物：

```text
rust-toolchain.toml
rust/Cargo.toml
rust/targets/i686-x86-os.json
rust/kernel-rs/Cargo.toml
rust/kernel-rs/src/lib.rs
rust/kernel-rs/src/panic.rs
Makefile 中的 Rust 构建和链接规则
make test-rust
```

成功输出建议为：

```text
LPDEXY
RUST_OK
```

完成后再进入串口、VGA 和中断模块迁移。
