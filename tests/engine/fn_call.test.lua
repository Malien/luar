function _callee(arg, arg2)
  assert(type(arg) == "table", "argument passed in is not a table")
  assert(arg.foo == 42, "argument passed in is not a table with foo = 42")
  assert(arg2 == nil, "there shouldn't be a second argument")
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
  assert(_return_multi ~= nil, "function _return_multi is not defined")
  local a, b = _return_multi(nil)
  assert(a == 1, "first return of single-return list is not set correctly")
  assert(b == nil, "second return of single-return shouldn't be present")

  local c, d = _return_multi(1)
  assert(c == 1, "first return of multi-return list is not set correctly")
  assert(d == 2, "second return of multi-return list is not set correctly")
end
