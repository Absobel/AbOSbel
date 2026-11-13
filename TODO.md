### What i'm working on currently

* paging
    * create a heap allocator
    * don't forget guard pages for stacks
* Shadow buffer so that it runs at normal speed on real hardware
    * Just make the reads read from the backbuffer and the writes write to both
* rewrite whatever is after the new allocator with the heap

### Testing

* Do the rest of the integration tests

### The rest (in no particular order)

* Support AZERTY
* Add key combo maybe (or maybe direct usb for moderner hardware)
* Make it so that WRITER can't be unlocked while BUFFER is locked
* Support BIOS for sure
* After allocator, rewrite the graphical part and a lot of other things
    * rewrite buffer into a vec/smth
    * Adapt for different framebuffers colors and offset
* Logging
* APIC (better discovery for keyboard and mouse)
* Write the multiboot part in rust, even the bootstrap if i'm feeling like it
* Make podman connect to VNC for display (use a vnc argument to give it the `-display vnc=0.0.0.0:0`)

### Future goals

* github actions ??
* Global peace in the whole world
* documentation
