--local remote_url = "https://files.nielsoverkamp.com/replicca"
local remote_url = "http://localhost:17576/files/replicca"
--local json_url = "https://raw.githubusercontent.com/rxi/json.lua/master/json.lua"
local json_url = "http://localhost:17576/files/replicca/json.lua"

local function download_file(url, path)
    local r = http.get(url)
    if r == nil then
        return "Could not get " .. path .. " from " .. url
    end
    local f = fs.open(path, "w")
    f.write(r.readAll())
    f.close()
    r.close()
end

local err = download_file(remote_url .. "/startup.lua", "/startup.lua")
if err ~= nil then
    error(err .. " rebooting in 5...")
    os.sleep(5)
    os.reboot()
    return
end

err = download_file(json_url, "/json.lua")
if err ~= nil then
    error(err)
    return
end

for _, v in ipairs({ "update.lua", "install.lua", "websocket.lua", "move.lua", "task.lua", "test.lua" }) do
    err = download_file(remote_url .. "/" .. v, "/" .. v)
    if err ~= nil then
        error(err)
        return
    end
end

fs.makeDir("/tasks")

for _, v in ipairs({ "fell.lua" }) do
    err = download_file(remote_url .. "/tasks/" .. v, "/tasks/" .. v)
    if err ~= nil then
        error(err)
        return
    end
end

print("installed/updated files")
--print("updated startup.lua, rebooting in 3")
--for i=2,0,-1 do
--    os.sleep(1)
--    print(i)
--end
--os.reboot()
