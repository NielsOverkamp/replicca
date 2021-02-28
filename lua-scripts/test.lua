local task = require("task")
local t = require("move")
local wt = task.wrap(t)
local pretty = require("cc.pretty")

local pos = t.origin()

local fell
do

end


local function onError(err)
    print("Error: "..err)
    print("Aborting task")
    return false
end

local function errorHandler()
    while true do
        local err = os.pullEvent("replicca:task_error")
        local continue = onError(err)
        os.queueEvent("replicca:task_error_response", continue)
    end
end

local function posUpdateHandler()
    while true do
        local newPos = os.pullEvent("replicca:pos_update")
        print("dummy Sending and saving pos update")
        print(pretty.pretty(newPos))
    end
end

print(parallel.waitForAny(fell, errorHandler, posUpdateHandler))
