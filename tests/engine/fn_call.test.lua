function _callee(arg, arg2)
  assert(type(arg) == "table")
  assert(arg.foo == 42)
  assert(arg2 == nil)
end

function calling_function_with_table_as_argument()
  _callee { foo = 42 }
  _callee({ foo = 42 })
end
