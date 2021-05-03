#include "common.dm"

/test/proc/rev_parse_head()
    ASSERT(rustg_git_revparse("HEAD"))
