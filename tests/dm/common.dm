#define RUST_G world.GetConfig("env", "RUST_G")
#include "../../target/rust_g.dm"

/// Asserts that the two parameters passed are equal, fails otherwise
/// Optionally allows an additional message in the case of a failure
#define TEST_ASSERT_EQUAL(a, b, message) do { \
	var/lhs = ##a; \
	var/rhs = ##b; \
	if (lhs != rhs) { \
		stack_trace("Expected [isnull(lhs) ? "null" : lhs] to be equal to [isnull(rhs) ? "null" : rhs].[message ? " [message]" : ""]"); \
	} \
} while (FALSE)

/world/New()
    for(var/func in typesof(/test/proc))
        log << "[func] [copytext("------------------------------------------------------------------------", length("[func]"))]"
        call(new /test, func)()
    del src

/proc/stack_trace(message)
	CRASH(message)
