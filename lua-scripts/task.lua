local task = {}

function task:send_event(event, data)
    print(self.cid)
    return self.ws:sendBlocking({ cid=self.cid, c = event, b = data })
end

function task:pull_event(event, cid)
    local ev, data
    repeat
        print("pulling event", event, cid)
        ev, data = os.pullEvent("replicca:"..event)
        print("pulled event", ev, data.cid)
    until data.cid == cid
    return data.b
end

function task:execute_function(f, args)
    args = args or {}
    local res
    repeat
        res = table.pack(f(table.unpack(args)))
        if res[1] == false then
            local mid = self:send_event("task_error", res[2])
            local continue = self:pull_event("task_error_response", mid)
            print("Continue? " .. tostring(continue))
            if not continue then
                error("Aborting task, reason: " .. res[2])
            end
        end
    until res[1]
    return table.unpack(res, 2, #res)
end

function task:wrap(api)
    local function __index(_, k)
        local f = api[k]
        if type(f) ~= "function" then
            return f
        end
        return function(...)
            return self:execute_function(f, arg)
        end
    end

    local wrapped = { rawApi = api }
    setmetatable(wrapped, { __index = __index })
    return wrapped
end

function task:task_question(q)
    local mid = self:send_event("task_question", q)
    return self:pull_event("task_answer",  mid)
end

local t = require("move")
local wt = task:wrap(t)

local util = require("util"):new(wt)

local inventory = require("inventory"):new()

function task:load(name, pos, taskArgs)
    local f = dofile("tasks/" .. name .. ".lua")
    if f == nil then
        error("failed to load task " .. name .. ".lua in tasks")
    end

    local function subtask_execute(subtask_name)
        return self:load(subtask_name, pos, taskArgs)()
    end

    local env = {
        wt = wt,
        util = util,
        inventory = inventory,
        task = self,
        subtask_execute = subtask_execute,
    }
    setmetatable(env, { __index = _G })
    setfenv(f, env)

    return function()
        local success, msg = pcall(f, pos, taskArgs)
        if not success then
            print("Task error: "..msg)
        end
        return success
    end
end

function task:execute(name, pos, taskArgs, ...)
    local success = self:load(name, pos, taskArgs)()
    if success then
        local mid = self:send_event("task_finish")
        self:pull_event("task_finish_response", mid)
        print("finished task")
    else
        self:send_event("task_cancelled")
        print("cancelled task")
    end
    return success
end

function task.new(cid)
    local new = {}
    setmetatable(new, { __index = task })
    new.cid = cid
    return new
end

return task
