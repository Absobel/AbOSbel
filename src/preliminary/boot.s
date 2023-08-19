/**
 * This asm program setups the stack and paging to jump into long mode
 * and then calls the main function of the kernel.
*/

.section .text
.global _start
.extern long_mode_start

.code32

/**
 * This is the entry point of the kernel.
*/
_start:
    lea esp, [stack_top]

    call check_multiboot
    call check_cpuid
    call check_long_mode

    call set_up_page_tables
    call enable_paging 

    /* load the 64-bit GDT */
    lgdt [gdt64_pointer]

    ljmp 0x8, offset long_mode_start /* TODO should not be hardcoded but should be set to GDT64_CODE but doesn't work so... */
    
    /* Shouldn't ever reach here */
    cli
    hlt

/* CHECKS */

/**
 * Checks if the multiboot magic number is present in the eax register.
 * If it is, it returns. If it isn't, calls the error function.
*/
check_multiboot:
    cmp eax, 0x36d76289
    jne .no_multiboot
    ret
.no_multiboot:
    mov al, '0'
    jmp error


/**
 * Checks if the CPUID instruction is supported by the CPU.
 * If it is, it returns. If it isn't, calls the error function.
*/
check_cpuid:
    /* Check if CPUID is supported by attempting to flip the ID bit (bit 21)
       in the FLAGS register. If we can flip it, CPUID is available. */

    /* Copy FLAGS in to EAX via stack */
    pushfd
    pop eax

    /* Copy to ECX as well for comparing later on */
    mov ecx, eax

    /* Flip the ID bit */
    xor eax, 1 << 21

    /* Copy EAX to FLAGS via the stack */
    push eax
    popfd

    /* Copy FLAGS back to EAX (with the flipped bit if CPUID is supported) */
    pushfd
    pop eax

    /* Restore FLAGS from the old version stored in ECX (i.e. flipping the
       ID bit back if it was ever flipped). */
    push ecx
    popfd

    /* Compare EAX and ECX. If they are equal then that means the bit
       wasn't flipped, and CPUID isn't supported. */
    cmp eax, ecx
    je .no_cpuid
    ret
.no_cpuid:
    mov al, '1'
    jmp error

/**
 * Checks if the CPU supports long mode.
 * If it does, it returns. If it doesn't, calls the error function.
*/
check_long_mode:
    /* test if extended processor info in available */
    mov eax, 0x80000000    /* implicit argument for cpuid */
    cpuid                  /* get highest supported argument */
    cmp eax, 0x80000001    /* it needs to be at least 0x80000001 */
    jb .no_long_mode       /* if it's less, the CPU is too old for long mode */

    /* use extended info to test if long mode is available */
    mov eax, 0x80000001    /* argument for extended processor info */
    cpuid                  /* returns various feature bits in ecx and edx */
    test edx, 1 << 29      /* test if the LM-bit is set in the D-register */
    jz .no_long_mode       /* If it's not set, there is no long mode */
    ret
.no_long_mode:
    mov al, '2'
    jmp error

/* FUNCTIONS */

/**
 * Error function : writes an error message to the screen and halts the CPU.
*/
error:
    mov dword ptr [0xb8000], 0x4f524f45
    mov dword ptr [0xb8004], 0x4f3a4f52
    mov dword ptr [0xb8008], 0x4f204f20
    mov byte  ptr [0xb800a], al
    hlt

/**
 * Initialize the page tables with 512 2MiB pages in P2 table.
*/
set_up_page_tables:
    /* map first P4 entry to P3 table */
    lea eax, [p3_table]
    or eax, 0b11 /* present + writable */
    mov [p4_table], eax

    /* map first P3 entry to P2 table */
    lea eax, [p2_table]
    or eax, 0b11 /* present + writable */
    mov [p3_table], eax

    /* map each P2 entry to a huge 2MiB page */
    mov ecx, 0         /* counter variable */

.map_p2_table:
    /* map ecx-th P2 entry to a huge page that starts at address 2MiB*ecx */
    mov eax, 0x200000  /* 2MiB */
    mul ecx            /* start address of ecx-th page */
    or eax, 0b10000011 /* present + writable + huge */
    mov [p2_table + ecx * 8], eax /* map ecx-th entry */

    inc ecx            /* increase counter */
    cmp ecx, 512       /* if counter == 512, the whole P2 table is mapped */
    jne .map_p2_table  /* else map the next entry */

    ret

/**
 * Enable paging and long mode. (These comments are mostly to separate functions lol)
*/
enable_paging:
    /* load P4 to cr3 register (cpu uses this to access the P4 table) */
    lea eax, [p4_table]
    mov cr3, eax

    /* enable PAE-flag in cr4 (Physical Address Extension) */
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    /* set the long mode bit in the EFER MSR (model specific register) */
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    /* enable paging in the cr0 register */
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ret





/* LONG MODE */

.code64
.extern main

/**
 * This is the entry point of the kernel in long mode.
*/
long_mode_start:
    /* reinit all the segment registers */
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    call main

    /* Shouldn't ever reach here */
    cli
    hlt







/* BSS : section where uninitialised data is stored */

.section .bss

/* allocate pages */
.align 4096 /* align to 4096 bytes (page size) */
p4_table:
    .space 4096
p3_table:
    .space 4096
p2_table:
    .space 4096

/* allocate stack */
stack_bottom:
    .space 4096
stack_top:







/* RODATA : section where read-only data is stored */
.section .rodata

/* GDT : Global Descriptor Table */
gdt64:
    .quad 0 /* zero entry */
    .set GDT64_CODE, . - gdt64
    .quad (1<<43) | (1<<44) | (1<<47) | (1<<53) /* code segment */
gdt64_pointer:
    .word . - gdt64 - 1
    .quad gdt64