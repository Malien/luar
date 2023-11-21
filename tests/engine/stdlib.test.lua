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

function strlen_returns_length_of_string()
    assert(strlen ~= nil)
    assert(strlen("") == 0)
    assert(strlen("hello") == 5)
    assert(strlen("hello world") == 11)
end

function strlen_stringifies_numbers()
    assert(strlen ~= nil)
    assert(strlen(42) == 2)
    assert(strlen(42.2) == 4)
    assert(strlen(-42.2) == 5)
    assert(strlen(69.9) == 4)
end

function type_returns_correct_types()
    assert(type ~= nil, "type function exists")
    assert(type(42) == "number", "42 is a number")
    assert(type(42.2) == "number", "42.2 is a number")
    assert(type(-42.2) == "number", "-42.2 is a number")
    assert(type(69.9) == "number", "69.9 is a number")
    assert(type("hello") == "string", "'hello' is a string")
    assert(type("") == "string", "'' is a string")
    assert(type(nil) == "nil", "nil is nil")
    assert(type({}) == "table", "{} is a table")
    assert(type({1, 2, 3}) == "table", "{1, 2, 3} is a table")
    assert(type({foo = 42}) == "table", "{foo = 42} is a table")
    assert(type(type) == "function", "type is a function")
    assert(type(type_returns_correct_types) == "function", "type_returns_correct_types is a function")
end
