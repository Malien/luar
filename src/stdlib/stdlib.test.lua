-- tonumber tests
assert(tonumber ~= nil)
assert(tonumber() == nil)
assert(tonumber(42) == 42)
assert(tonumber(nil) == nil)
assert(tonumber("42") == 42)
assert(tonumber("not a number") == nil)
assert(tonumber("42 and some text") == nil)
-- print tests
assert(print)
-- I cannot check the behavior of print inside of lua
print()
print(nil)
print(1)
print("hello")
function foo() end
print(foo)
print(1, nil, "hello", foo, foo, nil, "bye", 42.32)