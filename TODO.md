### What i'm working on currently

* Package the state machine + keycode to an event and send it to an event queue
    * Translate events to actual keys (a, b, c, Shift+a, etc) 
* Shadow buffer so that it runs at normal speed on real hardware
* paging
    * don't forget guard pages for stacks

### Testing

* Do the rest of the integration tests

### The rest

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
