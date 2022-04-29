function _side_effecty_fn() side_effect_committed = 1 end

function and_short_circuits()
    side_effect_committed = nil

    local res = nil and _side_effecty_fn()

    assert(not side_effect_committed)
end

function or_short_circuits()
    side_effect_committed = nil

    local res = 1 or _side_effecty_fn()

    assert(not side_effect_committed)
end