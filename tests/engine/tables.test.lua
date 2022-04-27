-- function trying_to_trick_array_like_tables()
--     local tbl = {}
--     tbl[2] = 42
--     tbl[1] = 0
--     tbl[2] = 69
--     assert(tbl[2] == 69)
-- end

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
