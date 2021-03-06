return function(pos)
    print("starting fell")
    wt.mf(pos, 1)
    os.queueEvent("replicca:position_update", "UP", pos)

    subtask_execute("fell_inter")
    print("done")
    os.queueEvent("replicca:inventory_update", "UP", inventory:update())
end
