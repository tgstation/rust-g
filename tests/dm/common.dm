#include "../../target/rust_g.dm"
#undef RUST_G
/var/RUST_G = world.GetConfig("env", "RUST_G")

/world/New()
    for(var/func in typesof(/test/proc))
        log << "[func] [copytext("------------------------------------------------------------------------", length("[func]"))]"
        call(new /test, func)()
    del src
