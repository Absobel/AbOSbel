.section .text
.global _start

_start:
    call main

    /* Shouldn't ever reach here */
    cli
    hlt
