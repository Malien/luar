function ordering_of_ints_is_preserved()
    local value = -1000
    while value < 1000 do
        assert(value < value + 1)
        assert(value == value)
        assert(value + 1 > value)
        assert(value - 1 < value)
        assert(value + 1 >= value)
        assert(value >= value)
        assert(value <= value)
        assert(value - 1 <= value)
        assert(value <= value + 1)
        value = value + 1
    end
end

function ordering_of_floats_is_preserved()
    local value = -100.0
    local inf = - -"inf"
    local ninf = -"inf"
    while value < 100.0 do
        assert(value < value + 1)
        assert(value == value)
        assert(value + 1 > value)
        assert(value - 1 < value)
        assert(value + 1 >= value)
        assert(value >= value)
        assert(value <= value)
        assert(value - 1 <= value)
        assert(value <= value + 1)
        assert(value > ninf)
        assert(value < inf)
        assert(value >= ninf)
        assert(value <= inf)
        value = value + 0.1
    end
end

function oredering_of_strings_is_preserved()
    assert("aaa" < "aab")
    assert("aaa" <= "aab")
    assert("bba" > "bb0")
    assert("bba" >= "bb0")
    assert("yaroslav" > "petryk")
    assert("A" < "a")
end

function numbers_are_coerced_to_strings_when_comparing()

end