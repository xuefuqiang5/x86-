# 定义目标目录
TARGET_DIR = ../bin

.PHONY: build clean

# 编译 MBR
$(TARGET_DIR)/mbr.bin: mbr.asm boot.inc | $(TARGET_DIR)
	nasm -f bin -o $@ $<

# 编译 Loader
$(TARGET_DIR)/loader.bin: loader.asm boot.inc | $(TARGET_DIR)
	nasm -f bin -o $@ $<

# 确保目标目录存在
$(TARGET_DIR):
	mkdir -p $(TARGET_DIR)

# 默认目标
build: $(TARGET_DIR)/mbr.bin $(TARGET_DIR)/loader.bin


# 清理生成的文件
clean:
	rm -f $(TARGET_DIR)/mbr.bin $(TARGET_DIR)/loader.bin
