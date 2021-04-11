---
--- Fells a single tree, fuels itself and crafts a chest
--- Assumes inventory is empty
---

return function(pos)
    turtle.dig()
    task:send_event("position_update",pos)
    turtle.select(1)
    turtle.craft()
    turtle.refuel()

    wt.f(pos)
    wt.mu(pos, 2)
    turtle.craft()
    turtle.transferTo(2, 1)
    turtle.transferTo(3, 1)
    turtle.transferTo(5, 1)
    turtle.transferTo(7, 1)
    turtle.transferTo(9, 1)
    turtle.transferTo(10, 1)
    turtle.transferTo(11, 1)
    turtle.select(16)
    turtle.craft()

    local h, state = subtask_execute("fell_inter")
                
    return state == "regrow"
end