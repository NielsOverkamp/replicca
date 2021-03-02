local t = {}

local direction = {
    N = "N",
    E = "E",
    S = "S",
    W = "W",
}

function direction.toIndex(d)
    if d == direction.N then
        return 0
    elseif d == direction.E then
        return 1
    elseif d == direction.S then
        return 2
    elseif d == direction.W then
        return 3
    end
end

function direction.fromIndex(i)
    if i == 0 then
        return direction.N
    elseif i == 1 then
        return direction.E
    elseif i == 2 then
        return direction.S
    elseif i == 3 then
        return direction.W
    end
end

do

    --- MOVE

    function t.f(pos, n)
        n = n or 1
        if turtle.getFuelLevel() < n then
            return false, "fuel", 0
        end
        for i = 1, n do
            while not turtle.forward() do
                if turtle.detect() then
                    t.move_pos_horizontal(pos, i - 1)
                    return false, "obstacle", i - 1
                elseif turtle.getFuelLevel() == 0 then
                    error("Fuel level 0. Starting fuel checks failed.")
                else
                    print("Move creature! Shoo! I need to go forward!")
                    sleep(0.5)
                end
            end
        end
        t.move_pos_horizontal(pos, n)
        return true, n
    end

    function t.u(pos, n)
        n = n or 1
        if turtle.getFuelLevel() < n then
            return false, "fuel", 0
        end
        for i = 1, n do
            while not turtle.up() do
                if turtle.detectUp() then
                    t.move_pos_vertical(pos, i - 1)
                    return false, "obstacle", i - 1
                else
                    print("Move creature! Shoo! I need to go up!")
                    sleep(0.5)
                end
            end
        end
        t.move_pos_vertical(pos, n)
        return true, n
    end

    function t.d(pos, n)
        n = n or 1
        if turtle.getFuelLevel() < n then
            return false, "fuel", 0
        end
        for i = 1, n do
            while not turtle.down() do
                if turtle.detectDown() then
                    t.move_pos_vertical(pos, -(i - 1))
                    return false, "obstacle", i - 1
                else
                    print("Move creature! Shoo! I need to go down!")
                    sleep(0.5)
                end
            end
        end
        t.move_pos_vertical(pos, -n)
        return true, n
    end

    function t.b(pos, n)
        n = n or 1
        if turtle.getFuelLevel() < n then
            return false, "fuel", 0
        end
        for i = 1, n do
            if not turtle.back() then
                t.turn(pos, 2)
                repeat
                    if turtle.detect() then
                        t.turn(pos, 2)
                        t.move_pos_horizontal(pos, -(i - 1))
                        return false, "obstacle", i - 1
                    else
                        print("Move creature! Shoo! I need to go back!")
                        sleep(0.5)
                    end
                until turtle.forward()
                t.turn(pos, 2)
            end
        end
        t.move_pos_horizontal(pos, -n)
        return true, n
    end

    function t.r(pos, n)
        n = n or 1
        for _ = 1, n do
            turtle.turnRight()
        end
        t.turn(pos, n)
        return true
    end

    function t.l(pos, n)
        n = n or 1
        for _ = 1, n do
            turtle.turnLeft()
        end
        t.turn(pos, -n)
        return true
    end

    --- MINE AND MOVE

    function t.mf(pos, n)
        n = n or 1
        if turtle.getFuelLevel() < n then
            return false, "fuel", 0
        end
        for i = 1, n do
            turtle.dig()
            local success, err = t.f(pos, 1)
            if not success then
                return success, err, i-1
            end
        end
        return true, n
    end

    function t.mu(pos, n)
        n = n or 1
        if turtle.getFuelLevel() < n then
            return false, "fuel", 0
        end
        for i = 1, n do
            turtle.digUp()
            local success, err = t.u(pos, 1)
            if not success then
                return success, err, i-1
            end
        end
        return true, n
    end

    function t.md(pos, n)
        n = n or 1
        if turtle.getFuelLevel() < n then
            return false, "fuel", 0
        end
        for i = 1, n do
            turtle.digDown()
            local success, err = t.d(pos, 1)
            if not success then
                return success, err, i-1
            end
        end
        return true, n
    end

    function t.mb(pos, n)
        n = n or 1
        for i = 1, n do
            local success, err = t.b(pos, 1)
            if not success and err == "obstacle" then
                t.l(pos, 2)
                local delta
                success, err, delta = t.mf(pos, n-(i-1))
                t.r(pos, 2)
                return success, err, delta
            elseif not success then
                return success, err, i-1
            end
        end
        return true, n
    end
    
    --- POSITION LOGIC

    function t.move_pos_horizontal(pos, n)
        n = n or 1
        local d = pos.direction
        if d == direction.N then
            pos.coordinate.z = pos.coordinate.z - n
        elseif d == direction.E then
            pos.coordinate.x = pos.coordinate.x + n
        elseif d == direction.S then
            pos.coordinate.z = pos.coordinate.z + n
        elseif d == direction.W then
            pos.coordinate.x = pos.coordinate.x - n
        end
    end

    function t.move_pos_vertical(pos, n)
        pos.coordinate.y = pos.coordinate.y + n
    end

    function t.turn(pos, n)
        pos.direction = direction.fromIndex((direction.toIndex(pos.direction) + n) % 4)
    end

    function t.origin()
        return {
            coordinate = { x = 0, y = 0, z = 0 },
            direction = direction.N,
        }
    end

    --- COMMAND PARSING

    function t.runString(pos, s)
        local l = string.len(s)
        local i = 1
        while true do
            local j = i
            local c = string.sub(s, j, j)
            if c == "m" then
                c = string.sub(s, j, j+1)
                j = j + 2
            else
                j = j + 1
            end

            local k = 1
            local g = j
            while true do
                local tryK = tonumber(string.sub(s, j, g))

                if tryK == nil then
                    j = g
                    break
                else
                    k = tryK
                end
                
                if g == l then
                    j = l + 1
                    break
                end
                g = g + 1
            end

            local f = t[c]
            if type(f) == "function" then
                local success, err = f(pos, k)
                if not success then
                    return false, "Could not complete move "..c..k.." due to \""..err.."\"", i
                end
            else
                return false, "Unknown command "..c, i
            end

            i = j
            
            if i > l then
                break
            end
        end
        return true
    end
end

return t