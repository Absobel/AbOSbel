.SET MAGIC, 0xE85250D6
.SET ARCH_I386, 0
.SET HEADER_LENGTH, header_end - header_start
.SET CHECKSUM, -(MAGIC + ARCH_I386 + HEADER_LENGTH)

/* TAGS */
.SET TAG_FLAG_OPTIONAL, 1
.SET TAG_FLAG_REQUIRED, 0

/* HEADER TAGS TYPES */
.SET HEADER_ENDING_TAG_TYPE, 0
.SET HEADER_INFORMATION_REQUEST_TAG_TYPE, 1
    .SET HEADER_INFORMATION_REQUEST_TAG_SIZE, info_request_tag_end - info_request_tag_start
.SET HEADER_FRAMEBUFFER_TAG_TYPE, 5

/* MBI TAGS TYPES */
.SET MBI_EFI_SYSTEM_TABLE_TAG_TYPE, 12
.SET MBI_EFI_MEMORY_MAP_TAG_TYPE, 17


.section .multiboot
header_start:
    .long MAGIC
    .long ARCH_I386
    .long HEADER_LENGTH
    .long CHECKSUM

    /* multiboot tags go here */
    /* Information request tag */
    info_request_tag_start:
        .short HEADER_INFORMATION_REQUEST_TAG_TYPE
        .short TAG_FLAG_REQUIRED
        .long HEADER_INFORMATION_REQUEST_TAG_SIZE
        /* mbi_tag_types */
        .long MBI_EFI_SYSTEM_TABLE_TAG_TYPE
        .long MBI_EFI_MEMORY_MAP_TAG_TYPE
    info_request_tag_end:

    .short HEADER_FRAMEBUFFER_TAG_TYPE
    .short TAG_FLAG_REQUIRED
    .long 20
    .long 0 /* let the bootloader fill this in */
    .long 0
    .long 32

    .align 8

    /* end tag */
    .short HEADER_ENDING_TAG_TYPE
    .short 0
    .long 8     /* size, including itself (short + short + long) */
header_end:
