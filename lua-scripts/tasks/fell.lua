return function(pos)
    print("starting fell")
    wt.mf(pos, 1)
    os.queueEvent("replicca:position_update", pos)

    subtask_execute("fell_inter")
    print("done")
    os.queueEvent("replicca:inventory_update", inventory:update())
end
