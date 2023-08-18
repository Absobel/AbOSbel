.SET MAGIC, 0xE85250D6
.SET ARCH_I386, 0
.SET HEADER_LENGTH, header_end - header_start
.SET CHECKSUM, -(MAGIC + ARCH_I386 + HEADER_LENGTH)

.section .multiboot
header_start:
    .long MAGIC
    .long ARCH_I386
    .long HEADER_LENGTH
    .long CHECKSUM

    /* multiboot tags go here */

    /* end tag */
    .short 0
    .short 0    /* flags, none set */
    .long 8     /* size, including itself (short + short + long) */
header_end:
