return function(pos)
    print("starting fell")
    wt.mf(pos, 1)
    os.queueEvent("replicca:position_update", pos)
    local h = 0
    while true do
        print("going up?")
        local isBlock, res = turtle.inspectUp()
        if not isBlock or not res.tags["minecraft:logs"] then
            print("top reached")
            break
        end
        print("ha-cha")
        wt.mu(pos, 1)
        h = h + 1
    end
    print("going down")
    os.queueEvent("replicca:position_update", pos)
    wt.md(pos, h)
    print("one back")
    os.queueEvent("replicca:position_update", pos)
    wt.mb(pos, 1)
    print("done")
    os.queueEvent("replicca:position_update", pos)
    return
end
