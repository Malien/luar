function trying_to_trick_array_like_tables()
    local tbl = {}
    tbl[2] = 42
    tbl[1] = 0
    tbl[2] = 69
    assert(tbl[2] == 69)
end
