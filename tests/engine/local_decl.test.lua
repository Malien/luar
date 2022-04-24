local foo_one = 42

function _bar()
    return foo_one
end

function local_var_in_global_context_is_not_accessible_from_other_function_contexts()
    assert(not _bar())
end



foo_too = 69

function _bar_too()
    return foo_too
end

function local_shadows_global_in_that_scope() 
    local foo_too = 42
    assert(foo_too == 42)
    assert(_bar_too() == 69)
end



function _foo_four()
    return a
end

function local_vars_do_not_leak_through_function_calls()
    local a = 42
    assert(not _foo_four())
end




function local_scopes_are_different()
    if 1 then
        local foo = 42
    end

    if 1 then
        assert(not foo)
    end
end




function redeclaring_local_creates_new_local()
    local foo = 42
    local foo
    assert(not foo)
end




function redeclaring_local_with_new_value_creates_new_local_with_that_value()
    local foo = 42
    local foo = 69
    assert(foo == 69)
end