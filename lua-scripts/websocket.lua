return function ()
    local proto_task = require("task")
    local json = require("json")
    local t = require("move")

    --local remote = "replicca.mc.nielsoverkamp.com/api"
    local remote = "localhost:17576"

    local MESSAGE_TYPE = {
        COMMAND="COMMAND",
        TASK_EVENT="TASK_EVENT",
    }

    local COMMANDS = {
        EVAL="EVAL",
        TASK="TASK",
        MOVE="MOVE",
    }

    local function connect(url)
        local id = 1
        local ws, err = http.websocket("ws://"..url .. "/ws/"..id)

        if not ws then
            error(err)
        end

        return ws
    end

    local pos = t.origin()

    local receivedCommand = nil -- Stores the last received command that is not yet consumed.
                                -- If another command arrives while this has a value, that value gets overwritten
                                -- This should however not happen, and if it does it should not have bad consequences
                                -- *knock knock*

    ws = {
        _socket = connect(remote),
        _mid_counter = 0,
    }

    function ws:sendBlocking(msg)
        local mid = self._mid_counter
        self._mid_counter = mid + 1
        msg.mid = mid
        local data = json.encode(msg)
        repeat
            local status = pcall(function () return self._socket.send(data) end)
            if not status then
                print("Lost connection, reconnecting in 1 second...")
                sleep(1)
                self._socket.close()
                self._socket = connect(remote)
            end
        until status
        return mid
    end

    function ws:receiveBlocking()
        while true do
            local status, res = pcall(function() return self._socket.receive() end)
            print("received", status, res)
            if status then
                if res ~= nil then
                    return json.decode(res)
                else
                    -- Timeout, try again
                end
            else
                print("Lost connection, reconnecting...")
                self._socket.close()
                self._socket = connect(remote)
            end
        end
    end

    proto_task.ws = ws

    local function websocketListener()
        while true do
            local msg
            msg = ws:receiveBlocking()
            print("wsListener", msg.c)
            if msg.c == MESSAGE_TYPE.COMMAND then
                if receivedCommand ~= nil then
                    print("WARNING: Received a new command while another was not yet processed")
                    print("-- New command:", msg.b.c)
                    print("-- Previous command", receivedCommand.c)
                end
                receivedCommand = msg.b
                receivedCommand.cid = msg.mid
                os.queueEvent("replicca:received_command")
            elseif msg.c == MESSAGE_TYPE.TASK_EVENT then
                print("queueing event", msg.b.c)
                os.queueEvent("replicca:"..msg.b.c, { b=msg.b.b, mid=msg.mid, cid= msg.cid })
            end
        end
    end

    local function executor()
        while true do
            while receivedCommand == nil do
                os.pullEvent("replicca:received_command")
            end
            local command = receivedCommand
            receivedCommand = nil
            print("executor", command.c)

            if command.c == COMMANDS.EVAL then
                local body = loadstring(command.b)
                if body == nil then
                    local err = "Error: Could not parse: "..command.b
                    print(err)
                    ws:sendBlocking({ cid=command.cid, c="eval_response", b=err})
                else
                    setfenv(body, _ENV)
                    local res = table.pack(pcall(body))
                    if res[1] then
                        print(table.unpack(res, 2, #res))
                    else
                        print("Error: ", table.unpack(res, 2, #res))
                    end
                    ws:sendBlocking({ cid=command.cid, c="eval_response", b=res})
                end
            elseif command.c == COMMANDS.TASK then
                local task_description = command.b
                local task = proto_task.new(command.cid)

                local function execute ()
                    return task:execute(task_description.c, pos, task_description.b)
                end
                print(parallel.waitForAny(execute, executor))
            elseif command.c == COMMANDS.MOVE then
                local success, err, completed = t.runString(pos, command.b)
                local body
                if not success then
                    body = {e=err, c=completed}
                end
                ws:sendBlocking({ cid=command.cid, c="move_response", b=body})
            else
                print("Unknown command "..command.c)
            end
        end
    end

    print(parallel.waitForAny(websocketListener, executor))
end