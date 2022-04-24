function while_loop_with_falsy_condition_does_not_execute_body()
    local side_effect_committed
    while nil do
        side_effect_committed = 1
    end
    assert(not side_effect_committed)
end

function while_loop_with_truthy_condition_executes_body_at_least_once()
    local side_effect_committed
    while not side_effect_committed do
        side_effect_committed = 1
    end
    assert(side_effect_committed)
end

function _early_return()
    while 1 do
        return "early"
    end
    return "late"
end

function while_loop_early_return() 
    assert(_early_return() == "early")
end

function while_loop_executes_until_condition_is_true()
    -- should be a hyper parameter
    local times = 10

    local i, count_executed = times, 0
    while i ~= 0 do
        count_executed = count_executed + 1
        i = i - 1
    end

    assert(i == 0)
    assert(count_executed == times)
end
