#pragma once
#include "../lib/kernel/klib.h"
#include "thread.h"
#include "pic.h"
#include "idt.h"
#include "keyboard.h"
#define IDT_DESC_CNT 0x30 
void general_program(uint32_t vecnum);
//extern void *isr_entry_table[33];
extern void *intr_entry_table[IDT_DESC_CNT];
typedef void (*isr_func)(void);
extern void* idt_table[IDT_DESC_CNT];
#define CONCAT(X, Y) X##Y
void init_idt_table();
//void init_isr_table();
void clock_interrupt();
void register_intr_handler(uint32_t vecnum, isr_func func);
void keyboard_intr_handler();
#define KEY_PORT 0x60

