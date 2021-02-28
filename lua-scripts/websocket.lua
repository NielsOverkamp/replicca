return function ()
    local task = require("task")
    local json = require("json")
    local t = require("move")

    --local remote = "replicca.mc.nielsoverkamp.com/api"
    local remote = "localhost:17576"


    local COMMANDS = {
        EVAL="EVAL",
        TASK="TASK",
    }

    local function connect(url)
        local ws, err = http.websocket("ws://"..url .. "/ws")

        if not ws then
            error(err)
        end

        return ws
    end

    local function receiveBlocking(ws)
        while true do
            local status, res = pcall(function() return ws.receive(60) end)
            print(status, res)
            if status then
                if res ~= nil then
                    return ws, json.decode(res)
                else
                    -- Continue, try again
                end
            else
                print("Lost connection, reconnecting...")
                ws.close()
                ws = connect(remote)
            end
        end
    end

    local function sendBlocking(ws, msg)
        local data = json.encode(msg)
        while true do
            local status = pcall(function () return ws.send(data) end)
            if status then
                return ws
            else
                print("Lost connection, reconnecting...")
                ws.close()
                ws = connect(remote)
            end
        end
    end

    local ws = connect(remote)

    local pos = t.origin()

    while true do
        local msg
        ws, msg = receiveBlocking(ws)

        print(msg)

        if msg.c == COMMANDS.EVAL then
            local body = loadstring(msg.b)
            if body == nil then
                local err = "Error: Could not parse: "..msg.b
                print(err)
                ws = sendBlocking(ws, {r=msg.id, err=err})
            else
                setfenv(body, _ENV)
                local res = table.pack(pcall(body))
                if res[1] then
                    print(table.unpack(res, 2, #res))
                else
                    print("Error: ", table.unpack(res, 2, #res))
                end
                ws = sendBlocking(ws, {r=msg.id, c="eval_response", b=res})
            end
        elseif msg.c == COMMANDS.TASK then
            local task_name = msg.b

            local function sendEvents()
                while true do
                    local ev, data = os.pullEvent()
                    print(ev, data)
                    if string.sub(ev, 1, 9) == "replicca:" then
                        local event = string.sub(ev, 10)
                        ws = sendBlocking(ws, {c=event, b=data})
                    end
                end
            end

            local function receiveCommands()
                while true do
                    local msg
                    ws, msg = receiveBlocking(ws)
                    os.queueEvent("replica:"..msg.c, msg.b)
                end
            end

            task.execute(task_name, pos, sendEvents, receiveCommands)
        end
    end
end