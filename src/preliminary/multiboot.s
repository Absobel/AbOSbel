.SET MAGIC, 0xE85250D6
.SET ARCH_I386, 0
.SET HEADER_LENGTH, header_end - header_start
.SET CHECKSUM, -(MAGIC + ARCH_I386 + HEADER_LENGTH)

/* TAGS */
.SET TAG_FLAG_OPTIONAL, 0
.SET TAG_FLAG_REQUIRED, 1

/* HEADER TAGS TYPES */
.SET ENDING_TAG_TYPE, 0
.SET INFORMATION_REQUEST_TAG_TYPE, 1
    .SET INFORMATION_REQUEST_TAG_SIZE, info_request_tag_end - info_request_tag_start
    
/* MBI TAGS TYPES */
.SET VBE_INFO_TAG_TYPE, 7
.SET FRAMEBUFFER_INFO_TAG_TYPE, 8



.section .multiboot
header_start:
    .long MAGIC
    .long ARCH_I386
    .long HEADER_LENGTH
    .long CHECKSUM


    /* multiboot tags go here */
    /* Information request tag */
    info_request_tag_start:
        .short INFORMATION_REQUEST_TAG_TYPE
        .short TAG_FLAG_REQUIRED
        .long INFORMATION_REQUEST_TAG_SIZE
        /* mbi_tag_types */
        .long VBE_INFO_TAG_TYPE
        .long FRAMEBUFFER_INFO_TAG_TYPE
    info_request_tag_end:

    /* end tag */
    .short ENDING_TAG_TYPE
    .short TAG_FLAG_OPTIONAL
    .long 8     /* size, including itself (short + short + long) */
header_end:
