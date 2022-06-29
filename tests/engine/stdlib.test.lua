-- -- tonumber tests
-- assert(tonumber ~= nil)
-- assert(tonumber() == nil)
-- assert(tonumber(42) == 42)
-- assert(tonumber(nil) == nil)
-- assert(tonumber("42") == 42)
-- assert(tonumber("not a number") == nil)
-- assert(tonumber("42 and some text") == nil)

-- print tests
-- assert(print)
-- I cannot check the behavior of print inside of lua
-- print()
-- print(nil)
-- print(1)
-- print("hello")
-- function foo() end
-- print(foo)
-- print(1, nil, "hello", foo, foo, nil, "bye", 42.32)

function random_produces_values_from_0_to_1()
    assert(random ~= nil)
    i = 1
    while i ~= 1000 do
        res = random()
        assert(res >= 0)
        assert(res <= 1)
        i = i + 1
    end
end
-- I could test "randomness" sorta speak, by calculating 
-- that entropy is sufficient, but yeah... Not today

function floor_floors_numbers()
    assert(floor ~= nil)
    assert(floor(1) == 1)
    assert(floor(42.2) == 42)
    assert(floor(-42.2) == -43)
    assert(floor(69.9) == 69)
end
