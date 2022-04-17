function simple_assignment() 
    foo = 42
    bar = 69
    assert(foo == 42)
    assert(bar == 69)
end

function swaps()
    foo, bar = 42, 69
    foo, bar = bar, foo
    assert(foo == 69)
    assert(bar == 42)
end

function table_swaps()
    tbl = { 42, 69 }
    assert(tbl[1] == 42)
    assert(tbl[2] == 69)

    tbl[1], tbl[2] = tbl[2], tbl[1]
    assert(tbl[1] == 69)
    assert(tbl[2] == 42)
end


function deep_assignment()
    table = { foo = { bar = { } } }
    assert(table.foo.bar.baz == nil)

    table.foo.bar.baz = 42
    assert(table.foo.bar.baz == 42)
end
