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

    print("chopping off top")
    for _ = 1, 3 do
        turtle.dig()
        wt.l(pos)
    end
    turtle.dig()

    print("2 down")
    wt.md(pos, 2)
    os.queueEvent("replicca:position_update", pos)

    local function cutAction(isFinal)
        util.defaultMineAction(isFinal)
        os.queueEvent("replicca:position_update", pos)
    end

    print("spiralling")
    util:spiral(5, true, cutAction, pos)
end