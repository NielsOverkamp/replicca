local tArgs = { ... }

if #tArgs < 1 then
    print("Usage: test_task <task_name>")
    return
end

local task = require("task")
local move = require("move")
local pos = move.origin()

task.execute(tArgs[1], pos, function() while true do print(os.pullEvent("replicca:task_error")) end end)
