### What i'm working on currently

* Fix the bugs Claude mentioned in the Test Artifact Document
* Fix the deadlock between interrupt handlers and the framebuffer actions (println can't write on lock held by the framebuffer)

### Testing

* Do the rest of the integration tests

### The rest

* Support BIOS for sure
* After allocator, rewrite the graphical part and a lot of other things
    * rewrite buffer into a vec/smth
    * Adapt for different framebuffers colors and offset
* Logging
* APIC
* Write the multiboot part in rust, even the bootstrap if i'm feeling like it
* Make podman connect to VNC for display (use a vnc argument to give it the `-display vnc=0.0.0.0:0`)

### Future goals

* github actions ??
* Global peace in the whole world
* documentation
