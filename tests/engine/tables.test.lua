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
