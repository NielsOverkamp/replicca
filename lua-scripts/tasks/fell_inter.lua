---
--- Task intended for internal use only
---

return function(pos)
    local h = 0
    while true do
        print("going up?")
        local isBlock, res = turtle.inspectUp()
        if not isBlock or not res.tags["minecraft:logs"] then
            print("top reached")
            break
        end
        print("ha-cha")
        wt.mu(pos)
        h = h + 1
    end

    wt.mu(pos)
    h = h + 1
    os.queueEvent("replicca:position_update", "UP", pos)

    print("chopping off top")
    for _ = 1, 3 do
        turtle.dig()
        wt.l(pos)
    end
    turtle.dig()

    print("2 down")
    wt.md(pos, 2)
    os.queueEvent("replicca:position_update", "UP", pos)

    local function cutAction(isFinal)
        util.defaultMineAction(isFinal)
        os.queueEvent("replicca:position_update", "UP", pos)
    end

    print("spiralling")
    util:spiral(5, true, cutAction, pos)

    os.queueEvent("replicca:inventory_update", "UP", inventory:update())
    local replant, wait, saplingSlot = table.unpack(task.task_question("replant"))
    if replant then
        wt.r(pos)
        wt.mf(pos, 2)
        wt.r(pos)
        wt.mf(pos)
        wt.md(pos, h)
        turtle.select(saplingSlot)
        turtle.place()
        if wait then
            while true do
                local block = task.execute_function(turtle.inspect)
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