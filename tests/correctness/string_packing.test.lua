function pack_string(input)
  local i = 0
  local len = strlen(input)
  local counts = {}
  local order = {}
  local unique_chars = 0

  while i < len do
    local char_str = strsub(input, i, i)
    local count = counts[char_str]
    if count == nil then
      counts[char_str] = 1
      order[unique_chars] = char_str
      unique_chars = unique_chars + 1
    else
      counts[char_str] = count + 1
    end
  end

  i = 0
  local result = ""
  while i < unique_chars do
    local char_str = order[i]
    local count = counts[char_str]
    while count > 0 do
      result = result .. char_str
      count = count - 1
    end
    i = i + 1
  end

  return result
end

assert(pack_string("") == "")
assert(pack_string("A") == "A")
assert(pack_string("ABA") == "AAB")
assert(pack_string("ABCD") == "ABCD")
assert(pack_string("ABCDABACDBE") == "AAABBBCCDDE")
