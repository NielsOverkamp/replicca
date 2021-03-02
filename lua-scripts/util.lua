local t = require("move")
local inventory = require("inventory")

local util = {}

util.__index = util

function util:new(moveApi)
    moveApi = moveApi or t
    local new = { moveApi = moveApi }
    setmetatable(new, self)
    return new
end

function util:setMoveApi(moveApi)
    self.moveApi = moveApi
end

function util.defaultMineAction()
    turtle.digUp()
    turtle.digDown()
end

function util:spiral(d, mine, action, pos)
    if action == nil then
        action = function()
        end
    end
    for i = 1, d do
        for _ = 1, 2 do
            for j = 1, i do
                action(false, pos, self.moveApi)
                if mine then
                    self.moveApi.mf(pos)
                else
                    self.moveApi.f(pos)
                end
                if i == d and j == (d - 1) then
                    action(true, pos, self.moveApi)
                    return
                end
            end
            self.moveApi.r(pos)
        end
    end
end

return util
