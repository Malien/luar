function _callee(arg, arg2)
  assert(type(arg) == "table")
  assert(arg.foo == 42)
  assert(arg2 == nil)
end

function calling_function_with_table_as_argument()
  _callee { foo = 42 }
  _callee({ foo = 42 })
end

function _return_multi(is_multi)
  if not is_multi then
    return 1
  else
    return 1, 2
  end
end

function multiple_return_values_are_correctly_set()
  local a, b = _return_multi(nil)
  assert(a == 1)
  assert(b == nil)

  local c, d = _return_multi(1)
  assert(c == 1)
  assert(d == 2)
end

