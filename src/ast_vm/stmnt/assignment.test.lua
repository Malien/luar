-- simple assignment
foo = 42
bar = 69
assert(foo == 42)
assert(bar == 69)

-- swaps
foo, bar = bar, foo
assert(foo == 69)
assert(bar == 42)

-- table swaps
tbl = { 42, 69 }
assert(tbl[1] == 42)
assert(tbl[2] == 69)

tbl[1], tbl[2] = tbl[2], tbl[1]
assert(tbl[1] == 69)
assert(tbl[2] == 42)

-- deep assignment
tbl = { foo = { bar = { } } }
assert(tbl.foo.bar.baz == nil)

tbl.foo.bar.baz = 42
assert(tbl.foo.bar.baz == 42)
