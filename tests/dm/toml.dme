#include "common.dm"

var/test_toml = @{"
[database]
enabled = true
ports = [ 8000, 25565 ]
data = [ ["delta", "phi"], [3.14] ]
temp_targets = { cpu = 79.5, case = 72.0 }
"}

var/test_json = @{"
{"database":{"data":[["delta","phi"],[3.14]],"enabled":true,"ports":[8000,25565],"temp_targets":{"case":72.0,"cpu":79.5}}}
"}

/test/proc/check_toml_file2json()
    rustg_file_write(test_toml, "test.toml")
    var/toml_output = rustg_read_toml_file("test.toml")

	// ~= checks for structural equality
    if (json_decode(test_json) ~= toml_output)
        CRASH("test:\n[test_toml]\n \nexpected:\n[test_json]\n \nrustg:\n[json_encode(toml_output)]")
