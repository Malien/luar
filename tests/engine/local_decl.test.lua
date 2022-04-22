local foo_one = 42

function _bar()
    return foo_one
end

function local_var_in_global_context_is_not_accessible_from_other_function_contexts()
    assert(not _bar())
end



foo_too = 69
local foo_too = 42

function _bar_too()
    return foo_too
end

foo_three = foo_too

function global_var_cannot_be_redeclared_local() 
    assert(foo_three == 69)
    assert(_bar_too() == 69)
end



function _foo_four()
    return a
end

function _bar_four()
    local a = 42
    return _foo_four()
end

function local_vars_do_not_leak_through_function_calls()
    assert(not _bar_four())
end



function _foo_five()
    if 1 then
        local foo = 42
    end

    if 1 then
        return foo
    end
    return 69
end

function local_scopes_are_different()
    assert(not _foo_five())
end