---
--- Task intended for internal use only
---

return function(pos)
    local h = 0
    while true do
        local isBlock, res = turtle.inspectUp()
        if not isBlock or not res.tags["minecraft:logs"] then
            break
        end
        wt.mu(pos)
        h = h + 1
    end

    wt.mu(pos)
    h = h + 1
    task:send_event("position_update", pos)

    for _ = 1, 3 do
        turtle.dig()
        wt.l(pos)
    end
    turtle.dig()

    wt.md(pos, 2)
    h = h - 2
    task:send_event("position_update", pos)

    local function cutAction(isFinal)
        util.defaultMineAction(isFinal)
        task:send_event("position_update", pos)
    end

    util:spiral(5, true, cutAction, pos)

    task:send_event("inventory_update", inventory:update())
    local replant, wait, saplingSlot = table.unpack(task:task_question("replant"))
    if replant then
        wt.r(pos)
        wt.mf(pos, 2)
        wt.r(pos)
        wt.mf(pos, 2)
        while not turtle.detectDown() do
            wt.d(pos)
        end
        wt.mb(pos)
        turtle.select(saplingSlot)
        turtle.place()
        turtle.select(1)
        if wait then
            while true do
                local block = task:execute_function(turtle.inspect)
                if block.tags["minecraft:logs"] then
                    break
                end
                sleep(1)
            end
        end
        return 0, "regrow"
    else
        return h
    end
end