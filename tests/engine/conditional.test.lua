-- Assumptions of working language constructs:
--   - function declaration
--   - local declaration
--   - assert
--   - equality operator
--   - negation operator
--   - assignment
--   - number literals
--   - string literals

-- Assumptions guranteed to work:
--   - conditionals

function if_with_falsy_condition_does_not_evaluate_body()
    local result

    if nil then result = 1 end

    assert(not result)
end

function if_with_truthy_condition_evaluates_body()
    local result

    if 1 then result = 1 end

    assert(result)
end

function if_with_truthy_condition_does_not_evaluate_else_branch()
    local result

    if 1 then
        result = "true branch"
    else
        result = "false branch"
    end

    assert(result == "true branch")
end

function if_with_falsy_condition_evaluates_else_branch()
    local result

    if nil then
        result = "true branch"
    else
        result = "false branch"
    end

    assert(result == "false branch")
end

function _side_effect() side_effect_committed = 1 end

function if_with_truthy_condition_does_not_evaluate_elseif_branch()
    local result

    if 1 then
        result = 'if branch'
    elseif side_effect() then
        result = 'elseif branch'
    else
        result = 'else branch'
    end

    assert(result == "if branch")
    assert(not side_effect_committed)
end

function if_with_falsy_condition_and_passing_elseif_should_evaluate_elseif_branch()
    local result

    if nil then
        result = 'if branch'
    elseif 1 then
        result = 'elseif branch'
    else
        result = 'else branch'
    end

    assert(result == "elseif branch")
end

function if_with_falsy_condition_and_falsy_elseif_condition_should_not_evaluate_anything()
    local result

    if nil then
        result = 'if branch'
    elseif nil then
        result = 'elseif branch'
    end

    assert(result == nil)
end

function if_with_falsy_condition_and_falsy_elseif_condition_should_evaluate_else_branch()
    local result

    if nil then
        result = 'if branch'
    elseif nil then
        result = 'elseif branch'
    else
        result = 'else branch'
    end

    assert(result == "else branch")
end
