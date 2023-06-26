function print_table(table, offset, visited)
    if visited[table] then
        io.write("<reccursive>\n")
        return
    end
    visited[table] = table

    io.write("{\n")
    local key, value = next(table)
    while key do
        local i = 0
        while i < offset do
            io.write("  ")
            i = i + 1
        end
        io.write(tostring(key))
        io.write(" = ")
        pprint_impl(value, offset + 1, visited)
        key, value = next(table, key)
    end
    local i = 0
    while i < offset - 1 do
        io.write("  ")
        i = i + 1
    end
    io.write("}\n")
end

function pprint_impl(value, offset, visited)
    if type(value) == "table" then
        print_table(value, offset, visited)
    else 
        io.write(tostring(value))
        io.write("\n")
    end
end

function pprint(value) 
    pprint_impl(value, 1, {})
end
