local task = {}

function task.execute_function(f, args)
    local res
    repeat
        res = table.pack(f(table.unpack(args)))
        if res[1] == false then
            os.queueEvent("replicca:task_error", table.unpack(res, 2, #res))
            local _, continue = os.pullEvent("replicca:task_error_response")
            print("Continue? " .. tostring(continue))
            if not continue then
                error("Aborting fell task, reason: " .. res[2])
            end
        end
    until res[1]
    return table.unpack(res, 2, #res)
end

function task.wrap(api)
    local function __index(_, k)
        local f = api[k]
        if type(f) ~= "function" then
            return f
        end
        return function(...)
            task.execute_function(f, arg)
        end
    end

    local wrapped = { rawApi = api }
    setmetatable(wrapped, { __index = __index })
    return wrapped
end

local t = require("move")
local wt = task.wrap(t)

local util = require("util"):new(wt)

local inventory = require("inventory"):new()

function task.load(name, pos)
    local f = dofile("tasks/" .. name .. ".lua")
    if f == nil then
        error("failed to load task " .. name .. ".lua in tasks")
    end

    local function subtask_execute(subtask_name)
        task.load(subtask_name, pos)()
    end

    local env = {
        wt = wt,
        util = util,
        inventory = inventory,
        subtask_execute = subtask_execute
    }
    print(inventory)
    print(inventory.update)
    setmetatable(env, { __index = _G })
    setfenv(f, env)

    return function()
        f(pos)
    end
end


function task.execute(name, pos, ...)
    local subtask_exec = task.load(name, pos)
    local function task_exec()
        subtask_exec()
        os.queueEvent("replicca:task_finish")
        os.pullEvent("replicca:task_finish_response")
    end

    print(parallel.waitForAny(task_exec, table.unpack(arg)))
end

return task
