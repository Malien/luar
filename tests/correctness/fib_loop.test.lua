function fib(n)
    local n1, n2 = 1, 0
    while n ~= 0 do
        n1, n2 = n1 + n2, n1
        n = n - 1
    end
    return n2
end

assert(fib(0) == 0)
assert(fib(1) == 1)
assert(fib(20) == 6765)