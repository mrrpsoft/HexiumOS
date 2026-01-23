.set ALIGN,    1<<0
.set MEMINFO,  1<<1
.set FLAGS,    ALIGN | MEMINFO
.set MAGIC,    0x1BADB002
.set CHECKSUM, -(MAGIC + FLAGS)

.section .multiboot
.align 4
.long MAGIC
.long FLAGS
.long CHECKSUM

.section .bss
.align 16
stack_bottom:
.skip 16384
stack_top:

.section .text
.global _start
.type _start, @function
_start:
	/* Set up stack */
	mov $stack_top, %esp

	/* Reset EFLAGS */
	pushl $0
	popf

	/* Disable FPU (prevents SSE faults if compiler generates FPU code) */
	mov %cr0, %eax
	and $0xFFFFFFFB, %eax  /* Clear EM (bit 2) - no FPU emulation */
	or $0x2, %eax          /* Set MP (bit 1) */
	mov %eax, %cr0

	call kernel_main

	cli
1:	hlt
	jmp 1b
.size _start, . - _start

.section .note.GNU-stack, "", @progbits
