local inventory = {
    noChange = {__json_type="object"}
}

inventory.__index = inventory

function inventory:new()
    local new = {}
    local slots = {}
    for i=1,16 do
        slots[i] = nil
    end
    new.slots = slots
    setmetatable(new, self)
    return new
end

function inventory:update()
    local delta = {}
    for i=1,16 do
        local c = turtle.getItemCount(i)
        local content
        if c > 0 then
            content = turtle.getItemDetail(i, false)
            local prev = self.slots[i]
            if prev == nil or prev.name ~= content.name then
                self.slots[i] = {
                    name=content.name,
                    count=c
                }
                delta[i] = {
                    n=content.name,
                    c=c
                }
            elseif prev.count == c and prev.name == content.name then
                delta[i] = self.noChange
            else
                delta[i] = c
                self.slots[i].count = c
            end
        else
            content = nil
            if self.slots[i] ~= nil then
                self.slots[i] = nil
                delta[i] = self.noChange
            else
                delta[i] = self.noChange
            end
        end
    end
    return delta
end

return inventory
