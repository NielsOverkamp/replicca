return function(pos)
    print("starting fell")
    wt.mf(pos, 1)
    task:send_event("position_update", pos)

    subtask_execute("fell_inter")
    print("done")
    task:send_event("inventory_update", inventory:update())
end
