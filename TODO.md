### What i'm working on currently

* rewrite the flush print ring to be named flush irq buffer prob, and make it not just use print!() cause that has some side effects
* rewrite whatever is after the new allocator with the heap

### Testing

* Do the rest of the integration tests

### The rest (in no particular order)

* Make a better allocator (for now it's a bump one which is mid)
* Refactor the paging with a Mapper trait and Page struct that would align the addresses
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
