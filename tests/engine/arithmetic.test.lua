function add_two_constants()
    assert(1 + 2 == 3)
end

function sub_two_constants()
    assert(1 - 2 == -1)
end

function mul_two_constants()
    assert(2 * 3 == 6)
end

function div_two_constants()
    assert(3 / 2 == 1.5)
end

function inf_equality()
    local inf = 1 / 0
    local neg_inf = -1 / 0

    assert(inf == inf)
    assert(neg_inf == neg_inf)
    assert(inf ~= neg_inf)
end

function nan_equality()
    local nan = 0 / 0
    assert(nan ~= nan)
    assert(nan ~= 0)
    assert(nan ~= 1)
end
