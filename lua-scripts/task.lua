local task = {}

local EVENT_TYPE = {
    FROM_REMOTE="DOWN",
    FROM_TURTLE="UP"
}

function task.execute_function(f, args)
    args = args or {}
    local res
    repeat
        res = table.pack(f(table.unpack(args)))
        if res[1] == false then
            os.queueEvent("replicca:task_error", table.unpack(res, 2, #res))
            local _, continue = os.pullEvent("replicca:task_error_response")
            print("Continue? " .. tostring(continue))
            if not continue then
                error("Aborting task, reason: " .. res[2])
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
            return task.execute_function(f, arg)
        end
    end

    local wrapped = { rawApi = api }
    setmetatable(wrapped, { __index = __index })
    return wrapped
end

function task.task_question(...)
    os.queueEvent("replicca:task_question", EVENT_TYPE.FROM_TURTLE, table.unpack(arg))
    local _, _, answer = os.pullEvent("replicca:task_answer")
    return answer
end

local t = require("move")
local wt = task.wrap(t)

local util = require("util"):new(wt)

local inventory = require("inventory"):new()

function task.load(name, pos, taskArgs)
    local f = dofile("tasks/" .. name .. ".lua")
    if f == nil then
        error("failed to load task " .. name .. ".lua in tasks")
    end

    local function subtask_execute(subtask_name)
        return task.load(subtask_name, pos, taskArgs)()
    end

    local env = {
        wt = wt,
        util = util,
        inventory = inventory,
        task = task,
        subtask_execute = subtask_execute,
    }
    print(inventory)
    print(inventory.update)
    setmetatable(env, { __index = _G })
    setfenv(f, env)

    return function()
        return f(pos, taskArgs)
    end
end


function task.execute(name, pos, taskArgs, ...)
    local subtask_exec = task.load(name, pos, taskArgs)
    local function task_exec()
        subtask_exec()
        os.queueEvent("replicca:task_finish", EVENT_TYPE.FROM_TURTLE)
        os.pullEvent("replicca:task_finish_response")
    end

    print(parallel.waitForAny(task_exec, table.unpack(arg)))
end

return task
