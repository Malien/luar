function trying_to_trick_array_like_tables()
    local tbl = {}
    tbl[2] = 42
    tbl[1] = 0
    tbl[2] = 69
    assert(tbl[2] == 69)
end

function storing_float_that_is_integer_like_is_equivalent_to_storing_that_int()
    local tbl = {}
    tbl[1.0] = 42
    tbl[1.2] = 69
    assert(tbl[1.0] == 42)
    assert(tbl[1.2] == 69)
    assert(tbl[1] == 42)
    assert(not tbl[2])
    assert(not tbl[0])
end

function constructing_list_table_does_not_error_out()
    local tbl = { 1, 2, 3, 4, "foo", "bar", nil, constructing_list_table_does_not_error_out }
end

function constructing_associative_table_does_not_error_out()
    local tbl = { foo = "bar", bar = 42, baz = constructing_associative_table_does_not_error_out }
end

function property_associations_are_preserved_in_the_table()
    local tbl = { foo = "bar", bar = 42, baz = constructing_associative_table_does_not_error_out }

    assert(tbl.foo == "bar")
    assert(tbl.bar == 42)
    assert(tbl.baz == constructing_associative_table_does_not_error_out)
    assert(not tbl.nope)
end

function member_associations_are_preserved_in_the_table()
    local tbl = { 42, 69; foo = "bar" }
    local assoc_name = 2

    assert(tbl[1] == 42)
    assert(tbl[2] == 69)
    assert(not tbl[3])
    assert(tbl["foo"] == "bar")
    assert(tbl[assoc_name] == 69)
end

function nil_lookup_is_not_an_error()
    local tbl = { }
    local nan = -"nan"
    assert(not tbl[nil])
    assert(not tbl[nan])
end

function assigning_to_a_property_modifies_value_of_a_table()
    local tbl = { }
    assert(not tbl.foo)
    tbl.foo = 42
    assert(tbl.foo == 42)
    tbl.bar = 69
    assert(tbl.bar == 69)
    tbl.foo = "hello"
    assert(tbl.foo == "hello")
end

function assigning_to_a_member_modifies_value_of_a_table()
    local tbl = { }

    assert(not tbl["foo"])
    tbl["foo"] = 42
    assert(tbl["foo"] == 42)
    tbl["bar"] = 69
    assert(tbl["bar"] == 69)
    tbl["foo"] = "hello"
    assert(tbl["foo"] == "hello")

    assert(not tbl[1])
    tbl[2 - 1] = 42
    assert(tbl[1] == 42)

    tbl[assigning_to_a_member_modifies_value_of_a_table] = 420
    assert(tbl[assigning_to_a_member_modifies_value_of_a_table] == 420)
end
