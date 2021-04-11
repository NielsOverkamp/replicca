local function refuel_logs(_, arg)
    local slot = arg.slot
    local count = arg.count
    turtle.select(16)
    turtle.dig()
    turtle.place()
    for i=1,15 do
        if i ~= slot and turtle.getItemCount(i) > 0 then
            turtle.select(i)
            turtle.drop()
        end
    end
    turtle.select(slot)
    turtle.drop(math.max(0, turtle.getItemCount()-count))
    turtle.craft()
    for i=1,15 do
        if turtle.getItemCount(i) > 0 then
            turtle.select(i)
            turtle.refuel()
        else
            break
        end
    end
    while turtle.suck() do end
end
return refuel_logs