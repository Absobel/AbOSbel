/* The bootloader will look at this image and start execution at the symbol
   designated as the entry point. */
ENTRY(_start)
 
/* Tell where the various sections of the object files will be put in the final
   kernel image. */
SECTIONS
{
	/* Begin putting sections at 1 MiB, a conventional place for kernels to be
	   loaded at by the bootloader. */
	. = 1M;
 
	/* First put the multiboot header, as it is required to be put very early
	   early in the image or the bootloader won't recognize the file format.
	   Next we'll put the .text section. */
	.text ALIGN(4K) :
	{
		*(.multiboot)
		*(.text)
        *(.text.*)
	}
 
	/* Read-only data. */
	.rodata ALIGN(4K) :
	{
        *(.rodata)
		*(.rodata.*)
	}
 
	/* Read-write data (initialized) */
	.data ALIGN(4K) :
	{
		*(.data)
        *(.data.*)
	}
	
	/* Read-write data (uninitialized) and stack */
	.bss ALIGN(4K) :
	{
		*(COMMON)
		*(.bss)
        *(.bss.*)
	}

	/* Debugging information. */
	.debug_abbrev ALIGN(4K) : { *(.debug_abbrev) }
    .debug_info ALIGN(4K) : { *(.debug_info) }
    .debug_aranges ALIGN(4K) : { *(.debug_aranges) }
    .debug_str ALIGN(4K) : { *(.debug_str) }
    .debug_pubnames ALIGN(4K) : { *(.debug_pubnames) }
    .debug_pubtypes ALIGN(4K) : { *(.debug_pubtypes) }
    .debug_frame ALIGN(4K) : { *(.debug_frame) }
    .debug_line ALIGN(4K) : { *(.debug_line) }
    .debug_ranges ALIGN(4K) : { *(.debug_ranges) }
    .debug_loc ALIGN(4K) : { *(.debug_loc) }
 
	/* The compiler may produce other sections, by default it will put them in
	   a segment with the same name. Simply add stuff here as needed. */
}
