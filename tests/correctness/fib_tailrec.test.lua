function fib_rec(left, n1, n2)
    if left == 0 then
        return n2
    end
    return fib_rec(left - 1, n1 + n2, n1)
end

function fib(n)
    return fib_rec(n, 1, 0)
end

assert(fib(0) == 0)
assert(fib(1) == 1)
assert(fib(20) == 6765)