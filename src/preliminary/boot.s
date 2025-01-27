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
    /* give grub memory map as arg to main */
    mov esi, eax
    mov edi, ebx

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

.SET NB_P2_TABLES, 4
.SET NB_P3_TABLES, 1
.SET NB_P4_TABLES, 1

.SET P2_TABLE_NB_ENTRIES, 512*NB_P2_TABLES /* 16 GiB of RAM cauz huge pages */
.SET P3_TABLE_NB_ENTRIES, NB_P2_TABLES
.SET P4_TABLE_NB_ENTRIES, NB_P3_TABLES

/* TODO : Reduce this part when paging will be dynamic */
set_up_page_tables:
    /* map first P4 entry to P3 table */
    lea eax, [p3_table]
    or eax, 0b11 /* present + writable */
    mov [p4_table], eax

    lea eax, p4_table
    or eax, 0b11 /* present + writable */
    mov [p4_table + 511*8], eax

    // map first P3 entry to P2 table
    lea eax, [p2_table]
    or eax, 0b11 // present + writable 
    mov [p3_table], eax

    // map second P3 entry to P2 table
    lea eax, [p2_table+4096]
    or eax, 0b11
    mov [p3_table+8], eax

    // map third P3 entry to P2 table
    lea eax, [p2_table+4096*2]
    or eax, 0b11
    mov [p3_table+16], eax

    // map fourth P3 entry to P2 table
    lea eax, [p2_table+4096*3]
    or eax, 0b11
    mov [p3_table+24], eax
    
    /*
    // map P3_TABLE_NB_ENTRIES P3 entries to P2 tables
    mov ecx, 0       // counter variable
    .map_p3_table:
        // Compute the address of the ecx-th p2_table
        // p2_table + 0x1000*ecx
        mov eax, ecx
        shl eax, 12  // multiply by 0x1000
        add eax, p2_table

        or eax, 0b11                  // present + writable
        mov [p3_table + ecx * 8], eax // map ecx-th entry

        inc ecx            // increase counter
        cmp ecx, P3_TABLE_NB_ENTRIES        // if counter == P3_TABLE_NB_ENTRIES, the whole P3 table is mapped
        jne .map_p3_table  // else map the next entry
    */

    /* map each P2 entry to a huge 2MiB page */
    mov ecx, 0         /* counter variable */
    .map_p2_table:
        /* map ecx-th P2 entry to a huge page that starts at address 2MiB*ecx */
        /* 0x200000*ecx */
        mov eax, ecx
        shl eax, 21  /* multiply by 0x200000 */

        or eax, 0b10000011 /* present + writable + huge */
        mov [p2_table + ecx * 8], eax /* map ecx-th entry */

        inc ecx            /* increase counter */
        cmp ecx, P2_TABLE_NB_ENTRIES  /* if counter == 1024, the whole P2 tables are mapped */
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
    call error
    cli
    hlt







/* BSS : section where uninitialised data is stored */

.section .bss
/* TODO : Make the stack grow automatically one day */
.SET STACK_SIZE, 0x1000*16
/* TODO : Make paging dynamic, too lazy for that now tho */
.SET P2_TABLE_SIZE, 0x1000*NB_P2_TABLES
.SET P3_TABLE_SIZE, 0x1000*NB_P3_TABLES
.SET P4_TABLE_SIZE, 0x1000*NB_P4_TABLES

/* allocate pages */
.align 0x1000 /* align to 4096 bytes (page size) */
p4_table:
    .space P4_TABLE_SIZE
p3_table:
    .space P3_TABLE_SIZE
p2_table:
    .space P2_TABLE_SIZE

/* allocate stack */
stack_bottom:
    .space STACK_SIZE
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