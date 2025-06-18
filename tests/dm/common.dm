#define RUST_G world.GetConfig("env", "RUST_G")
#include "rust_g.dm"

/world/New()
    for(var/func in typesof(/test/proc))
        log << "[func] [copytext("------------------------------------------------------------------------", length("[func]"))]"
        call(new /test, func)()
    del src
