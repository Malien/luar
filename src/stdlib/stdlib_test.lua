-- tonumber tests
assert(tonumber ~= nil)
assert(tonumber() == nil)
assert(tonumber(42) == 42)
assert(tonumber(nil) == nil)
assert(tonumber("42") == 42)
assert(tonumber("not a number") == nil)
assert(tonumber("42 and some text") == nil)