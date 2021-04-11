dofile("update.lua")

local conf_path = ".cfg"

-- TODO make id functional
local id = 1
if fs.exists(conf_path) then
    local f = fs.open(conf_path, "r")
    id = f.readLine()
    id = tonumber(id)
end

if id == nil then
    local r = http.get("http://"..remote .. "/newId")
    id = r.readAll()
    id = tonumber(id)
end

if id == nil then
    error("Could not find local id file and could not get new one from remote")
    return
end

require("websocket")()
