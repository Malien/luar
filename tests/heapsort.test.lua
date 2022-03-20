-- local random, floor = math.random, math.floor
-- floor = math.ifloor or floor

function heapsort(n, ra)
    local j, i, rra
    local l = floor(n/2) + 1
    -- local l = (n//2) + 1
    local ir = n;
    while 1 do
        if l > 1 then
            l = l - 1
            rra = ra[l]
        else
            rra = ra[ir]
            ra[ir] = ra[1]
            ir = ir - 1
            if (ir == 1) then
                ra[1] = rra
                return
            end
        end
        i = l
        j = l * 2
        while j <= ir do
            if (j < ir) and (ra[j] < ra[j+1]) then
                j = j + 1
            end
            if rra < ra[j] then
                ra[i] = ra[j]
                i = j
                j = j + i
            else
                j = ir + 1
            end
        end
        ra[i] = rra
    end
end

function populated_table(N)
    local a = {}
    local i = 1
    while i ~= N + 1 do
        a[i] = random()
        i = i + 1
    end
    return a
end

function assert_sorted(N, tbl)
    local i = 1
    while i ~= N - 1 do
        assert(tbl[i] <= tbl[i+1])
        i = i + 1
    end
end

local iterations = 4
local element_count = 10000

local i = 0
while i ~= iterations do
    local tbl = populated_table(element_count)
    heapsort(element_count, tbl)
    assert_sorted(element_count, tbl)
    i = i + 1
end

